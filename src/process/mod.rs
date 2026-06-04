mod command_line;
mod handles;
mod win32;

use std::ffi::OsString;

pub use win32::run_child;

#[derive(Debug)]
pub enum RunError {
    Usage,
    EmptyProgram,
    InteriorNul,
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
            Self::EmptyProgram
            | Self::InteriorNul
            | Self::InvalidHandle(_)
            | Self::Win32 { .. } => 125,
            Self::WaitFailed(_) | Self::UnexpectedWait(_) | Self::ExitCodeUnavailable(_) => 126,
        }
    }
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => write!(f, "usage: run.exe <program> <args...>"),
            Self::EmptyProgram => write!(f, "program must not be empty"),
            Self::InteriorNul => write!(f, "arguments must not contain an interior null"),
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
    let program = args.next().ok_or(RunError::Usage)?;
    let child_args: Vec<OsString> = args.collect();

    if program.is_empty() {
        return Err(RunError::EmptyProgram);
    }

    let command_line = command_line::build_command_line(&program, &child_args)?;
    run_child(command_line)
}
