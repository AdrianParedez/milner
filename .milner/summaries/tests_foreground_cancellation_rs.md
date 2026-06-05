# tests/foreground_cancellation.rs

- Citation: `milner:sha256:f0f5f165cfa64f72:6651e5b6a9c2`
- Source: `tests/foreground_cancellation.rs`
- Category: `test`
- Trace: `milner:sha256:f0f5f165cfa64f72:6651e5b6a9c2`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `timeout_cancels_foreground_child`, `timeout_cancels_descendant_processes`, `foreground_exit_closes_descendant_processes`, `ctrl_break_interrupt_cleans_up_descendant_processes`, `prompt_returns_after_cancelled_foreground_child`, `prompt_recovers_after_failed_foreground_launch`, `run_prompt_with_args`, `delayed_marker_script`

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
- use std::io::Write;
- use std::os::windows::process::CommandExt;
