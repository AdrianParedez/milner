# tests/redirection_pipeline.rs

- Citation: `milner:sha256:c8283a28bb19a643:a8383c59f157`
- Source: `tests/redirection_pipeline.rs`
- Category: `test`
- Trace: `milner:sha256:c8283a28bb19a643:a8383c59f157`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `stdout_redirection_writes_new_file`, `stdout_redirection_truncates_existing_file`, `stdout_redirection_appends_existing_file`, `stdin_redirection_reads_from_file`, `stderr_redirection_writes_new_file_without_capturing_stdout`, `stderr_redirection_truncates_existing_file`, `stderr_redirection_appends_existing_file`, `two_command_pipeline_transfers_bytes_and_delivers_eof`

## Source Outline
- #![cfg(windows)]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]

## Evidence Excerpt
- #![cfg(windows)]
- use std::fs;
- use std::path::PathBuf;
- use std::process::Output;
