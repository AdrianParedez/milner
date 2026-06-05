# tests/prompt_regression.rs

- Citation: `milner:sha256:4b7380ca9f01a374:dcffd534077f`
- Source: `tests/prompt_regression.rs`
- Category: `test`
- Trace: `milner:sha256:4b7380ca9f01a374:dcffd534077f`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `prompt_displays_prompt_and_exits_on_eof`, `prompt_treats_empty_lines_as_no_ops`, `prompt_runs_cargo_version`, `prompt_returns_last_child_exit_code_on_eof`, `prompt_exit_builtin_uses_requested_code`, `prompt_pwd_prints_shell_current_directory`, `prompt_cd_affects_next_external_command`, `prompt_reports_builtin_argument_errors`

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
- #[test]

## Evidence Excerpt
- #![cfg(windows)]
- use std::fs;
- use std::io::Write;
- use std::path::{Path, PathBuf};
