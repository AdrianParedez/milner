# Interactive Prompt Design

- Citation: `milner:sha256:a9f6808de0e6e05e:35ac09736b3b`
- Source: `architecture-decision-corpus/docs/design/interactive-prompt.md`
- Category: `specification`
- Trace: `milner:sha256:a9f6808de0e6e05e:35ac09736b3b`

## Structured Summary
- Architectural decisions: Add a minimal interactive loop that reads one command line, parses it, executes
- System behavior: The loop prints `keel>`, reads one line, parses it, and runs the parsed command.
- Domain concepts: Add a minimal interactive loop that reads one command line, parses it, executes
- Development workflows: # Interactive Prompt Design
- Testing expectations: Tests should cover empty input, EOF, parser error recovery, command execution,

## Source Outline
- # Interactive Prompt Design
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
- # Interactive Prompt Design
- ## Purpose
- Add a minimal interactive loop that reads one command line, parses it, executes
- it, records the status, and prompts again.
