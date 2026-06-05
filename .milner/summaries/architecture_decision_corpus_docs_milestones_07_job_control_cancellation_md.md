# Milestone 07: Job Control And Cancellation

- Citation: `milner:sha256:5f4ef083045caf29:ee089f9eee23`
- Source: `architecture-decision-corpus/docs/milestones/07-job-control-cancellation.md`
- Category: `corpus`
- Trace: `milner:sha256:5f4ef083045caf29:ee089f9eee23`

## Structured Summary
- Architectural decisions: # Milestone 07: Job Control And Cancellation
- System behavior: # Milestone 07: Job Control And Cancellation
- Domain concepts: - Track the active foreground child.
- Development workflows: - A long-running child can be interrupted without corrupting shell state.
- Testing expectations: - A long-running child can be interrupted without corrupting shell state.

## Source Outline
- # Milestone 07: Job Control And Cancellation
- ## Status
- ## Goal
- ## Scope
- ## Non-Goals
- ## Supporting Documents
- ## Acceptance Gates
- ## Exit Criteria

## Evidence Excerpt
- # Milestone 07: Job Control And Cancellation
- ## Status
- Completed in `ed1243f feat(process): add foreground cancellation timeout`.
- The shipped implementation tracks foreground child handles through a
