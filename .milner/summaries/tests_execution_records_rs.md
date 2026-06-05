# tests/execution_records.rs

- Citation: `milner:sha256:b52e0fcb7fa99958:270c95360d42`
- Source: `tests/execution_records.rs`
- Category: `test`
- Trace: `milner:sha256:b52e0fcb7fa99958:270c95360d42`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `successful_command_writes_execution_record`, `failed_resolution_writes_error_record`, `timeout_writes_error_record`, `secret_bearing_command_is_skipped`, `persistence_failure_does_not_change_command_exit_status`, `pipeline_record_preserves_pipeline_shape`, `run<const N: usize>`, `write_records_config`

## Source Outline
- #![cfg(windows)]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]

## Evidence Excerpt
- #![cfg(windows)]
- use std::fs;
- use std::path::{Path, PathBuf};
- use std::process::{Command, Output};
