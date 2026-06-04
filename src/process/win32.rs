use std::ffi::c_void;
use std::mem::{size_of, size_of_val};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess, GetExitCodeProcess, INFINITE,
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, ResumeThread, STARTF_USESTDHANDLES,
    STARTUPINFOEXW, TerminateProcess, WaitForSingleObject,
};

use super::RunError;
use super::handles::{OwnedHandle, last_error, validate_borrowed_handle};

#[derive(Clone, Copy)]
pub struct StdioHandles {
    pub stdin: HANDLE,
    pub stdout: HANDLE,
    pub stderr: HANDLE,
}

pub struct ChildProcess {
    process: OwnedHandle,
    _thread: OwnedHandle,
    job: JobObject,
}

#[derive(Clone, Copy)]
pub struct LaunchConfig<'a> {
    pub application_name: &'a [u16],
    pub current_directory: Option<&'a [u16]>,
    pub environment: Option<&'a [u16]>,
}

pub fn spawn_child(
    command_line: &mut Vec<u16>,
    stdio: StdioHandles,
    launch: LaunchConfig<'_>,
) -> Result<ChildProcess, RunError> {
    let child_stdin = duplicate_inheritable(stdio.stdin, "DuplicateHandle(stdin)")?;
    let child_stdout = duplicate_inheritable(stdio.stdout, "DuplicateHandle(stdout)")?;
    let child_stderr = duplicate_inheritable(stdio.stderr, "DuplicateHandle(stderr)")?;
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

    let creation_flags = EXTENDED_STARTUPINFO_PRESENT
        | CREATE_SUSPENDED
        | if launch.environment.is_some() {
            CREATE_UNICODE_ENVIRONMENT
        } else {
            0
        };
    let environment = launch
        .environment
        .map_or(null(), |block| block.as_ptr().cast());
    let current_directory = launch
        .current_directory
        .map_or(null(), |directory| directory.as_ptr());

    let created = unsafe {
        CreateProcessW(
            launch.application_name.as_ptr(),
            command_line.as_mut_ptr(),
            null(),
            null(),
            1,
            creation_flags,
            environment,
            current_directory,
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
    let thread = OwnedHandle::new(process_info.hThread, "CreateProcessW(hThread)")?;
    let job = JobObject::new()?;
    if let Err(err) = job.assign_process(process.raw()) {
        let _ = terminate_process_for_cleanup(process.raw());
        let _ = wait_for_process(process.raw());
        return Err(err);
    }

    if let Err(err) = resume_thread(thread.raw()) {
        let _ = job.terminate(1);
        let _ = wait_for_process(process.raw());
        return Err(err);
    }

    Ok(ChildProcess {
        process,
        _thread: thread,
        job,
    })
}

pub fn stdio_handles() -> Result<StdioHandles, RunError> {
    let stdin = get_std_handle(STD_INPUT_HANDLE, "GetStdHandle(STD_INPUT_HANDLE)")?;
    let stdout = get_std_handle(STD_OUTPUT_HANDLE, "GetStdHandle(STD_OUTPUT_HANDLE)")?;
    let stderr = get_std_handle(STD_ERROR_HANDLE, "GetStdHandle(STD_ERROR_HANDLE)")?;

    Ok(StdioHandles {
        stdin,
        stdout,
        stderr,
    })
}

pub fn create_pipe() -> Result<(OwnedHandle, OwnedHandle), RunError> {
    let mut read = null_mut();
    let mut write = null_mut();
    let created = unsafe { CreatePipe(&mut read, &mut write, null(), 0) };

    if created == 0 {
        return Err(RunError::Win32 {
            context: "CreatePipe",
            code: last_error(),
        });
    }

    Ok((
        OwnedHandle::new(read, "CreatePipe(read)")?,
        OwnedHandle::new(write, "CreatePipe(write)")?,
    ))
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

impl ChildProcess {
    pub fn wait(&self) -> Result<u32, RunError> {
        wait_for_process(self.process.raw())?;
        child_exit_code(self.process.raw())
    }

    pub fn wait_timeout(&self, timeout_ms: u32) -> Result<Option<u32>, RunError> {
        match wait_for_process_timeout(self.process.raw(), timeout_ms)? {
            WaitOutcome::Exited => child_exit_code(self.process.raw()).map(Some),
            WaitOutcome::StillRunning => Ok(None),
        }
    }

    pub fn terminate(&self, exit_code: u32) -> Result<(), RunError> {
        self.job.terminate(exit_code)
    }
}

struct JobObject {
    handle: OwnedHandle,
}

impl JobObject {
    fn new() -> Result<Self, RunError> {
        let raw = unsafe { CreateJobObjectW(null(), null()) };
        let handle = OwnedHandle::new(raw, "CreateJobObjectW")?;
        let job = Self { handle };
        job.set_kill_on_close()?;
        Ok(job)
    }

    fn set_kill_on_close(&self) -> Result<(), RunError> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let set = unsafe {
            SetInformationJobObject(
                self.handle.raw(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };

        if set == 0 {
            return Err(RunError::Win32 {
                context: "SetInformationJobObject(JobObjectExtendedLimitInformation)",
                code: last_error(),
            });
        }

        Ok(())
    }

    fn assign_process(&self, process: HANDLE) -> Result<(), RunError> {
        let assigned = unsafe { AssignProcessToJobObject(self.handle.raw(), process) };
        if assigned == 0 {
            return Err(RunError::Win32 {
                context: "AssignProcessToJobObject",
                code: last_error(),
            });
        }

        Ok(())
    }

    fn terminate(&self, exit_code: u32) -> Result<(), RunError> {
        let terminated = unsafe { TerminateJobObject(self.handle.raw(), exit_code) };
        if terminated == 0 {
            return Err(RunError::Win32 {
                context: "TerminateJobObject",
                code: last_error(),
            });
        }

        Ok(())
    }
}

fn resume_thread(thread: HANDLE) -> Result<(), RunError> {
    let previous_suspend_count = unsafe { ResumeThread(thread) };
    if previous_suspend_count == u32::MAX {
        return Err(RunError::Win32 {
            context: "ResumeThread",
            code: last_error(),
        });
    }

    Ok(())
}

fn terminate_process_for_cleanup(process: HANDLE) -> Result<(), RunError> {
    let terminated = unsafe { TerminateProcess(process, 1) };
    if terminated == 0 {
        return Err(RunError::Win32 {
            context: "TerminateProcess",
            code: last_error(),
        });
    }

    Ok(())
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
    match wait_for_process_timeout(handle, INFINITE)? {
        WaitOutcome::Exited => Ok(()),
        WaitOutcome::StillRunning => Err(RunError::UnexpectedWait(WAIT_TIMEOUT)),
    }
}

enum WaitOutcome {
    Exited,
    StillRunning,
}

fn wait_for_process_timeout(handle: HANDLE, timeout_ms: u32) -> Result<WaitOutcome, RunError> {
    let status = unsafe { WaitForSingleObject(handle, timeout_ms) };
    match status {
        WAIT_OBJECT_0 => Ok(WaitOutcome::Exited),
        WAIT_TIMEOUT => Ok(WaitOutcome::StillRunning),
        WAIT_FAILED => Err(RunError::WaitFailed(last_error())),
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
