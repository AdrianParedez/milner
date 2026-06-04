#![cfg(windows)]

use std::fs;
use std::path::PathBuf;
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn stdout_redirection_writes_new_file() {
    let temp = temp_dir("stdout-new");
    let output = temp.join("out.txt");

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"[Console]::Out.Write('alpha')\" > \"{}\"",
        output.display()
    ));

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(fs::read_to_string(output).unwrap(), "alpha");
}

#[test]
fn stdout_redirection_truncates_existing_file() {
    let temp = temp_dir("stdout-truncate");
    let output = temp.join("out.txt");
    fs::write(&output, "before").unwrap();

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"[Console]::Out.Write('after')\" > \"{}\"",
        output.display()
    ));

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(fs::read_to_string(output).unwrap(), "after");
}

#[test]
fn stdout_redirection_appends_existing_file() {
    let temp = temp_dir("stdout-append");
    let output = temp.join("out.txt");
    fs::write(&output, "before").unwrap();

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"[Console]::Out.Write(' after')\" >> \"{}\"",
        output.display()
    ));

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(fs::read_to_string(output).unwrap(), "before after");
}

#[test]
fn stdin_redirection_reads_from_file() {
    let temp = temp_dir("stdin");
    let input = temp.join("input.txt");
    let output = temp.join("out.txt");
    fs::write(&input, "from-file").unwrap();

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"$input | ForEach-Object {{ [Console]::Out.Write($_) }}\" < \"{}\" > \"{}\"",
        input.display(),
        output.display()
    ));

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(fs::read_to_string(output).unwrap(), "from-file");
}

#[test]
fn two_command_pipeline_transfers_bytes_and_delivers_eof() {
    let temp = temp_dir("pipeline");
    let output = temp.join("out.txt");

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"[Console]::Out.Write('hello')\" | powershell -NoProfile -Command \"$input | ForEach-Object {{ [Console]::Out.Write($_.ToUpperInvariant()) }}; [Console]::Out.Write(':done')\" > \"{}\"",
        output.display()
    ));

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(fs::read_to_string(output).unwrap(), "HELLO:done");
}

#[test]
fn missing_redirection_file_prevents_launch() {
    let temp = temp_dir("missing-input");
    let missing = temp.join("missing.txt");
    let marker = temp.join("marker.txt");

    let result = run_line(&format!(
        "powershell -NoProfile -Command \"[System.IO.File]::WriteAllText('{}','launched')\" < \"{}\"",
        marker.display(),
        missing.display()
    ));

    assert_eq!(result.status.code(), Some(125));
    assert!(!marker.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("open stdin"));
}

fn run_line(input: &str) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_run"))
        .arg("--line")
        .arg(input)
        .output()
        .unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "keel-redirection-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
