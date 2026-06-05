# Milestone 04: Shell State And Built-Ins

- Citation: `milner:sha256:8614b4dd8ed5964c:d9452ef28d1e`
- Source: `architecture-decision-corpus/docs/milestones/04-shell-state-builtins.md`
- Category: `corpus`
- Trace: `milner:sha256:8614b4dd8ed5964c:d9452ef28d1e`

## Structured Summary
- Architectural decisions: # Milestone 04: Shell State And Built-Ins
- System behavior: # Milestone 04: Shell State And Built-Ins
- Domain concepts: - Add a `ShellState` type.
- Development workflows: - `cd` affects the next external command.
- Testing expectations: - `cd` affects the next external command.

## Source Outline
- # Milestone 04: Shell State And Built-Ins
- ## Status
- ## Goal
- ## Scope
- ## Non-Goals
- ## Supporting Documents
- ## Acceptance Gates
- ## Exit Criteria

## Evidence Excerpt
- # Milestone 04: Shell State And Built-Ins
- ## Status
- Completed in `fa517d8 feat(prompt): add shell state builtins`.
- The shipped prompt owns shell current directory state, tracks the previous
