mod command_line;
mod handles;
mod parser;
mod win32;

use std::ffi::OsString;

use parser::{ParseError, parse_command_line};
pub use win32::run_child;

#[derive(Debug)]
pub enum RunError {
    Usage,
    Parse(ParseError),
    NonUnicodeCommandLine,
    EmptyProgram,
    InteriorNul,
    UnsupportedBatchTarget,
    InvalidHandle(&'static str),
    Win32 { context: &'static str, code: u32 },
    WaitFailed(u32),
    UnexpectedWait(u32),
    ExitCodeUnavailable(u32),
}

impl RunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::Parse(_) => 2,
            Self::NonUnicodeCommandLine => 2,
            Self::EmptyProgram
            | Self::InteriorNul
            | Self::UnsupportedBatchTarget
            | Self::InvalidHandle(_)
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
                "usage: run.exe <program> <args...>\n       run.exe --line <command-line>"
            ),
            Self::Parse(err) => write!(f, "{err}"),
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
    let (program, child_args) = if first == "--line" {
        let input = args.next().ok_or(RunError::Usage)?;
        if args.next().is_some() {
            return Err(RunError::Usage);
        }

        let input = input
            .into_string()
            .map_err(|_| RunError::NonUnicodeCommandLine)?;
        let parsed = parse_command_line(&input).map_err(RunError::Parse)?;
        (parsed.program, parsed.args)
    } else {
        (first, args.collect::<Vec<OsString>>())
    };

    if program.is_empty() {
        return Err(RunError::EmptyProgram);
    }

    command_line::reject_windows_batch_target(&program)?;
    let command_line = command_line::build_command_line(&program, &child_args)?;
    run_child(command_line)
}
