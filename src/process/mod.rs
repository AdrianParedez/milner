mod command_line;
mod config;
mod handles;
mod parser;
mod prompt;
mod win32;

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use config::{ConfigSource, ShellConfig};
use parser::{
    CommandSpec, ExecutionPlan, InputSpec, OutputSpec, ParseError, ParsedCommand,
    parse_execution_line,
};

#[derive(Debug)]
pub enum RunError {
    Usage,
    Parse(ParseError),
    Config(config::ConfigError),
    InvalidExecutionPlan(&'static str),
    InvalidCwd {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidEnvironmentName(OsString),
    InvalidEnvironmentAssignment(OsString),
    InvalidTimeout(OsString),
    AliasCycle(Vec<String>),
    ExecutableNotFound {
        program: OsString,
        searched: Vec<PathBuf>,
    },
    Io {
        context: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    NonUnicodeCommandLine,
    EmptyProgram,
    InteriorNul,
    UnsupportedBatchTarget,
    InvalidHandle(&'static str),
    Win32 {
        context: &'static str,
        code: u32,
    },
    WaitFailed(u32),
    UnexpectedWait(u32),
    ExitCodeUnavailable(u32),
    Cancelled {
        timeout_ms: u32,
    },
}

impl RunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::Parse(_) => 2,
            Self::Config(_) => 125,
            Self::InvalidExecutionPlan(_) => 2,
            Self::InvalidEnvironmentName(_)
            | Self::InvalidEnvironmentAssignment(_)
            | Self::InvalidTimeout(_)
            | Self::AliasCycle(_) => 2,
            Self::NonUnicodeCommandLine => 2,
            Self::EmptyProgram
            | Self::InteriorNul
            | Self::UnsupportedBatchTarget
            | Self::InvalidCwd { .. }
            | Self::ExecutableNotFound { .. }
            | Self::InvalidHandle(_)
            | Self::Io { .. }
            | Self::Win32 { .. } => 125,
            Self::WaitFailed(_) | Self::UnexpectedWait(_) | Self::ExitCodeUnavailable(_) => 126,
            Self::Cancelled { .. } => CANCELLED_EXIT_CODE as i32,
        }
    }
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => write!(
                f,
                "usage: milner.exe [--no-config] [--config <file>] [--cwd <dir>] [--set-env NAME=VALUE] [--unset-env NAME] [--timeout-ms <ms>] <program> <args...>\n       milner.exe [options] --line <command-line>\n       milner.exe [options] --prompt"
            ),
            Self::Parse(err) => write!(f, "{err}"),
            Self::Config(err) => write!(f, "{err}"),
            Self::InvalidExecutionPlan(message) => write!(f, "{message}"),
            Self::InvalidCwd { path, source } => {
                write!(f, "cwd `{}` is invalid: {source}", path.display())
            }
            Self::InvalidEnvironmentName(name) => {
                write!(
                    f,
                    "environment variable name `{}` is invalid",
                    name.to_string_lossy()
                )
            }
            Self::InvalidEnvironmentAssignment(assignment) => write!(
                f,
                "environment assignment `{}` must be NAME=VALUE",
                assignment.to_string_lossy()
            ),
            Self::InvalidTimeout(value) => write!(
                f,
                "timeout `{}` must be a positive millisecond value",
                value.to_string_lossy()
            ),
            Self::AliasCycle(names) => {
                write!(f, "alias cycle detected: {}", names.join(" -> "))
            }
            Self::ExecutableNotFound { program, searched } => {
                write!(
                    f,
                    "executable `{}` not found; searched PATH entries and did not search the current directory for bare names",
                    program.to_string_lossy()
                )?;
                if !searched.is_empty() {
                    write!(f, ":")?;
                    for path in searched {
                        write!(f, " {}", path.display())?;
                    }
                }
                Ok(())
            }
            Self::Io {
                context,
                path,
                source,
            } => {
                write!(f, "{context} `{}` failed: {source}", path.display())
            }
            Self::NonUnicodeCommandLine => write!(f, "--line input must be valid Unicode"),
            Self::EmptyProgram => write!(f, "program must not be empty"),
            Self::InteriorNul => write!(f, "arguments must not contain an interior null"),
            Self::UnsupportedBatchTarget => {
                write!(f, "batch targets are not supported")
            }
            Self::InvalidHandle(context) => write!(f, "{context} returned an invalid handle"),
            Self::Win32 { context, code } => write!(f, "{context} failed with Win32 error {code}"),
            Self::WaitFailed(code) => {
                write!(f, "WaitForSingleObject failed with Win32 error {code}")
            }
            Self::UnexpectedWait(code) => {
                write!(f, "WaitForSingleObject returned unexpected status {code}")
            }
            Self::ExitCodeUnavailable(code) => {
                write!(f, "GetExitCodeProcess failed with Win32 error {code}")
            }
            Self::Cancelled { timeout_ms } => {
                write!(f, "foreground command cancelled after {timeout_ms} ms")
            }
        }
    }
}

