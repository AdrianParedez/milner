# Milestone 05: Redirection And Pipelines

- Citation: `milner:sha256:020b6204b762321a:cc7b1e12a09d`
- Source: `architecture-decision-corpus/docs/milestones/05-redirection-pipelines.md`
- Category: `corpus`
- Trace: `milner:sha256:020b6204b762321a:cc7b1e12a09d`

## Structured Summary
- Architectural decisions: # Milestone 05: Redirection And Pipelines
- System behavior: # Milestone 05: Redirection And Pipelines
- Domain concepts: - Extend the parser to recognise `>`, `>>`, `<`, and `|`.
- Development workflows: - `echo` equivalent output can be redirected to a file through a stable test
- Testing expectations: - `echo` equivalent output can be redirected to a file through a stable test

## Source Outline
- # Milestone 05: Redirection And Pipelines
- ## Status
- ## Goal
- ## Scope
- ## Non-Goals
- ## Supporting Documents
- ## Acceptance Gates
- ## Exit Criteria

## Evidence Excerpt
- # Milestone 05: Redirection And Pipelines
- ## Status
- Completed in `cac3cbf feat(process): add redirection and pipelines`.
- The shipped parser and executor support stdout redirection with `>` and `>>`,
