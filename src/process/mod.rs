mod command_line;
mod handles;
mod parser;
mod prompt;
mod win32;

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};

use parser::{
    CommandSpec, ExecutionPlan, InputSpec, OutputSpec, ParseError, ParsedCommand,
    parse_execution_line,
};

#[derive(Debug)]
pub enum RunError {
    Usage,
    Parse(ParseError),
    InvalidExecutionPlan(&'static str),
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
}

impl RunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::Parse(_) => 2,
            Self::InvalidExecutionPlan(_) => 2,
            Self::NonUnicodeCommandLine => 2,
            Self::EmptyProgram
            | Self::InteriorNul
            | Self::UnsupportedBatchTarget
            | Self::InvalidHandle(_)
            | Self::Io { .. }
            | Self::Win32 { .. } => 125,
            Self::WaitFailed(_) | Self::UnexpectedWait(_) | Self::ExitCodeUnavailable(_) => 126,
        }
    }
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => write!(
                f,
                "usage: run.exe <program> <args...>\n       run.exe --line <command-line>\n       run.exe --prompt"
            ),
            Self::Parse(err) => write!(f, "{err}"),
            Self::InvalidExecutionPlan(message) => write!(f, "{message}"),
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
        }
    }
}

impl std::error::Error for RunError {}

pub fn run_from_env() -> Result<u32, RunError> {
    let mut args = std::env::args_os();
    let _runner = args.next();
    let first = args.next().ok_or(RunError::Usage)?;
    if first == "--prompt" {
        if args.next().is_some() {
            return Err(RunError::Usage);
        }

        return Ok(prompt::run_prompt() as u32);
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
        return execute_plan(plan);
    } else {
        ParsedCommand {
            program: first,
            args: args.collect::<Vec<OsString>>(),
        }
    };

    execute_command(command)
}

fn execute_command(command: ParsedCommand) -> Result<u32, RunError> {
    execute_command_spec(CommandSpec {
        command,
        stdin: InputSpec::Inherit,
        stdout: OutputSpec::Inherit,
    })
}

fn execute_plan(plan: ExecutionPlan) -> Result<u32, RunError> {
    match plan {
        ExecutionPlan::Command(command) => execute_command_spec(command),
        ExecutionPlan::Pipeline { left, right } => execute_pipeline(left, right),
    }
}

fn execute_command_spec(command: CommandSpec) -> Result<u32, RunError> {
    if command.command.program.is_empty() {
        return Err(RunError::EmptyProgram);
    }

    command_line::reject_windows_batch_target(&command.command.program)?;
    let mut command_line =
        command_line::build_command_line(&command.command.program, &command.command.args)?;
    let stdio = win32::stdio_handles()?;
    let stdin_file = open_stdin_file(&command.stdin)?;
    let stdout_file = open_stdout_file(&command.stdout)?;
    let child_stdio = win32::StdioHandles {
        stdin: stdin_file
            .as_ref()
            .map_or(stdio.stdin, |file| file.as_raw_handle()),
        stdout: stdout_file
            .as_ref()
            .map_or(stdio.stdout, |file| file.as_raw_handle()),
        stderr: stdio.stderr,
    };

    win32::run_child_with_stdio(&mut command_line, child_stdio)
}

fn execute_pipeline(left: CommandSpec, right: CommandSpec) -> Result<u32, RunError> {
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

    command_line::reject_windows_batch_target(&left.command.program)?;
    command_line::reject_windows_batch_target(&right.command.program)?;

    let mut left_line =
        command_line::build_command_line(&left.command.program, &left.command.args)?;
    let mut right_line =
        command_line::build_command_line(&right.command.program, &right.command.args)?;
    let stdio = win32::stdio_handles()?;
    let left_stdin_file = open_stdin_file(&left.stdin)?;
    let right_stdout_file = open_stdout_file(&right.stdout)?;
    let (pipe_read, pipe_write) = win32::create_pipe()?;

    let left_stdio = win32::StdioHandles {
        stdin: left_stdin_file
            .as_ref()
            .map_or(stdio.stdin, |file| file.as_raw_handle()),
        stdout: pipe_write.raw(),
        stderr: stdio.stderr,
    };
    let right_stdio = win32::StdioHandles {
        stdin: pipe_read.raw(),
        stdout: right_stdout_file
            .as_ref()
            .map_or(stdio.stdout, |file| file.as_raw_handle()),
        stderr: stdio.stderr,
    };

    let left_child = win32::spawn_child(&mut left_line, left_stdio)?;
    let right_child = match win32::spawn_child(&mut right_line, right_stdio) {
        Ok(child) => child,
        Err(err) => {
            drop(pipe_read);
            drop(pipe_write);
            let _ = left_child.wait();
            return Err(err);
        }
    };

    drop(pipe_read);
    drop(pipe_write);

    let left_result = left_child.wait();
    let right_result = right_child.wait();
    left_result?;
    right_result
}

fn open_stdin_file(spec: &InputSpec) -> Result<Option<File>, RunError> {
    match spec {
        InputSpec::Inherit => Ok(None),
        InputSpec::File(path) => open_file_for_read(path).map(Some),
    }
}

fn open_stdout_file(spec: &OutputSpec) -> Result<Option<File>, RunError> {
    match spec {
        OutputSpec::Inherit => Ok(None),
        OutputSpec::File { path, append } => open_file_for_write(path, *append).map(Some),
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
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }

    options.open(path).map_err(|source| RunError::Io {
        context: "open stdout",
        path: path.to_path_buf(),
        source,
    })
}
