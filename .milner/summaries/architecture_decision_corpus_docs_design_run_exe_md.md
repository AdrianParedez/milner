# `run.exe` Design

- Citation: `milner:sha256:e80914788ed3bd28:e4ec6179f388`
- Source: `architecture-decision-corpus/docs/design/run-exe.md`
- Category: `specification`
- Trace: `milner:sha256:e80914788ed3bd28:e4ec6179f388`

## Structured Summary
- Architectural decisions: Implement a Windows-only process runner:
- System behavior: The runner validates the program argument, rejects batch targets, builds a
- Domain concepts: Implement a Windows-only process runner:
- Development workflows: # `run.exe` Design
- Testing expectations: Tests cover argument quoting, batch-target rejection, handle-inheritance

## Source Outline
- # `run.exe` Design
- ## Purpose
- ## Scope
- ## Non-Goals
- ## Interfaces
- ## Behaviour
- ## Error Handling
- ## Security
- ## Tests
- ## Future Work

## Evidence Excerpt
- # `run.exe` Design
- ## Purpose
- Implement a Windows-only process runner:
- run.exe <program> <args...>
