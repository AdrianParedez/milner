# tests/security_regression.rs

- Citation: `milner:sha256:8382a538397ea60c:8f787cfcc555`
- Source: `tests/security_regression.rs`
- Category: `test`
- Trace: `milner:sha256:8382a538397ea60c:8f787cfcc555`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `rejects_batch_targets_before_cmd_can_reinterpret_arguments`, `rejects_windows_normalized_batch_targets_before_launch`, `parsed_command_line_launches_through_process_runner`, `parsed_batch_targets_remain_rejected_before_launch`, `parsed_batch_targets_with_trailing_spaces_remain_rejected_before_launch`, `child_process_does_not_receive_unrelated_inheritable_handles`, `pipeline_children_do_not_receive_unrelated_inheritable_handles`, `temp_dir`

## Source Outline
- #![cfg(windows)]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]
- #[test]

## Evidence Excerpt
- #![cfg(windows)]
- use std::ffi::{OsStr, OsString};
- use std::fs::{self, OpenOptions};
- use std::os::windows::ffi::OsStrExt;
