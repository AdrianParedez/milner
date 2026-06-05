# Job Object Execution Boundary Design

- Citation: `milner:sha256:7e3f9d22bd2b99cd:bb78dcbe30ff`
- Source: `architecture-decision-corpus/docs/design/job-object-boundary.md`
- Category: `specification`
- Trace: `milner:sha256:7e3f9d22bd2b99cd:bb78dcbe30ff`

## Structured Summary
- Architectural decisions: Make foreground execution bounded at the process-tree level using Windows Job
- System behavior: Before launching a foreground external command, Milner creates a job object and
- Domain concepts: Make foreground execution bounded at the process-tree level using Windows Job
- Development workflows: # Job Object Execution Boundary Design
- Testing expectations: - Unit tests for job setup error mapping where practical.

## Source Outline
- # Job Object Execution Boundary Design
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
- # Job Object Execution Boundary Design
- ## Purpose
- Make foreground execution bounded at the process-tree level using Windows Job
- Objects.
