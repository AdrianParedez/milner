use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use super::config::default_history_path;
use super::parser::parse_execution_line;
use super::parser::{CommandSpec, ExecutionPlan, InputSpec, OutputSpec, ParsedCommand};
use super::{ExecutionOptions, RunError, execute_plan};

pub fn run_prompt(options: ExecutionOptions) -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();

    match run_prompt_with_io(stdin.lock(), stdout.lock(), stderr.lock(), options) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("milner: {err}");
            125
        }
    }
}

struct PromptState {
    cwd: PathBuf,
    execution_options: ExecutionOptions,
    history: Option<History>,
    last_status: i32,
    should_exit: bool,
    exit_code: i32,
}

impl PromptState {
    fn new(mut options: ExecutionOptions) -> Result<Self, io::Error> {
        let cwd = match &options.cwd {
            Some(path) => path.clone(),
            None => std::env::current_dir()?,
        };
        options.cwd = Some(cwd.clone());
        let history = History::open(&options.config.history)?;

        Ok(Self {
            cwd,
            execution_options: options,
            history,
            last_status: 0,
            should_exit: false,
            exit_code: 0,
        })
    }

    fn prompt_text(&self) -> &str {
        &self.execution_options.config.prompt.text
    }
}

struct History {
    file: File,
}

impl History {
    fn open(config: &super::config::HistoryConfig) -> Result<Option<Self>, io::Error> {
        if !config.enabled {
            return Ok(None);
        }

        let path = match &config.path {
            Some(path) => path.clone(),
            None => default_history_path().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "APPDATA is required for history")
            })?,
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map(|file| Some(Self { file }))
    }

    fn record(&mut self, command_line: &str) -> Result<(), io::Error> {
        if should_record_history(command_line) {
            writeln!(self.file, "{command_line}")?;
        }

        Ok(())
    }
}

fn run_prompt_with_io<R, W, E>(
    mut input: R,
    mut output: W,
    mut errors: E,
    options: ExecutionOptions,
) -> Result<i32, io::Error>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let mut state = PromptState::new(options)?;
    let mut line = String::new();

    loop {
        output.write_all(state.prompt_text().as_bytes())?;
        output.flush()?;

        line.clear();
        let bytes = input.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(state.last_status);
        }

        let command_line = trim_line_ending(&line);
        if command_line.trim().is_empty() {
            continue;
        }

        match parse_execution_line(command_line) {
            Ok(plan) => {
                if let Some(history) = &mut state.history {
                    history.record(command_line)?;
                }
                run_execution_plan(plan, &mut state, &mut output, &mut errors)?;
                if state.should_exit {
                    return Ok(state.exit_code);
                }
            }
            Err(err) => {
                writeln!(errors, "milner: {err}")?;
                state.last_status = RunError::Parse(err).exit_code();
            }
        }
    }
}

fn run_execution_plan<W, E>(
    plan: ExecutionPlan,
    state: &mut PromptState,
    output: &mut W,
    errors: &mut E,
) -> Result<(), io::Error>
where
    W: Write,
    E: Write,
{
    match run_prompt_plan(plan, state, output) {
        PromptPlanResult::Handled(Ok(())) => {}
        PromptPlanResult::Handled(Err(err)) => {
            writeln!(errors, "milner: {err}")?;
            state.last_status = err.exit_code();
        }
        PromptPlanResult::External(plan) => match execute_plan(*plan, &state.execution_options) {
            Ok(code) => state.last_status = code as i32,
            Err(err) => {
                writeln!(errors, "milner: {err}")?;
                state.last_status = err.exit_code();
            }
        },
    }

    Ok(())
}

enum PromptPlanResult {
    Handled(Result<(), BuiltinError>),
    External(Box<ExecutionPlan>),
}

enum BuiltinResult {
    Handled(Result<(), BuiltinError>),
    External(ParsedCommand),
}

