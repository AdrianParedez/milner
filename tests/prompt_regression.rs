#![cfg(windows)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prompt_displays_prompt_and_exits_on_eof() {
    let output = run_prompt("");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "milner> ");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_treats_empty_lines_as_no_ops() {
    let output = run_prompt("\n");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "milner> milner> ");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_runs_cargo_version() {
    let output = run_prompt("cargo --version\n");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("cargo"));
}

#[test]
fn prompt_returns_last_child_exit_code_on_eof() {
    let output = run_prompt("powershell -NoProfile -Command \"exit 7\"\n");

    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn prompt_exit_builtin_uses_requested_code() {
    let output = run_prompt("exit 7\n");

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "milner> ");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_pwd_prints_shell_current_directory() {
    let temp = temp_dir("pwd");
    let output = run_prompt(&format!("cd \"{}\"\npwd\nexit 0\n", temp.display()));

    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains(&normalize_existing_path(&temp))
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_cd_affects_next_external_command() {
    let temp = temp_dir("cd-affects-child");
    let marker = temp.join("cwd_marker.txt");
    let input = format!(
        "cd \"{}\"\npowershell -NoProfile -Command \"[System.IO.File]::WriteAllText('cwd_marker.txt',[System.IO.Directory]::GetCurrentDirectory())\"\nexit 0\n",
        temp.display()
    );
    let output = run_prompt(&input);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        normalize_existing_path(Path::new(&fs::read_to_string(marker).unwrap())),
        normalize_existing_path(&temp)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_reports_builtin_argument_errors() {
    let output = run_prompt("cd\npwd extra\nexit nope\nexit 0\n");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("cd: missing operand"));
    assert!(stderr.contains("pwd: too many operands"));
    assert!(stderr.contains("exit: invalid code `nope`"));
}

#[test]
fn prompt_rejects_stderr_redirection_for_builtins() {
    let temp = temp_dir("builtin-stderr");
    let error = temp.join("err.txt");
    let output = run_prompt(&format!("pwd 2> \"{}\"\nexit 0\n", error.display()));
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("pwd: redirection is not supported for built-ins"));
    assert!(!error.exists());
}

#[test]
fn prompt_reports_invalid_cd_paths() {
    let output = run_prompt("cd X:\\keel\\definitely-missing-directory\nexit 0\n");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cd: cannot change to"));
}

#[test]
fn prompt_recovers_from_parser_errors() {
    let output = run_prompt("tool &&\npowershell -NoProfile -Command \"exit 0\"\n");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported operator `&&`"));
}

#[test]
fn prompt_rejects_batch_targets() {
    let temp = temp_dir("prompt-batch-reject");
    let script = temp.join("echo_args.cmd");
    let marker = temp.join("prompt_batch_marker.txt");
    fs::write(
        &script,
        format!(
            "@echo off\r\necho SHOULD_NOT_EXIST>{}\r\n",
            marker.display()
        ),
    )
    .unwrap();

    let output = run_prompt(&format!("\"{}\"\n", script.display()));

    assert_eq!(output.status.code(), Some(125));
    assert!(!marker.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("batch targets are not supported"));
}

fn run_prompt(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_milner"))
        .arg("--no-config")
        .arg("--prompt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "milner-prompt-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn strip_extended_prefix(path: &str) -> &str {
    path.strip_prefix("\\\\?\\").unwrap_or(path)
}

fn normalize_existing_path(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap();
    strip_extended_prefix(&canonical.display().to_string()).to_ascii_lowercase()
}
