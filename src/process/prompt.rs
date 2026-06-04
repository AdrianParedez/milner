use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use super::parser::ParsedCommand;
use super::parser::parse_command_line;
use super::{RunError, execute_command};

const PROMPT: &str = "keel> ";

pub fn run_prompt() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();

    match run_prompt_with_io(stdin.lock(), stdout.lock(), stderr.lock()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("run: {err}");
            125
        }
    }
}

struct PromptState {
    cwd: PathBuf,
    last_status: i32,
    should_exit: bool,
    exit_code: i32,
}

impl PromptState {
    fn new() -> Result<Self, io::Error> {
        Ok(Self {
            cwd: std::env::current_dir()?,
            last_status: 0,
            should_exit: false,
            exit_code: 0,
        })
    }
}

fn run_prompt_with_io<R, W, E>(mut input: R, mut output: W, mut errors: E) -> Result<i32, io::Error>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let mut state = PromptState::new()?;
    let mut line = String::new();

    loop {
        output.write_all(PROMPT.as_bytes())?;
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

        match parse_command_line(command_line) {
            Ok(command) => {
                run_parsed_command(command, &mut state, &mut output, &mut errors)?;
                if state.should_exit {
                    return Ok(state.exit_code);
                }
            }
            Err(err) => {
                writeln!(errors, "run: {err}")?;
                state.last_status = RunError::Parse(err).exit_code();
            }
        }
    }
}

fn run_parsed_command<W, E>(
    command: ParsedCommand,
    state: &mut PromptState,
    output: &mut W,
    errors: &mut E,
) -> Result<(), io::Error>
where
    W: Write,
    E: Write,
{
    match run_builtin(command, state, output) {
        BuiltinResult::Handled(Ok(())) => {}
        BuiltinResult::Handled(Err(err)) => {
            writeln!(errors, "run: {err}")?;
            state.last_status = err.exit_code();
        }
        BuiltinResult::External(command) => match execute_command(command) {
            Ok(code) => state.last_status = code as i32,
            Err(err) => {
                writeln!(errors, "run: {err}")?;
                state.last_status = err.exit_code();
            }
        },
    }

    Ok(())
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
    CurrentDirectory(io::Error),
    WriteOutput(io::Error),
}

impl BuiltinError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::MissingOperand(_)
            | Self::ExtraOperand(_)
            | Self::NonUnicodeExitCode
            | Self::InvalidExitCode(_) => 2,
            Self::ChangeDirectory { .. } | Self::CurrentDirectory(_) | Self::WriteOutput(_) => 1,
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
            Self::CurrentDirectory(err) => write!(f, "cd: cannot read current directory: {err}"),
            Self::WriteOutput(err) => write!(f, "pwd: cannot write output: {err}"),
        }
    }
}

impl std::error::Error for BuiltinError {}

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

    if command.program == OsStr::new("exit") {
        return BuiltinResult::Handled(exit_prompt(command.args, state));
    }

    BuiltinResult::External(command)
}

fn change_directory(args: Vec<OsString>, state: &mut PromptState) -> Result<(), BuiltinError> {
    let mut args = args.into_iter();
    let path = args.next().ok_or(BuiltinError::MissingOperand("cd"))?;
    if args.next().is_some() {
        return Err(BuiltinError::ExtraOperand("cd"));
    }

    let path = PathBuf::from(path);
    std::env::set_current_dir(&path).map_err(|source| BuiltinError::ChangeDirectory {
        path: path.clone(),
        source,
    })?;
    state.cwd = std::env::current_dir().map_err(BuiltinError::CurrentDirectory)?;
    state.last_status = 0;
    Ok(())
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::run_prompt_with_io;

    #[test]
    fn empty_input_prompts_again_until_eof() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(Cursor::new("\n"), &mut output, &mut errors).unwrap();

        assert_eq!(code, 0);
        assert_eq!(String::from_utf8(output).unwrap(), "keel> keel> ");
        assert_eq!(String::from_utf8(errors).unwrap(), "");
    }

    #[test]
    fn parser_errors_are_non_fatal() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(Cursor::new("tool |\n"), &mut output, &mut errors).unwrap();

        assert_eq!(code, 2);
        assert_eq!(String::from_utf8(output).unwrap(), "keel> keel> ");
        assert!(
            String::from_utf8(errors)
                .unwrap()
                .contains("unsupported operator `|`")
        );
    }
}