impl std::error::Error for RunError {}

const CANCELLED_EXIT_CODE: u32 = 130;
const FOREGROUND_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, Default)]
pub(super) struct ExecutionOptions {
    cwd: Option<PathBuf>,
    environment: EnvironmentSpec,
    timeout_ms: Option<u32>,
    config: ShellConfig,
}

#[derive(Clone, Debug, Default)]
struct EnvironmentSpec {
    changes: Vec<EnvironmentChange>,
}

#[derive(Clone, Debug)]
enum EnvironmentChange {
    Set { name: OsString, value: OsString },
    Unset { name: OsString },
}

struct PreparedEnvironment {
    block: Option<Vec<u16>>,
    lookup: BTreeMap<String, OsString>,
}

struct ResolvedCommand {
    application_name: PathBuf,
    argv0: OsString,
    args: Vec<OsString>,
}

struct PreparedLaunch {
    application_name: Vec<u16>,
    current_directory: Option<Vec<u16>>,
    environment: Option<Vec<u16>>,
}

pub fn run_from_env() -> Result<u32, RunError> {
    let mut args = std::env::args_os();
    let _runner = args.next();
    let mut options = ExecutionOptions::default();
    let mut config_source = ConfigSource::Default;
    let first = loop {
        let Some(arg) = args.next() else {
            return Err(RunError::Usage);
        };

        if arg == "--no-config" {
            config_source = ConfigSource::Disabled;
            continue;
        }

        if arg == "--config" {
            let path = args.next().ok_or(RunError::Usage)?;
            config_source = ConfigSource::Path(PathBuf::from(path));
            continue;
        }

        if arg == "--cwd" {
            let cwd = args.next().ok_or(RunError::Usage)?;
            options.cwd = Some(validate_cwd(PathBuf::from(cwd))?);
            continue;
        }

        if arg == "--set-env" {
            let assignment = args.next().ok_or(RunError::Usage)?;
            options.environment.set(assignment)?;
            continue;
        }

        if arg == "--unset-env" {
            let name = args.next().ok_or(RunError::Usage)?;
            options.environment.unset(name)?;
            continue;
        }

        if arg == "--timeout-ms" {
            let value = args.next().ok_or(RunError::Usage)?;
            options.timeout_ms = Some(parse_timeout_ms(value)?);
            continue;
        }

        break arg;
    };
    options.config = ShellConfig::load(config_source).map_err(RunError::Config)?;

    if first == "--prompt" {
        if args.next().is_some() {
            return Err(RunError::Usage);
        }

        return Ok(prompt::run_prompt(options) as u32);
    }

    let command = if first == "--line" {
        let input = args.next().ok_or(RunError::Usage)?;
        if args.next().is_some() {
            return Err(RunError::Usage);
        }

        let input = input
            .into_string()
            .map_err(|_| RunError::NonUnicodeCommandLine)?;
        let plan = parse_execution_line(&input).map_err(RunError::Parse)?;
        return execute_plan(plan, &options);
    } else {
        ParsedCommand {
            program: first,
            args: args.collect::<Vec<OsString>>(),
        }
    };

    execute_command(command, &options)
}

fn execute_command(command: ParsedCommand, options: &ExecutionOptions) -> Result<u32, RunError> {
    execute_command_spec(
        CommandSpec {
            command,
            stdin: InputSpec::Inherit,
            stdout: OutputSpec::Inherit,
            stderr: OutputSpec::Inherit,
        },
        options,
    )
}