#[derive(Debug)]
enum BuiltinError {
    MissingOperand(&'static str),
    ExtraOperand(&'static str),
    NonUnicodeExitCode,
    InvalidExitCode(OsString),
    ChangeDirectory { path: PathBuf, source: io::Error },
    WriteOutput(io::Error),
    UnsupportedRedirection(&'static str),
    UnsupportedPipeline(&'static str),
}

impl BuiltinError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::MissingOperand(_)
            | Self::ExtraOperand(_)
            | Self::NonUnicodeExitCode
            | Self::InvalidExitCode(_) => 2,
            Self::ChangeDirectory { .. } | Self::WriteOutput(_) => 1,
            Self::UnsupportedRedirection(_) | Self::UnsupportedPipeline(_) => 2,
        }
    }
}

impl std::fmt::Display for BuiltinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingOperand(name) => write!(f, "{name}: missing operand"),
            Self::ExtraOperand(name) => write!(f, "{name}: too many operands"),
            Self::NonUnicodeExitCode => write!(f, "exit: code must be valid Unicode"),
            Self::InvalidExitCode(code) => {
                write!(f, "exit: invalid code `{}`", code.to_string_lossy())
            }
            Self::ChangeDirectory { path, source } => {
                write!(f, "cd: cannot change to `{}`: {source}", path.display())
            }
            Self::WriteOutput(err) => write!(f, "pwd: cannot write output: {err}"),
            Self::UnsupportedRedirection(name) => {
                write!(f, "{name}: redirection is not supported for built-ins")
            }
            Self::UnsupportedPipeline(name) => {
                write!(f, "{name}: pipelines are not supported for built-ins")
            }
        }
    }
}

impl std::error::Error for BuiltinError {}

fn run_prompt_plan<W>(
    plan: ExecutionPlan,
    state: &mut PromptState,
    output: &mut W,
) -> PromptPlanResult
where
    W: Write,
{
    match plan {
        ExecutionPlan::Command(command) => run_command_spec(command, state, output),
        ExecutionPlan::Pipeline { left, right } => {
            if let Some(name) = builtin_name(&left.command) {
                return PromptPlanResult::Handled(Err(BuiltinError::UnsupportedPipeline(name)));
            }

            if let Some(name) = builtin_name(&right.command) {
                return PromptPlanResult::Handled(Err(BuiltinError::UnsupportedPipeline(name)));
            }

            PromptPlanResult::External(Box::new(ExecutionPlan::Pipeline { left, right }))
        }
    }
}

fn run_command_spec<W>(
    command: CommandSpec,
    state: &mut PromptState,
    output: &mut W,
) -> PromptPlanResult
where
    W: Write,
{
    let Some(name) = builtin_name(&command.command) else {
        return PromptPlanResult::External(Box::new(ExecutionPlan::Command(command)));
    };

    if command.stdin != InputSpec::Inherit
        || command.stdout != OutputSpec::Inherit
        || command.stderr != OutputSpec::Inherit
    {
        return PromptPlanResult::Handled(Err(BuiltinError::UnsupportedRedirection(name)));
    }

    match run_builtin(command.command, state, output) {
        BuiltinResult::Handled(result) => PromptPlanResult::Handled(result),
        BuiltinResult::External(command) => {
            PromptPlanResult::External(Box::new(ExecutionPlan::Command(CommandSpec {
                command,
                stdin: InputSpec::Inherit,
                stdout: OutputSpec::Inherit,
                stderr: OutputSpec::Inherit,
            })))
        }
    }
}

fn run_builtin<W>(command: ParsedCommand, state: &mut PromptState, output: &mut W) -> BuiltinResult
where
    W: Write,
{
    if command.program == OsStr::new("cd") {
        return BuiltinResult::Handled(change_directory(command.args, state));
    }

    if command.program == OsStr::new("pwd") {
        return BuiltinResult::Handled(print_working_directory(command.args, state, output));
    }

    if command.program == OsStr::new("complete") {
        return BuiltinResult::Handled(complete(command.args, state, output));
    }

    if command.program == OsStr::new("exit") {
        return BuiltinResult::Handled(exit_prompt(command.args, state));
    }

    BuiltinResult::External(command)
}

fn builtin_name(command: &ParsedCommand) -> Option<&'static str> {
    if command.program == OsStr::new("cd") {
        Some("cd")
    } else if command.program == OsStr::new("complete") {
        Some("complete")
    } else if command.program == OsStr::new("pwd") {
        Some("pwd")
    } else if command.program == OsStr::new("exit") {
        Some("exit")
    } else {
        None
    }
}

