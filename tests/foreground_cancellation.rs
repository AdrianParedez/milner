#![cfg(windows)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn timeout_cancels_foreground_child() {
    let temp = temp_dir("direct-timeout");
    let marker = temp.join("late-marker.txt");
    let script = delayed_marker_script(&marker);

    let output = Command::new(env!("CARGO_BIN_EXE_milner"))
        .args([
            "--no-config",
            "--timeout-ms",
            "200",
            "powershell",
            "-NoProfile",
            "-Command",
        ])
        .arg(script)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(130));
    assert!(!marker.exists());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("foreground command cancelled after 200 ms")
    );
}

#[test]
fn prompt_returns_after_cancelled_foreground_child() {
    let temp = temp_dir("prompt-timeout");
    let marker = temp.join("late-marker.txt");
    let input = format!(
        "powershell -NoProfile -Command \"{}\"\nexit 0\n",
        delayed_marker_script(&marker)
    );

    let output = run_prompt_with_args(&["--timeout-ms", "200"], &input);

    assert_eq!(output.status.code(), Some(0));
    assert!(!marker.exists());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("foreground command cancelled after 200 ms")
    );
}

#[test]
fn prompt_recovers_after_failed_foreground_launch() {
    let output = run_prompt_with_args(
        &[],
        "definitely-missing-milner-executable\npowershell -NoProfile -Command \"exit 0\"\nexit\n",
    );

    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("executable `definitely-missing-milner-executable` not found")
    );
}

fn run_prompt_with_args(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_milner"))
        .arg("--no-config")
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

fn delayed_marker_script(marker: &Path) -> String {
    format!(
        "[System.Threading.Thread]::Sleep(5000); [System.IO.File]::WriteAllText('{}','late')",
        powershell_single_quoted(marker)
    )
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
        "milner-foreground-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
