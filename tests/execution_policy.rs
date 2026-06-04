#![cfg(windows)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn bare_executable_names_resolve_through_path() {
    let output = run(["--line", "cargo --version"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("cargo"));
}

#[test]
fn absolute_executable_paths_launch_directly() {
    let output = run([
        "--line",
        &format!(
            "\"{}\" -NoProfile -Command \"exit 0\"",
            powershell_path().display()
        ),
    ]);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn relative_executable_paths_resolve_against_child_cwd() {
    let temp = temp_dir("relative-exe");
    fs::copy(
        env!("CARGO_BIN_EXE_run"),
        temp.join("keel-relative-probe.exe"),
    )
    .unwrap();
    let output = run([
        "--cwd",
        &temp.display().to_string(),
        "--line",
        ".\\keel-relative-probe.exe --no-config --line \"cargo --version\"",
    ]);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn child_cwd_is_set_without_parent_global_directory_change() {
    let original = std::env::current_dir().unwrap();
    let temp = temp_dir("cwd");
    let output = temp.join("cwd.txt");
    let result = run([
        "--cwd",
        &temp.display().to_string(),
        "--line",
        &format!(
            "powershell -NoProfile -Command \"[Console]::Out.Write([System.IO.Directory]::GetCurrentDirectory())\" > \"{}\"",
            output.display()
        ),
    ]);

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(
        normalize_existing_path(Path::new(&fs::read_to_string(output).unwrap())),
        normalize_existing_path(&temp)
    );
    assert_eq!(std::env::current_dir().unwrap(), original);
}

#[test]
fn child_environment_is_inherited_by_default() {
    let output = run([
        "--line",
        "powershell -NoProfile -Command \"if ($env:PATH) { exit 0 } else { exit 7 }\"",
    ]);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn child_environment_can_be_extended() {
    let output = run([
        "--set-env",
        "KEEL_POLICY_VALUE=available",
        "--line",
        "powershell -NoProfile -Command \"if ($env:KEEL_POLICY_VALUE -eq 'available') { exit 0 } else { exit 7 }\"",
    ]);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn child_environment_can_remove_child_only_values() {
    let output = run([
        "--set-env",
        "KEEL_POLICY_REMOVE=gone",
        "--unset-env",
        "KEEL_POLICY_REMOVE",
        "--line",
        "powershell -NoProfile -Command \"if ($env:KEEL_POLICY_REMOVE) { exit 7 } else { exit 0 }\"",
    ]);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn bare_names_do_not_search_child_cwd() {
    let temp = temp_dir("cwd-search");
    fs::copy(env!("CARGO_BIN_EXE_run"), temp.join("keel-local-probe.exe")).unwrap();

    let output = run([
        "--cwd",
        &temp.display().to_string(),
        "--line",
        "keel-local-probe.exe --no-config --line \"cargo --version\"",
    ]);

    assert_eq!(output.status.code(), Some(125));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("did not search the current directory")
    );
}

#[test]
fn missing_executable_reports_resolution_policy() {
    let output = run(["--line", "keel-definitely-missing-executable"]);

    assert_eq!(output.status.code(), Some(125));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("executable `keel-definitely-missing-executable` not found"));
    assert!(stderr.contains("did not search the current directory"));
}

fn run<const N: usize>(args: [&str; N]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_run"))
        .arg("--no-config")
        .args(args)
        .output()
        .unwrap()
}

fn powershell_path() -> PathBuf {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    PathBuf::from(system_root).join("System32\\WindowsPowerShell\\v1.0\\powershell.exe")
}

fn strip_extended_prefix(path: &str) -> &str {
    path.strip_prefix("\\\\?\\").unwrap_or(path)
}

fn normalize_existing_path(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap();
    strip_extended_prefix(&canonical.display().to_string()).to_ascii_lowercase()
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "keel-policy-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
