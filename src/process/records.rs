use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::RunError;
use super::config::{RecordsConfig, default_records_path};
use super::parser::{CommandSpec, InputSpec, OutputSpec};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub(super) struct ExecutionRecord {
    started_unix_ms: u128,
    ended_unix_ms: Option<u128>,
    cwd: PathBuf,
    plan_kind: PlanKind,
    commands: Vec<RecordedCommand>,
    status: Option<RecordStatus>,
    policy: Vec<&'static str>,
}

#[derive(Debug)]
enum PlanKind {
    Command,
    Pipeline,
}

#[derive(Debug)]
struct RecordedCommand {
    program: String,
    args: Vec<String>,
    stdin: RecordedInput,
    stdout: RecordedOutput,
    stderr: RecordedOutput,
    resolved_executable: Option<PathBuf>,
}

#[derive(Debug)]
enum RecordedInput {
    Inherit,
    File(PathBuf),
}

#[derive(Debug)]
enum RecordedOutput {
    Inherit,
    File { path: PathBuf, append: bool },
}

#[derive(Debug)]
enum RecordStatus {
    Success { exit_code: u32 },
    Error { message: String, exit_code: i32 },
}

pub(super) fn command(spec: &CommandSpec, cwd: Option<&Path>) -> Option<ExecutionRecord> {
    if spec_contains_obvious_secret(spec) {
        return None;
    }

    Some(ExecutionRecord::new(
        PlanKind::Command,
        vec![RecordedCommand::from_spec(spec)],
        cwd,
    ))
}

pub(super) fn pipeline(
    left: &CommandSpec,
    right: &CommandSpec,
    cwd: Option<&Path>,
) -> Option<ExecutionRecord> {
    if spec_contains_obvious_secret(left) || spec_contains_obvious_secret(right) {
        return None;
    }

    Some(ExecutionRecord::new(
        PlanKind::Pipeline,
        vec![
            RecordedCommand::from_spec(left),
            RecordedCommand::from_spec(right),
        ],
        cwd,
    ))
}

pub(super) fn persist_result(
    record: Option<ExecutionRecord>,
    config: &RecordsConfig,
    result: &Result<u32, RunError>,
) {
    let Some(mut record) = record else {
        return;
    };
    if !config.enabled {
        return;
    }

    record.finish(result);
    if let Err(err) = append_record(config, &record) {
        eprintln!("milner: execution record persistence failed: {err}");
    }
}

impl ExecutionRecord {
    fn new(plan_kind: PlanKind, commands: Vec<RecordedCommand>, cwd: Option<&Path>) -> Self {
        Self {
            started_unix_ms: unix_ms_now(),
            ended_unix_ms: None,
            cwd: cwd
                .map(Path::to_path_buf)
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from(".")),
            plan_kind,
            commands,
            status: None,
            policy: vec!["no_cmd_fallback", "batch_targets_rejected"],
        }
    }

    pub(super) fn set_resolved_executable(&mut self, index: usize, path: &Path) {
        if let Some(command) = self.commands.get_mut(index) {
            command.resolved_executable = Some(path.to_path_buf());
        }
    }

    pub(super) fn add_policy(&mut self, decision: &'static str) {
        if !self.policy.contains(&decision) {
            self.policy.push(decision);
        }
    }

    fn finish(&mut self, result: &Result<u32, RunError>) {
        self.ended_unix_ms = Some(unix_ms_now());
        self.status = Some(match result {
            Ok(exit_code) => RecordStatus::Success {
                exit_code: *exit_code,
            },
            Err(err) => RecordStatus::Error {
                message: err.to_string(),
                exit_code: err.exit_code(),
            },
        });
    }

    fn to_json_line(&self) -> String {
        let mut output = String::new();
        output.push('{');
        push_field_number(
            &mut output,
            "schema_version",
            u128::from(SCHEMA_VERSION),
            false,
        );
        push_field_number(&mut output, "started_unix_ms", self.started_unix_ms, true);
        if let Some(ended) = self.ended_unix_ms {
            push_field_number(&mut output, "ended_unix_ms", ended, true);
        } else {
            push_field_null(&mut output, "ended_unix_ms", true);
        }
        push_field_string(&mut output, "cwd", &path_text(&self.cwd), true);
        push_field_string(&mut output, "plan_kind", self.plan_kind.as_str(), true);
        output.push_str(",\"commands\":[");
        for (index, command) in self.commands.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(&command.to_json());
        }
        output.push(']');
        output.push_str(",\"status\":");
        match &self.status {
            Some(status) => output.push_str(&status.to_json()),
            None => output.push_str("null"),
        }
        output.push_str(",\"policy\":[");
        for (index, decision) in self.policy.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(&mut output, decision);
        }
        output.push_str("]}");
        output
    }
}

impl RecordedCommand {
    fn from_spec(spec: &CommandSpec) -> Self {
        Self {
            program: spec.command.program.to_string_lossy().to_string(),
            args: spec
                .command
                .args
                .iter()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect(),
            stdin: RecordedInput::from(&spec.stdin),
            stdout: RecordedOutput::from(&spec.stdout),
            stderr: RecordedOutput::from(&spec.stderr),
            resolved_executable: None,
        }
    }

