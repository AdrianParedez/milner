# Console Interrupt Policy Design

- Citation: `milner:sha256:ec694b6345d4bb0d:a3108e90c2f4`
- Source: `architecture-decision-corpus/docs/design/console-interrupt-policy.md`
- Category: `specification`
- Trace: `milner:sha256:ec694b6345d4bb0d:a3108e90c2f4`

## Structured Summary
- Architectural decisions: Define Windows-native Ctrl+C and Ctrl+Break behaviour that preserves Milner's
- System behavior: Milner should start with a conservative policy:
- Domain concepts: Define Windows-native Ctrl+C and Ctrl+Break behaviour that preserves Milner's
- Development workflows: # Console Interrupt Policy Design
- Testing expectations: - Prompt remains usable after a simulated interrupt.

## Source Outline
- # Console Interrupt Policy Design
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
- # Console Interrupt Policy Design
- ## Purpose
- Define Windows-native Ctrl+C and Ctrl+Break behaviour that preserves Milner's
- foreground state and process cleanup guarantees.
