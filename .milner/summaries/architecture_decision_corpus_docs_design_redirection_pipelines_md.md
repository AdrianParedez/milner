# Redirection And Pipelines Design

- Citation: `milner:sha256:99307a3f91dffaf7:90504d958729`
- Source: `architecture-decision-corpus/docs/design/redirection-pipelines.md`
- Category: `specification`
- Trace: `milner:sha256:99307a3f91dffaf7:90504d958729`

## Structured Summary
- Architectural decisions: Route child standard handles to files and pipes while preserving explicit handle
- System behavior: Keel opens files and pipes before launching children. Each child receives only
- Domain concepts: Route child standard handles to files and pipes while preserving explicit handle
- Development workflows: # Redirection And Pipelines Design
- Testing expectations: Tests should cover truncate output, append output, stdin file input, a

## Source Outline
- # Redirection And Pipelines Design
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
- # Redirection And Pipelines Design
- ## Purpose
- Route child standard handles to files and pipes while preserving explicit handle
- ownership.
