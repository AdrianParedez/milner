#![cfg(windows)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prompt_displays_prompt_and_exits_on_eof() {
    let output = run_prompt("");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "keel> ");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn prompt_treats_empty_lines_as_no_ops() {
    let output = run_prompt("\n");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "keel> keel> ");
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
fn prompt_recovers_from_parser_errors() {
    let output = run_prompt("tool |\npowershell -NoProfile -Command \"exit 0\"\n");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported operator `|`"));
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_run"))
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
        "keel-prompt-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
