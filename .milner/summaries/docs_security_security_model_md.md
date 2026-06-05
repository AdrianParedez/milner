# Security Model

- Citation: `milner:sha256:98b4f11dd9c5865d:b1478e33ace9`
- Source: `docs/security/security-model.md`
- Category: `documentation`
- Trace: `milner:sha256:98b4f11dd9c5865d:b1478e33ace9`

## Structured Summary
- Architectural decisions: | --- | --- |
- System behavior: | --- | --- |
- Domain concepts: The security model is intentionally modest: launch native processes explicitly,
- Development workflows: # Security Model
- Testing expectations: # Security Model

## Source Outline
- # Security Model
- ## Core Policies
- ## Batch Target Policy
- ## Executable Resolution
- ## Handle Inheritance
- ## What Milner Does Not Claim

## Evidence Excerpt
- # Security Model
- The security model is intentionally modest: launch native processes explicitly,
- keep shell syntax small, and reject behaviour that would require pretending to
- be `cmd.exe`.
