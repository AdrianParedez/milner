# Shell State And Built-Ins Design

- Citation: `milner:sha256:c3f098bb919d6fcf:1f8d0ce140ff`
- Source: `architecture-decision-corpus/docs/design/shell-state-builtins.md`
- Category: `specification`
- Trace: `milner:sha256:c3f098bb919d6fcf:1f8d0ce140ff`

## Structured Summary
- Architectural decisions: Introduce shell-owned state and the first commands that must execute inside the
- System behavior: `cd` changes shell state and the current directory used by future children.
- Domain concepts: Introduce shell-owned state and the first commands that must execute inside the
- Development workflows: # Shell State And Built-Ins Design
- Testing expectations: Tests should prove `cd` affects later external commands, `pwd` matches shell

## Source Outline
- # Shell State And Built-Ins Design
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
- # Shell State And Built-Ins Design
- ## Purpose
- Introduce shell-owned state and the first commands that must execute inside the
- shell process.
