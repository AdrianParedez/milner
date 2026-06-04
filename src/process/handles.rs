use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE, WIN32_ERROR,
};

use super::RunError;

#[derive(Debug)]
pub struct OwnedHandle {
    handle: HANDLE,
}

impl OwnedHandle {
    pub fn new(handle: HANDLE, context: &'static str) -> Result<Self, RunError> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(RunError::InvalidHandle(context));
        }

        Ok(Self { handle })
    }

    pub fn raw(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

pub fn validate_borrowed_handle(handle: HANDLE, context: &'static str) -> Result<HANDLE, RunError> {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(RunError::InvalidHandle(context));
    }

    Ok(handle)
}

pub fn last_error() -> WIN32_ERROR {
    unsafe { GetLastError() }
}