pub(super) fn execute_plan(
    plan: ExecutionPlan,
    options: &ExecutionOptions,
) -> Result<u32, RunError> {
    match plan {
        ExecutionPlan::Command(command) => {
            execute_command_spec(expand_alias(command, options)?, options)
        }
        ExecutionPlan::Pipeline { left, right } => execute_pipeline(
            expand_alias(left, options)?,
            expand_alias(right, options)?,
            options,
        ),
    }
}

fn expand_alias(
    mut command: CommandSpec,
    options: &ExecutionOptions,
) -> Result<CommandSpec, RunError> {
    let mut seen = Vec::new();

    while let Some(name) = alias_name(&command.command).map(str::to_string) {
        let Some(alias) = options.config.aliases.get(&name) else {
            break;
        };

        if seen.iter().any(|seen_name| seen_name == &name) {
            seen.push(name);
            return Err(RunError::AliasCycle(seen));
        }

        seen.push(name);
        let original_args = std::mem::take(&mut command.command.args);
        command.command = alias.clone();
        command.command.args.extend(original_args);
    }

    Ok(command)
}

fn alias_name(command: &ParsedCommand) -> Option<&str> {
    config::os_string_to_alias_key(&command.program)
}

fn execute_command_spec(command: CommandSpec, options: &ExecutionOptions) -> Result<u32, RunError> {
    if command.command.program.is_empty() {
        return Err(RunError::EmptyProgram);
    }

    let prepared_environment = prepare_environment(&options.environment)?;
    let resolved = resolve_command(
        &command.command,
        options.cwd.as_deref(),
        &prepared_environment,
    )?;
    let launch = prepare_launch(
        &resolved,
        options.cwd.as_deref(),
        prepared_environment.block,
    )?;
    let mut command_line = command_line::build_command_line(&resolved.argv0, &resolved.args)?;
    let stdio = win32::stdio_handles()?;
    let stdin_file = open_stdin_file(&command.stdin, options.cwd.as_deref())?;
    let stdout_file = open_stdout_file(&command.stdout, options.cwd.as_deref())?;
    let stderr_file = open_stderr_file(&command.stderr, options.cwd.as_deref())?;
    let child_stdio = win32::StdioHandles {
        stdin: stdin_file
            .as_ref()
            .map_or(stdio.stdin, |file| file.as_raw_handle()),
        stdout: stdout_file
            .as_ref()
            .map_or(stdio.stdout, |file| file.as_raw_handle()),
        stderr: stderr_file
            .as_ref()
            .map_or(stdio.stderr, |file| file.as_raw_handle()),
    };

    let child = win32::spawn_child(&mut command_line, child_stdio, launch.as_config())?;
    ForegroundTask::new(vec![child], options.timeout_ms).wait()
}

fn execute_pipeline(
    left: CommandSpec,
    right: CommandSpec,
    options: &ExecutionOptions,
) -> Result<u32, RunError> {
    if left.stdout != OutputSpec::Inherit {
        return Err(RunError::InvalidExecutionPlan(
            "left pipeline command cannot redirect stdout",
        ));
    }

    if right.stdin != InputSpec::Inherit {
        return Err(RunError::InvalidExecutionPlan(
            "right pipeline command cannot redirect stdin",
        ));
    }

    if left.command.program.is_empty() || right.command.program.is_empty() {
        return Err(RunError::EmptyProgram);
    }

    let prepared_environment = prepare_environment(&options.environment)?;
    let left_resolved =
        resolve_command(&left.command, options.cwd.as_deref(), &prepared_environment)?;
    let right_resolved = resolve_command(
        &right.command,
        options.cwd.as_deref(),
        &prepared_environment,
    )?;
    let left_launch = prepare_launch(
        &left_resolved,
        options.cwd.as_deref(),
        prepared_environment.block.clone(),
    )?;
    let right_launch = prepare_launch(
        &right_resolved,
        options.cwd.as_deref(),
        prepared_environment.block,
    )?;

    let mut left_line =
        command_line::build_command_line(&left_resolved.argv0, &left_resolved.args)?;
    let mut right_line =
        command_line::build_command_line(&right_resolved.argv0, &right_resolved.args)?;
    let stdio = win32::stdio_handles()?;
    let left_stdin_file = open_stdin_file(&left.stdin, options.cwd.as_deref())?;
    let right_stdout_file = open_stdout_file(&right.stdout, options.cwd.as_deref())?;
    let left_stderr_file = open_stderr_file(&left.stderr, options.cwd.as_deref())?;
    let right_stderr_file = open_stderr_file(&right.stderr, options.cwd.as_deref())?;
    let (pipe_read, pipe_write) = win32::create_pipe()?;

    let left_stdio = win32::StdioHandles {
        stdin: left_stdin_file
            .as_ref()
            .map_or(stdio.stdin, |file| file.as_raw_handle()),
        stdout: pipe_write.raw(),
        stderr: left_stderr_file
            .as_ref()
            .map_or(stdio.stderr, |file| file.as_raw_handle()),
    };
    let right_stdio = win32::StdioHandles {
        stdin: pipe_read.raw(),
        stdout: right_stdout_file
            .as_ref()
            .map_or(stdio.stdout, |file| file.as_raw_handle()),
        stderr: right_stderr_file
            .as_ref()
            .map_or(stdio.stderr, |file| file.as_raw_handle()),
    };

    let left_child = win32::spawn_child(&mut left_line, left_stdio, left_launch.as_config())?;
    let right_child =
        match win32::spawn_child(&mut right_line, right_stdio, right_launch.as_config()) {
            Ok(child) => child,
            Err(err) => {
                drop(pipe_read);
                drop(pipe_write);
                let _ = left_child.terminate(CANCELLED_EXIT_CODE);
                let _ = left_child.wait();
                return Err(err);
            }
        };

    drop(pipe_read);
    drop(pipe_write);

    ForegroundTask::new(vec![left_child, right_child], options.timeout_ms).wait()
}

