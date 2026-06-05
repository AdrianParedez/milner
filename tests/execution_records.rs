#![cfg(windows)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn successful_command_writes_execution_record() {
    let temp = temp_dir("success");
    let records = temp.join("records.ndjson");
    let config = write_records_config(&temp, &records);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "powershell",
        "-NoProfile",
        "-Command",
        "exit 7",
    ]);
    let record = read_single_record(&records);

    assert_eq!(output.status.code(), Some(7));
    assert!(record.contains("\"schema_version\":1"));
    assert!(record.contains("\"plan_kind\":\"command\""));
    assert!(record.contains("\"program\":\"powershell\""));
    assert!(record.contains("\"kind\":\"success\""));
    assert!(record.contains("\"exit_code\":7"));
    assert!(record.contains("\"resolved_executable\":"));
    assert!(record.contains("\"explicit_executable_resolution\""));
}

#[test]
fn failed_resolution_writes_error_record() {
    let temp = temp_dir("failed-resolution");
    let records = temp.join("records.ndjson");
    let config = write_records_config(&temp, &records);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "milner-definitely-missing-executable",
    ]);
    let record = read_single_record(&records);

    assert_eq!(output.status.code(), Some(125));
    assert!(record.contains("\"kind\":\"error\""));
    assert!(record.contains("\"exit_code\":125"));
    assert!(record.contains("milner-definitely-missing-executable"));
    assert!(record.contains("\"resolved_executable\":null"));
}

#[test]
fn timeout_writes_error_record() {
    let temp = temp_dir("timeout");
    let records = temp.join("records.ndjson");
    let config = write_records_config(&temp, &records);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "--timeout-ms",
        "200",
        "powershell",
        "-NoProfile",
        "-Command",
        "[System.Threading.Thread]::Sleep(5000)",
    ]);
    let record = read_single_record(&records);

    assert_eq!(output.status.code(), Some(130));
    assert!(record.contains("\"kind\":\"error\""));
    assert!(record.contains("\"exit_code\":130"));
    assert!(record.contains("foreground command cancelled after 200 ms"));
}

#[test]
fn secret_bearing_command_is_skipped() {
    let temp = temp_dir("secret-skip");
    let records = temp.join("records.ndjson");
    let config = write_records_config(&temp, &records);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "--line",
        "powershell -NoProfile -Command \"exit 0 # token=abc\"",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(!records.exists());
}

#[test]
fn persistence_failure_does_not_change_command_exit_status() {
    let temp = temp_dir("persistence-failure");
    let config = write_records_config(&temp, &temp);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "powershell",
        "-NoProfile",
        "-Command",
        "exit 0",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr.contains("execution record persistence failed"));
}

#[test]
fn pipeline_record_preserves_pipeline_shape() {
    let temp = temp_dir("pipeline");
    let records = temp.join("records.ndjson");
    let config = write_records_config(&temp, &records);

    let output = run([
        "--config",
        path_text(&config).as_str(),
        "--line",
        "powershell -NoProfile -Command \"[Console]::Out.Write('ok')\" | powershell -NoProfile -Command \"exit 0\"",
    ]);
    let record = read_single_record(&records);

    assert_eq!(output.status.code(), Some(0));
    assert!(record.contains("\"plan_kind\":\"pipeline\""));
    assert_eq!(record.matches("\"program\":\"powershell\"").count(), 2);
    assert!(record.contains("\"single_pipeline_limit\""));
    assert!(record.contains("\"kind\":\"success\""));
}

fn run<const N: usize>(args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_milner"))
        .args(args)
        .output()
        .unwrap()
}

fn write_records_config(temp: &Path, records: &Path) -> PathBuf {
    let config = temp.join("config.toml");
    fs::write(
        &config,
        format!("[records]\nenabled = true\npath = {}\n", records.display()),
    )
    .unwrap();
    config
}

fn read_single_record(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap();
    assert_eq!(text.lines().count(), 1);
    text
}

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "milner-records-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