    fn to_json(&self) -> String {
        let mut output = String::new();
        output.push('{');
        push_field_string(&mut output, "program", &self.program, false);
        output.push_str(",\"args\":[");
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(&mut output, arg);
        }
        output.push(']');
        output.push_str(",\"stdin\":");
        output.push_str(&self.stdin.to_json());
        output.push_str(",\"stdout\":");
        output.push_str(&self.stdout.to_json());
        output.push_str(",\"stderr\":");
        output.push_str(&self.stderr.to_json());
        match &self.resolved_executable {
            Some(path) => {
                push_field_string(&mut output, "resolved_executable", &path_text(path), true)
            }
            None => push_field_null(&mut output, "resolved_executable", true),
        }
        output.push('}');
        output
    }
}

impl From<&InputSpec> for RecordedInput {
    fn from(value: &InputSpec) -> Self {
        match value {
            InputSpec::Inherit => Self::Inherit,
            InputSpec::File(path) => Self::File(path.clone()),
        }
    }
}

impl From<&OutputSpec> for RecordedOutput {
    fn from(value: &OutputSpec) -> Self {
        match value {
            OutputSpec::Inherit => Self::Inherit,
            OutputSpec::File { path, append } => Self::File {
                path: path.clone(),
                append: *append,
            },
        }
    }
}

impl RecordedInput {
    fn to_json(&self) -> String {
        match self {
            Self::Inherit => "{\"kind\":\"inherit\"}".to_string(),
            Self::File(path) => format!(
                "{{\"kind\":\"file\",\"path\":{}}}",
                json_string(&path_text(path))
            ),
        }
    }
}

impl RecordedOutput {
    fn to_json(&self) -> String {
        match self {
            Self::Inherit => "{\"kind\":\"inherit\"}".to_string(),
            Self::File { path, append } => format!(
                "{{\"kind\":\"file\",\"path\":{},\"append\":{append}}}",
                json_string(&path_text(path))
            ),
        }
    }
}

impl PlanKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Pipeline => "pipeline",
        }
    }
}

impl RecordStatus {
    fn to_json(&self) -> String {
        match self {
            Self::Success { exit_code } => {
                format!("{{\"kind\":\"success\",\"exit_code\":{exit_code}}}")
            }
            Self::Error { message, exit_code } => format!(
                "{{\"kind\":\"error\",\"exit_code\":{exit_code},\"message\":{}}}",
                json_string(message)
            ),
        }
    }
}

fn append_record(config: &RecordsConfig, record: &ExecutionRecord) -> Result<(), std::io::Error> {
    let path = match &config.path {
        Some(path) => path.clone(),
        None => default_records_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "APPDATA is required for default records path",
            )
        })?,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", record.to_json_line())
}

fn spec_contains_obvious_secret(spec: &CommandSpec) -> bool {
    contains_obvious_secret(&spec.command.program.to_string_lossy())
        || spec
            .command
            .args
            .iter()
            .any(|arg| contains_obvious_secret(&arg.to_string_lossy()))
}

fn contains_obvious_secret(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    [
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "credential",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn unix_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

fn push_field_string(output: &mut String, key: &str, value: &str, needs_comma: bool) {
    if needs_comma {
        output.push(',');
    }
    push_json_string(output, key);
    output.push(':');
    push_json_string(output, value);
}

fn push_field_number(output: &mut String, key: &str, value: u128, needs_comma: bool) {
    if needs_comma {
        output.push(',');
    }
    push_json_string(output, key);
    output.push(':');
    output.push_str(&value.to_string());
}

fn push_field_null(output: &mut String, key: &str, needs_comma: bool) {
    if needs_comma {
        output.push(',');
    }
    push_json_string(output, key);
    output.push_str(":null");
}

fn json_string(value: &str) -> String {
    let mut output = String::new();
    push_json_string(&mut output, value);
    output
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::parser::{CommandSpec, InputSpec, OutputSpec, ParsedCommand};
    use super::command;
    use crate::process::RunError;

    #[test]
    fn skips_secret_bearing_commands() {
        let spec = spec("tool", &["--token=abc"]);

        assert!(command(&spec, None).is_none());
    }

    #[test]
    fn serializes_basic_success_record() {
        let spec = spec("tool", &["arg"]);
        let mut record = command(&spec, Some(&PathBuf::from("C:\\work"))).unwrap();
        record.set_resolved_executable(0, &PathBuf::from("C:\\bin\\tool.exe"));
        record.finish(&Ok(7));

        let json = record.to_json_line();

        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"plan_kind\":\"command\""));
        assert!(json.contains("\"program\":\"tool\""));
        assert!(json.contains("\"resolved_executable\":\"C:\\\\bin\\\\tool.exe\""));
        assert!(json.contains("\"kind\":\"success\""));
        assert!(json.contains("\"exit_code\":7"));
    }

    #[test]
    fn error_record_uses_run_error_exit_code() {
        let spec = spec("tool", &[]);
        let mut record = command(&spec, None).unwrap();
        record.finish(&Err(RunError::EmptyProgram));

        let json = record.to_json_line();

        assert!(json.contains("\"kind\":\"error\""));
        assert!(json.contains("\"exit_code\":125"));
        assert!(json.contains("program must not be empty"));
    }

    fn spec(program: &str, args: &[&str]) -> CommandSpec {
        CommandSpec {
            command: ParsedCommand {
                program: program.into(),
                args: args.iter().map(Into::into).collect(),
            },
            stdin: InputSpec::Inherit,
            stdout: OutputSpec::Inherit,
            stderr: OutputSpec::Inherit,
        }
    }
}