struct ForegroundTask {
    children: Vec<win32::ChildProcess>,
    timeout_ms: Option<u32>,
    started_at: Instant,
}

impl ForegroundTask {
    fn new(children: Vec<win32::ChildProcess>, timeout_ms: Option<u32>) -> Self {
        Self {
            children,
            timeout_ms,
            started_at: Instant::now(),
        }
    }

    fn wait(mut self) -> Result<u32, RunError> {
        match self.timeout_ms {
            Some(timeout_ms) => self.wait_with_timeout(timeout_ms),
            None => self.wait_unbounded(),
        }
    }

    fn wait_unbounded(&self) -> Result<u32, RunError> {
        let mut status = 0;
        for child in &self.children {
            status = child.wait()?;
        }

        Ok(status)
    }

    fn wait_with_timeout(&mut self, timeout_ms: u32) -> Result<u32, RunError> {
        let deadline = self.started_at + Duration::from_millis(u64::from(timeout_ms));
        let mut statuses = vec![None; self.children.len()];

        loop {
            let mut all_exited = true;
            for (index, child) in self.children.iter().enumerate() {
                if statuses[index].is_some() {
                    continue;
                }

                match child.wait_timeout(0)? {
                    Some(status) => statuses[index] = Some(status),
                    None => all_exited = false,
                }
            }

            if all_exited {
                return Ok(statuses.into_iter().flatten().last().unwrap_or(0));
            }

            let now = Instant::now();
            if now >= deadline {
                self.cancel_unfinished(&statuses)?;
                return Err(RunError::Cancelled { timeout_ms });
            }

            sleep(
                deadline
                    .saturating_duration_since(now)
                    .min(FOREGROUND_POLL_INTERVAL),
            );
        }
    }

