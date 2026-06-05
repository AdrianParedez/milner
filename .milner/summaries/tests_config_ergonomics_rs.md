# tests/config_ergonomics.rs

- Citation: `milner:sha256:7b1162dec54ea20b:c91635ed417c`
- Source: `tests/config_ergonomics.rs`
- Category: `test`
- Trace: `milner:sha256:7b1162dec54ea20b:c91635ed417c`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `prompt_starts_without_config`, `invalid_config_reports_path_and_line`, `prompt_text_can_be_configured`, `history_can_be_disabled`, `history_skips_obvious_secrets`, `completion_lists_builtins_and_aliases_without_executing_aliases`, `aliases_expand_through_typed_commands`, `aliases_cannot_include_stderr_redirection`

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
