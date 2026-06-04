#![cfg(windows)]

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::time::{SystemTime, UNIX_EPOCH};

use windows_sys::Win32::Foundation::{
    CloseHandle, DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, GetExitCodeProcess, INFINITE, PROCESS_INFORMATION,
    STARTUPINFOW, WaitForSingleObject,
};

#[test]
fn rejects_batch_targets_before_cmd_can_reinterpret_arguments() {
    let temp = temp_dir("batch-reject");
    let script = temp.join("echo_args.cmd");
    let marker = temp.join("batch_marker.txt");
    fs::write(&script, "@echo off\r\necho ARG1=%1\r\necho ARG2=%2\r\n").unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_run"))
        .arg(&script)
        .arg(format!(
            "SAFE\\\"&echo SHOULD_NOT_EXIST>{}&rem",
            marker.display()
        ))
        .arg("tail")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(125));
    assert!(!marker.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("batch targets are not supported"));
}

#[test]
fn child_process_does_not_receive_unrelated_inheritable_handles() {
    let temp = temp_dir("handle-list");
    let marker = temp.join("leaked_handle.txt");
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&marker)
        .unwrap();
    let inherited = duplicate_inheritable(file.as_raw_handle() as HANDLE);

    let script = format!(
        "try {{ $h=[IntPtr]::new({}); $sfh=[Microsoft.Win32.SafeHandles.SafeFileHandle]::new($h,$false); $fs=[System.IO.FileStream]::new($sfh,[System.IO.FileAccess]::Write); $b=[System.Text.Encoding]::UTF8.GetBytes('LEAKED'); $fs.Write($b,0,$b.Length); $fs.Flush(); exit 7 }} catch {{ exit 0 }}",
        inherited as usize
    );
    let status = spawn_run_with_inheritance(
        OsStr::new(env!("CARGO_BIN_EXE_run")),
        &[
            OsString::from("powershell"),
            OsString::from("-NoProfile"),
            OsString::from("-Command"),
            OsString::from(script),
        ],
    );

    unsafe {
        CloseHandle(inherited);
    }
    drop(file);

    assert_eq!(status, 0);
    assert_eq!(fs::read_to_string(&marker).unwrap(), "");
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "keel-security-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn duplicate_inheritable(handle: HANDLE) -> HANDLE {
    let process = unsafe { GetCurrentProcess() };
    let mut duplicate = null_mut();
    let duplicated = unsafe {
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
    assert_ne!(duplicated, 0, "DuplicateHandle failed");
    duplicate
}

fn spawn_run_with_inheritance(program: &OsStr, args: &[OsString]) -> u32 {
    let mut command_line = build_command_line(program, args);
    let startup_info = STARTUPINFOW {
        cb: size_of_startup_info(),
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
    assert_ne!(created, 0, "CreateProcessW failed");

    let wait = unsafe { WaitForSingleObject(process_info.hProcess, INFINITE) };
    assert_eq!(wait, WAIT_OBJECT_0);

    let mut exit_code = 0;
    let got_exit = unsafe { GetExitCodeProcess(process_info.hProcess, &mut exit_code) };
    assert_ne!(got_exit, 0, "GetExitCodeProcess failed");

    unsafe {
        CloseHandle(process_info.hThread);
        CloseHandle(process_info.hProcess);
    }

    exit_code
}

fn size_of_startup_info() -> u32 {
    std::mem::size_of::<STARTUPINFOW>() as u32
}

fn build_command_line(program: &OsStr, args: &[OsString]) -> Vec<u16> {
    let mut output = Vec::new();
    append_quoted_arg(&mut output, program);

    for arg in args {
        output.push(' ' as u16);
        append_quoted_arg(&mut output, arg);
    }

    output.push(0);
    output
}

fn append_quoted_arg(output: &mut Vec<u16>, arg: &OsStr) {
    output.push('"' as u16);
    let mut backslashes = 0usize;

    for unit in arg.encode_wide() {
        if unit == '\\' as u16 {
            backslashes += 1;
            continue;
        }

        if unit == '"' as u16 {
            output.extend(std::iter::repeat_n('\\' as u16, backslashes * 2 + 1));
            output.push(unit);
            backslashes = 0;
            continue;
        }

        output.extend(std::iter::repeat_n('\\' as u16, backslashes));
        backslashes = 0;
        output.push(unit);
    }

    output.extend(std::iter::repeat_n('\\' as u16, backslashes * 2));
    output.push('"' as u16);
}
