use std::ffi::c_void;
use std::mem::{size_of, size_of_val};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess,
    GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOEXW,
    WaitForSingleObject,
};

use super::RunError;
use super::handles::{OwnedHandle, last_error, validate_borrowed_handle};

pub fn run_child(mut command_line: Vec<u16>) -> Result<u32, RunError> {
    let stdin = get_std_handle(STD_INPUT_HANDLE, "GetStdHandle(STD_INPUT_HANDLE)")?;
    let stdout = get_std_handle(STD_OUTPUT_HANDLE, "GetStdHandle(STD_OUTPUT_HANDLE)")?;
    let stderr = get_std_handle(STD_ERROR_HANDLE, "GetStdHandle(STD_ERROR_HANDLE)")?;

    let child_stdin = duplicate_inheritable(stdin, "DuplicateHandle(stdin)")?;
    let child_stdout = duplicate_inheritable(stdout, "DuplicateHandle(stdout)")?;
    let child_stderr = duplicate_inheritable(stderr, "DuplicateHandle(stderr)")?;
    let inherited_handles = [child_stdin.raw(), child_stdout.raw(), child_stderr.raw()];
    let attribute_list = StartupAttributeList::new(&inherited_handles)?;

    let mut startup_info = STARTUPINFOEXW {
        StartupInfo: windows_sys::Win32::System::Threading::STARTUPINFOW {
            cb: size_of::<STARTUPINFOEXW>() as u32,
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: child_stdin.raw(),
            hStdOutput: child_stdout.raw(),
            hStdError: child_stderr.raw(),
            ..windows_sys::Win32::System::Threading::STARTUPINFOW::default()
        },
        lpAttributeList: attribute_list.raw(),
    };
    let mut process_info = PROCESS_INFORMATION::default();

    let created = unsafe {
        CreateProcessW(
            null(),
            command_line.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT,
            null(),
            null(),
            &mut startup_info as *mut STARTUPINFOEXW as *const _,
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

fn duplicate_inheritable(handle: HANDLE, context: &'static str) -> Result<OwnedHandle, RunError> {
    let process = unsafe { GetCurrentProcess() };
    let mut duplicate = null_mut();
    let result = unsafe {
        DuplicateHandle(
            process,
            handle,
            process,
            &mut duplicate,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    };

    if result == 0 {
        return Err(RunError::Win32 {
            context,
            code: last_error(),
        });
    }

    OwnedHandle::new(duplicate, context)
}

struct StartupAttributeList {
    _storage: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
}

impl StartupAttributeList {
    fn new(handles: &[HANDLE]) -> Result<Self, RunError> {
        let mut size = 0usize;
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size);
        }

        let units = size.div_ceil(size_of::<usize>());
        let mut storage = vec![0usize; units];
        let list = storage.as_mut_ptr().cast::<c_void>();
        let initialized = unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut size) };

        if initialized == 0 {
            return Err(RunError::Win32 {
                context: "InitializeProcThreadAttributeList",
                code: last_error(),
            });
        }

        let updated = unsafe {
            windows_sys::Win32::System::Threading::UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast::<c_void>(),
                size_of_val(handles),
                null_mut(),
                null(),
            )
        };

        if updated == 0 {
            unsafe {
                DeleteProcThreadAttributeList(list);
            }
            return Err(RunError::Win32 {
                context: "UpdateProcThreadAttribute(PROC_THREAD_ATTRIBUTE_HANDLE_LIST)",
                code: last_error(),
            });
        }

        Ok(Self {
            _storage: storage,
            list,
        })
    }

    fn raw(&self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.list
    }
}

impl Drop for StartupAttributeList {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.list);
        }
    }
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
