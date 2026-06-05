# Structured Execution Records Design

- Citation: `milner:sha256:02845082fb0e1204:5498fb1118fd`
- Source: `architecture-decision-corpus/docs/design/structured-execution-records.md`
- Category: `specification`
- Trace: `milner:sha256:02845082fb0e1204:5498fb1118fd`

## Structured Summary
- Architectural decisions: Create a typed local record of Milner executions so behaviour can be inspected
- System behavior: Milner creates an in-memory record after parsing succeeds and before execution
- Domain concepts: Create a typed local record of Milner executions so behaviour can be inspected
- Development workflows: # Structured Execution Records Design
- Testing expectations: - Serialization emits valid records for simple commands.

## Source Outline
- # Structured Execution Records Design
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
- # Structured Execution Records Design
- ## Purpose
- Create a typed local record of Milner executions so behaviour can be inspected
- without scraping stdout or stderr.
