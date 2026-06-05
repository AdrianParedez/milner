# `windows-sys` Feature Research

- Citation: `milner:sha256:12c9511ef98520ae:0d71fa82e786`
- Source: `architecture-decision-corpus/docs/research/windows-sys-features.md`
- Category: `corpus`
- Trace: `milner:sha256:12c9511ef98520ae:0d71fa82e786`

## Structured Summary
- Architectural decisions: Enable only the four features above for the first milestone. Do not enable the umbrella `Win32` or `Win32_System` features. They are broader than necessary and would make the dependency declaration less informative.
- System behavior: # `windows-sys` Feature Research
- Domain concepts: This note records the exact `windows-sys` dependency features needed for Keel's first `run.exe` milestone.
- Development workflows: The resolved crate source at:
- Testing expectations: # `windows-sys` Feature Research

## Source Outline
- # `windows-sys` Feature Research
- ## Purpose
- ## Primary Sources
- ## Findings
- ### Resolved Crate
- ### Required Features
- ### Local Symbol Verification
- ## Design Implications
- ## Open Questions

## Evidence Excerpt
- # `windows-sys` Feature Research
- Research date: 2026-06-04
- ## Purpose
- This note records the exact `windows-sys` dependency features needed for Keel's first `run.exe` milestone.
