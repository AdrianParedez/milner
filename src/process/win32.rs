use std::mem::size_of;
use std::ptr::null;

use windows_sys::Win32::Foundation::{
    HANDLE, HANDLE_FLAG_INHERIT, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, INFINITE, PROCESS_INFORMATION, STARTF_USESTDHANDLES,
    STARTUPINFOW, WaitForSingleObject,
};

use super::RunError;
use super::handles::{OwnedHandle, last_error, validate_borrowed_handle};

pub fn run_child(mut command_line: Vec<u16>) -> Result<u32, RunError> {
    let stdin = get_std_handle(STD_INPUT_HANDLE, "GetStdHandle(STD_INPUT_HANDLE)")?;
    let stdout = get_std_handle(STD_OUTPUT_HANDLE, "GetStdHandle(STD_OUTPUT_HANDLE)")?;
    let stderr = get_std_handle(STD_ERROR_HANDLE, "GetStdHandle(STD_ERROR_HANDLE)")?;

    make_inheritable(stdin, "SetHandleInformation(stdin)")?;
    make_inheritable(stdout, "SetHandleInformation(stdout)")?;
    make_inheritable(stderr, "SetHandleInformation(stderr)")?;

    let startup_info = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESTDHANDLES,
        hStdInput: stdin,
        hStdOutput: stdout,
        hStdError: stderr,
        ..STARTUPINFOW::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();

    let created = unsafe {
        CreateProcessW(
            null(),
            command_line.as_mut_ptr(),
            null(),
            null(),
            1,
            0,
            null(),
            null(),
            &startup_info,
            &mut process_info,
        )
    };

    if created == 0 {
        return Err(RunError::Win32 {
            context: "CreateProcessW",
            code: last_error(),
        });
    }

    let process = OwnedHandle::new(process_info.hProcess, "CreateProcessW(hProcess)")?;
    let _thread = OwnedHandle::new(process_info.hThread, "CreateProcessW(hThread)")?;

    wait_for_process(process.raw())?;
    child_exit_code(process.raw())
}

fn get_std_handle(handle: u32, context: &'static str) -> Result<HANDLE, RunError> {
    let raw = unsafe { GetStdHandle(handle) };
    validate_borrowed_handle(raw, context)
}

fn make_inheritable(handle: HANDLE, context: &'static str) -> Result<(), RunError> {
    let result = unsafe {
        windows_sys::Win32::Foundation::SetHandleInformation(
            handle,
            HANDLE_FLAG_INHERIT,
            HANDLE_FLAG_INHERIT,
        )
    };

    if result == 0 {
        return Err(RunError::Win32 {
            context,
            code: last_error(),
        });
    }

    Ok(())
}

fn wait_for_process(handle: HANDLE) -> Result<(), RunError> {
    let status = unsafe { WaitForSingleObject(handle, INFINITE) };
    match status {
        WAIT_OBJECT_0 => Ok(()),
        WAIT_FAILED => Err(RunError::WaitFailed(last_error())),
        WAIT_TIMEOUT => Err(RunError::UnexpectedWait(WAIT_TIMEOUT)),
        other => Err(RunError::UnexpectedWait(other)),
    }
}

fn child_exit_code(handle: HANDLE) -> Result<u32, RunError> {
    let mut exit_code = 0u32;
    let result = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
    if result == 0 {
        return Err(RunError::ExitCodeUnavailable(last_error()));
    }

    Ok(exit_code)
}
