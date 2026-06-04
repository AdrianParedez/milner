#![cfg(windows)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prompt_starts_without_config() {
    let output = run_prompt(&["--no-config"], "");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "milner> ");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn invalid_config_reports_path_and_line() {
    let temp = temp_dir("invalid-config");
    let config = temp.join("config.toml");
    fs::write(&config, "[prompt]\nunknown = value\n").unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(125));
    assert!(stderr.contains(&config.display().to_string()));
    assert!(stderr.contains("line 2"));
    assert!(stderr.contains("unknown prompt key"));
}

#[test]
fn prompt_text_can_be_configured() {
    let temp = temp_dir("prompt-text");
    let config = temp.join("config.toml");
    fs::write(&config, "[prompt]\ntext = \"k> \"\n").unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "exit 0\n");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "k> ");
}

#[test]
fn history_can_be_disabled() {
    let temp = temp_dir("history-disabled");
    let config = temp.join("config.toml");
    let history = temp.join("history.txt");
    fs::write(
        &config,
        format!("[history]\nenabled = false\npath = {}\n", history.display()),
    )
    .unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "pwd\nexit 0\n");

    assert_eq!(output.status.code(), Some(0));
    assert!(!history.exists());
}

#[test]
fn history_skips_obvious_secrets() {
    let temp = temp_dir("history-secrets");
    let config = temp.join("config.toml");
    let history = temp.join("history.txt");
    fs::write(
        &config,
        format!("[history]\nenabled = true\npath = {}\n", history.display()),
    )
    .unwrap();

    let output = run_prompt(
        &["--config", path_text(&config).as_str()],
        "pwd\ncomplete token=abc\nexit 0\n",
    );
    let history_text = fs::read_to_string(&history).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(history_text.contains("pwd"));
    assert!(history_text.contains("exit 0"));
    assert!(!history_text.contains("token=abc"));
}

#[test]
fn completion_lists_builtins_and_aliases_without_executing_aliases() {
    let temp = temp_dir("completion");
    let config = temp.join("config.toml");
    let marker = temp.join("marker.txt");
    fs::write(
        &config,
        format!(
            "[aliases]\nboom = powershell -NoProfile -Command \"[System.IO.File]::WriteAllText('{}','bad')\"\n",
            powershell_single_quoted(&marker)
        ),
    )
    .unwrap();

    let output = run_prompt(
        &["--config", path_text(&config).as_str()],
        "complete b\nexit 0\n",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("boom"));
    assert!(!marker.exists());
}

#[test]
fn aliases_expand_through_typed_commands() {
    let temp = temp_dir("alias-expand");
    let config = temp.join("config.toml");
    fs::write(
        &config,
        "[aliases]\nfail = powershell -NoProfile -Command \"exit 7\"\n",
    )
    .unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "fail\n");

    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn aliases_cannot_include_stderr_redirection() {
    let temp = temp_dir("alias-stderr");
    let config = temp.join("config.toml");
    let error = temp.join("err.txt");
    fs::write(
        &config,
        format!(
            "[aliases]\nbad = powershell -NoProfile -Command \"exit 0\" 2> {}\n",
            error.display()
        ),
    )
    .unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(125));
    assert!(stderr.contains("alias values must not include redirection"));
}

#[test]
fn alias_cycles_are_reported_without_exiting_prompt() {
    let temp = temp_dir("alias-cycle");
    let config = temp.join("config.toml");
    fs::write(&config, "[aliases]\na = b\nb = a\n").unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "a\nexit 0\n");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("alias cycle detected: a -> b -> a"));
}

#[test]
fn aliases_cannot_bypass_batch_rejection() {
    let temp = temp_dir("alias-batch");
    let config = temp.join("config.toml");
    let script = temp.join("bad.cmd");
    let marker = temp.join("marker.txt");
    fs::write(
        &script,
        format!(
            "@echo off\r\necho SHOULD_NOT_EXIST>{}\r\n",
            marker.display()
        ),
    )
    .unwrap();
    fs::write(
        &config,
        format!("[aliases]\nbad = \"{}\"\n", script.display()),
    )
    .unwrap();

    let output = run_prompt(&["--config", path_text(&config).as_str()], "bad\n");

    assert_eq!(output.status.code(), Some(125));
    assert!(!marker.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("batch targets are not supported"));
}

fn run_prompt(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_milner"))
        .args(args)
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

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

fn powershell_single_quoted(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "milner-config-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
