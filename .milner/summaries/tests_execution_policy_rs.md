# tests/execution_policy.rs

- Citation: `milner:sha256:400722fed649f3fe:f05070e95bc7`
- Source: `tests/execution_policy.rs`
- Category: `test`
- Trace: `milner:sha256:400722fed649f3fe:f05070e95bc7`

## Structured Summary
- Architectural decisions: #![cfg(windows)]
- System behavior: #![cfg(windows)]
- Domain concepts: #![cfg(windows)]
- Development workflows: #![cfg(windows)]
- Testing expectations: `bare_executable_names_resolve_through_path`, `absolute_executable_paths_launch_directly`, `relative_executable_paths_resolve_against_child_cwd`, `child_cwd_is_set_without_parent_global_directory_change`, `child_environment_is_inherited_by_default`, `child_environment_can_be_extended`, `child_environment_can_remove_child_only_values`, `bare_names_do_not_search_child_cwd`

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
- use std::path::{Path, PathBuf};
- use std::process::Output;
