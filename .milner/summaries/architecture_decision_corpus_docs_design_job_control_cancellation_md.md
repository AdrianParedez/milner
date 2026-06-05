# Job Control And Cancellation Design

- Citation: `milner:sha256:5a7507a57cdb6b97:b7a3df1c10b2`
- Source: `architecture-decision-corpus/docs/design/job-control-cancellation.md`
- Category: `specification`
- Trace: `milner:sha256:5a7507a57cdb6b97:b7a3df1c10b2`

## Structured Summary
- Architectural decisions: Handle long-running foreground children deliberately, including cancellation and
- System behavior: Keel records the active foreground task when a child starts and clears it when
- Domain concepts: Handle long-running foreground children deliberately, including cancellation and
- Development workflows: # Job Control And Cancellation Design
- Testing expectations: Tests should cover long-running child cancellation, foreground state cleanup,

## Source Outline
- # Job Control And Cancellation Design
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
- # Job Control And Cancellation Design
- ## Purpose
- Handle long-running foreground children deliberately, including cancellation and
- cleanup.