    fn cancel_unfinished(&self, statuses: &[Option<u32>]) -> Result<(), RunError> {
        let mut first_error = None;

        for (child, status) in self.children.iter().zip(statuses) {
            if status.is_some() {
                continue;
            }

            if child.wait_timeout(0)?.is_none()
                && let Err(err) = child.terminate(CANCELLED_EXIT_CODE)
                && first_error.is_none()
            {
                first_error = Some(err);
            }
        }

        for (child, status) in self.children.iter().zip(statuses) {
            if status.is_none()
                && let Err(err) = child.wait()
                && first_error.is_none()
            {
                first_error = Some(err);
            }
        }

        match first_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl EnvironmentSpec {
    fn set(&mut self, assignment: OsString) -> Result<(), RunError> {
        let Some((name, value)) = split_environment_assignment(&assignment) else {
            return Err(RunError::InvalidEnvironmentAssignment(assignment));
        };
        validate_environment_name(&name)?;
        validate_no_nul(&value)?;
        self.changes.push(EnvironmentChange::Set { name, value });
        Ok(())
    }

    fn unset(&mut self, name: OsString) -> Result<(), RunError> {
        validate_environment_name(&name)?;
        self.changes.push(EnvironmentChange::Unset { name });
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

impl PreparedLaunch {
    fn as_config(&self) -> win32::LaunchConfig<'_> {
        win32::LaunchConfig {
            application_name: &self.application_name,
            current_directory: self.current_directory.as_deref(),
            environment: self.environment.as_deref(),
        }
    }
}

fn parse_timeout_ms(value: OsString) -> Result<u32, RunError> {
    let Some(text) = value.to_str() else {
        return Err(RunError::InvalidTimeout(value));
    };

    let parsed = text
        .parse::<u32>()
        .map_err(|_| RunError::InvalidTimeout(value.clone()))?;
    if parsed == 0 {
        return Err(RunError::InvalidTimeout(value));
    }

    Ok(parsed)
}

fn validate_cwd(path: PathBuf) -> Result<PathBuf, RunError> {
    std::fs::canonicalize(&path).map_err(|source| RunError::InvalidCwd { path, source })
}

fn prepare_environment(spec: &EnvironmentSpec) -> Result<PreparedEnvironment, RunError> {
    let mut lookup = BTreeMap::new();
    for (name, value) in std::env::vars_os() {
        lookup.insert(environment_key(&name), value);
    }

    for change in &spec.changes {
        match change {
            EnvironmentChange::Set { name, value } => {
                lookup.insert(environment_key(name), value.clone());
            }
            EnvironmentChange::Unset { name } => {
                lookup.remove(&environment_key(name));
            }
        }
    }

    let block = if spec.is_empty() {
        None
    } else {
        Some(build_environment_block(&lookup)?)
    };

    Ok(PreparedEnvironment { block, lookup })
}

fn build_environment_block(lookup: &BTreeMap<String, OsString>) -> Result<Vec<u16>, RunError> {
    let mut block = Vec::new();
    for (name, value) in lookup {
        validate_no_nul(OsStr::new(name))?;
        validate_no_nul(value)?;
        block.extend(OsStr::new(name).encode_wide());
        block.push('=' as u16);
        block.extend(value.encode_wide());
        block.push(0);
    }

    block.push(0);
    Ok(block)
}

fn resolve_command(
    command: &ParsedCommand,
    cwd: Option<&Path>,
    environment: &PreparedEnvironment,
) -> Result<ResolvedCommand, RunError> {
    command_line::reject_windows_batch_target(&command.program)?;
    let base_cwd = match cwd {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().map_err(|source| RunError::InvalidCwd {
            path: PathBuf::from("."),
            source,
        })?,
    };
    let application_name = if has_path_separator(&command.program) {
        let candidate = candidate_from_path(&base_cwd, &command.program);
        resolve_path_candidate(candidate, &command.program)?
    } else {
        let (resolved, paths) = resolve_bare_name(&command.program, environment)?;
        resolved.ok_or_else(|| RunError::ExecutableNotFound {
            program: command.program.clone(),
            searched: paths,
        })?
    };

    command_line::reject_windows_batch_target(application_name.as_os_str())?;
    Ok(ResolvedCommand {
        application_name,
        argv0: command.program.clone(),
        args: command.args.clone(),
    })
}

fn resolve_path_candidate(candidate: PathBuf, program: &OsStr) -> Result<PathBuf, RunError> {
    let searched = candidate_variants(candidate);
    for path in &searched {
        if is_file(path) {
            return Ok(path.clone());
        }
    }

    Err(RunError::ExecutableNotFound {
        program: program.to_os_string(),
        searched,
    })
}

fn resolve_bare_name(
    program: &OsStr,
    environment: &PreparedEnvironment,
) -> Result<(Option<PathBuf>, Vec<PathBuf>), RunError> {
    let mut searched = Vec::new();
    let Some(path_value) = environment.lookup.get("PATH") else {
        return Ok((None, searched));
    };

    for directory in std::env::split_paths(path_value) {
        let base = directory.join(program);
        for candidate in candidate_variants(base) {
            searched.push(candidate.clone());
            if is_file(&candidate) {
                return Ok((Some(candidate), searched));
            }
        }
    }

    Ok((None, searched))
}

fn candidate_variants(path: PathBuf) -> Vec<PathBuf> {
    if path.extension().is_some() {
        vec![path]
    } else {
        let mut exe = path.clone();
        exe.set_extension("exe");
        vec![path, exe]
    }
}

fn candidate_from_path(cwd: &Path, program: &OsStr) -> PathBuf {
    let path = PathBuf::from(program);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn prepare_launch(
    command: &ResolvedCommand,
    cwd: Option<&Path>,
    environment: Option<Vec<u16>>,
) -> Result<PreparedLaunch, RunError> {
    Ok(PreparedLaunch {
        application_name: wide_null(command.application_name.as_os_str())?,
        current_directory: cwd.map(|path| wide_null(path.as_os_str())).transpose()?,
        environment,
    })
}

fn open_stdin_file(spec: &InputSpec, cwd: Option<&Path>) -> Result<Option<File>, RunError> {
    match spec {
        InputSpec::Inherit => Ok(None),
        InputSpec::File(path) => open_file_for_read(&resolve_io_path(path, cwd)).map(Some),
    }
}

fn open_stdout_file(spec: &OutputSpec, cwd: Option<&Path>) -> Result<Option<File>, RunError> {
    match spec {
        OutputSpec::Inherit => Ok(None),
        OutputSpec::File { path, append } => {
            open_file_for_write(&resolve_io_path(path, cwd), *append).map(Some)
        }
    }
}

fn open_stderr_file(spec: &OutputSpec, cwd: Option<&Path>) -> Result<Option<File>, RunError> {
    match spec {
        OutputSpec::Inherit => Ok(None),
        OutputSpec::File { path, append } => {
            open_file_for_write_with_context("open stderr", &resolve_io_path(path, cwd), *append)
                .map(Some)
        }
    }
}

fn resolve_io_path(path: &Path, cwd: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(cwd) = cwd {
        cwd.join(path)
    } else {
        path.to_path_buf()
    }
}

fn open_file_for_read(path: &Path) -> Result<File, RunError> {
    File::open(path).map_err(|source| RunError::Io {
        context: "open stdin",
        path: path.to_path_buf(),
        source,
    })
}

fn open_file_for_write(path: &Path, append: bool) -> Result<File, RunError> {
    open_file_for_write_with_context("open stdout", path, append)
}

fn open_file_for_write_with_context(
    context: &'static str,
    path: &Path,
    append: bool,
) -> Result<File, RunError> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }

    options.open(path).map_err(|source| RunError::Io {
        context,
        path: path.to_path_buf(),
        source,
    })
}

fn wide_null(value: &OsStr) -> Result<Vec<u16>, RunError> {
    let encoded: Vec<u16> = value.encode_wide().collect();
    if encoded.contains(&0) {
        return Err(RunError::InteriorNul);
    }

    let mut output = encoded;
    output.push(0);
    Ok(output)
}

fn validate_environment_name(name: &OsStr) -> Result<(), RunError> {
    validate_no_nul(name)?;
    if name.is_empty() || name.encode_wide().any(|unit| unit == '=' as u16) {
        return Err(RunError::InvalidEnvironmentName(name.to_os_string()));
    }

    Ok(())
}

fn validate_no_nul(value: &OsStr) -> Result<(), RunError> {
    if value.encode_wide().any(|unit| unit == 0) {
        Err(RunError::InteriorNul)
    } else {
        Ok(())
    }
}

fn split_environment_assignment(assignment: &OsStr) -> Option<(OsString, OsString)> {
    let encoded: Vec<u16> = assignment.encode_wide().collect();
    let split = encoded.iter().position(|unit| *unit == '=' as u16)?;
    if split == 0 {
        return None;
    }

    Some((
        OsString::from_wide(&encoded[..split]),
        OsString::from_wide(&encoded[split + 1..]),
    ))
}

fn environment_key(name: &OsStr) -> String {
    name.to_string_lossy().to_uppercase()
}

fn has_path_separator(program: &OsStr) -> bool {
    program
        .encode_wide()
        .any(|unit| unit == '\\' as u16 || unit == '/' as u16 || unit == ':' as u16)
}

fn is_file(path: &Path) -> bool {
    path.metadata().is_ok_and(|metadata| metadata.is_file())
}
