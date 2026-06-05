# Win32 Process Runner Research

- Citation: `milner:sha256:ddd8c8330bcda416:95aa8d5304a8`
- Source: `architecture-decision-corpus/docs/research/win32-process-runner.md`
- Category: `corpus`
- Trace: `milner:sha256:ddd8c8330bcda416:95aa8d5304a8`

## Structured Summary
- Architectural decisions: - Keep `CreateProcessW` behind a small typed Rust boundary.
- System behavior: # Win32 Process Runner Research
- Domain concepts: This document captures the source-backed behavior needed for Keel's first milestone: a Rust `run.exe` that launches a child process through the Win32 API and returns the child's exit code.
- Development workflows: # Win32 Process Runner Research
- Testing expectations: # Win32 Process Runner Research

## Source Outline
- # Win32 Process Runner Research
- ## Purpose
- ## Primary Sources
- ## Findings
- ### `CreateProcessW`
- ### Command Line Construction
- ### Handle Inheritance and Standard I/O
- ### Current Directory
- ### Environment Block
- ### Waiting and Exit Codes
- ### Handle Lifetime
- ### Rust Binding Choice

## Evidence Excerpt
- # Win32 Process Runner Research
- Research date: 2026-06-04
- ## Purpose
- This document captures the source-backed behavior needed for Keel's first milestone: a Rust `run.exe` that launches a child process through the Win32 API and returns the child's exit code.