fn complete<W>(
    args: Vec<OsString>,
    state: &mut PromptState,
    output: &mut W,
) -> Result<(), BuiltinError>
where
    W: Write,
{
    let mut args = args.into_iter();
    let prefix = args.next().unwrap_or_default();
    if args.next().is_some() {
        return Err(BuiltinError::ExtraOperand("complete"));
    }

    let Some(prefix) = prefix.to_str() else {
        return Err(BuiltinError::ExtraOperand("complete"));
    };

    for suggestion in completion_suggestions(prefix, state) {
        writeln!(output, "{suggestion}").map_err(BuiltinError::WriteOutput)?;
    }

    state.last_status = 0;
    Ok(())
}

fn completion_suggestions(prefix: &str, state: &PromptState) -> Vec<String> {
    let mut suggestions = ["cd", "complete", "exit", "pwd"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    suggestions.extend(state.execution_options.config.aliases.keys().cloned());
    suggestions.sort();
    suggestions.dedup();
    suggestions
        .into_iter()
        .filter(|suggestion| suggestion.starts_with(prefix))
        .collect()
}

fn change_directory(args: Vec<OsString>, state: &mut PromptState) -> Result<(), BuiltinError> {
    let mut args = args.into_iter();
    let path = args.next().ok_or(BuiltinError::MissingOperand("cd"))?;
    if args.next().is_some() {
        return Err(BuiltinError::ExtraOperand("cd"));
    }

    let requested = PathBuf::from(path);
    let candidate = resolve_shell_path(&state.cwd, &requested);
    state.cwd =
        std::fs::canonicalize(&candidate).map_err(|source| BuiltinError::ChangeDirectory {
            path: requested.clone(),
            source,
        })?;
    state.execution_options.cwd = Some(state.cwd.clone());
    if !state.cwd.is_dir() {
        return Err(BuiltinError::ChangeDirectory {
            path: requested,
            source: io::Error::new(io::ErrorKind::NotADirectory, "not a directory"),
        });
    }
    state.last_status = 0;
    Ok(())
}

fn resolve_shell_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn print_working_directory<W>(
    args: Vec<OsString>,
    state: &mut PromptState,
    output: &mut W,
) -> Result<(), BuiltinError>
where
    W: Write,
{
    if !args.is_empty() {
        return Err(BuiltinError::ExtraOperand("pwd"));
    }

    writeln!(output, "{}", state.cwd.display()).map_err(BuiltinError::WriteOutput)?;
    state.last_status = 0;
    Ok(())
}

fn exit_prompt(args: Vec<OsString>, state: &mut PromptState) -> Result<(), BuiltinError> {
    let mut args = args.into_iter();
    let code = match args.next() {
        Some(code) => parse_exit_code(code)?,
        None => state.last_status,
    };

    if args.next().is_some() {
        return Err(BuiltinError::ExtraOperand("exit"));
    }

    state.should_exit = true;
    state.exit_code = code;
    state.last_status = code;
    Ok(())
}

fn parse_exit_code(code: OsString) -> Result<i32, BuiltinError> {
    let Some(code_text) = code.to_str() else {
        return Err(BuiltinError::NonUnicodeExitCode);
    };

    let parsed = code_text
        .parse::<i32>()
        .map_err(|_| BuiltinError::InvalidExitCode(code.clone()))?;

    if (0..=255).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(BuiltinError::InvalidExitCode(code))
    }
}

fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

fn should_record_history(command_line: &str) -> bool {
    let lowered = command_line.to_ascii_lowercase();
    ![
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "credential",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::super::ExecutionOptions;
    use super::run_prompt_with_io;

    #[test]
    fn empty_input_prompts_again_until_eof() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(
            Cursor::new("\n"),
            &mut output,
            &mut errors,
            ExecutionOptions::default(),
        )
        .unwrap();

        assert_eq!(code, 0);
        assert_eq!(String::from_utf8(output).unwrap(), "milner> milner> ");
        assert_eq!(String::from_utf8(errors).unwrap(), "");
    }

    #[test]
    fn parser_errors_are_non_fatal() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(
            Cursor::new("tool &&\n"),
            &mut output,
            &mut errors,
            ExecutionOptions::default(),
        )
        .unwrap();

        assert_eq!(code, 2);
        assert_eq!(String::from_utf8(output).unwrap(), "milner> milner> ");
        assert!(
            String::from_utf8(errors)
                .unwrap()
                .contains("unsupported operator `&&`")
        );
    }
}
