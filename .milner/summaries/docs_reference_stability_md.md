# Stability Contract

- Citation: `milner:sha256:5499319f0612476c:b65b77272344`
- Source: `docs/reference/stability.md`
- Category: `documentation`
- Trace: `milner:sha256:5499319f0612476c:b65b77272344`

## Structured Summary
- Architectural decisions: Before v1.0, deprecated behaviour may move faster than it would in a mature
- System behavior: # Stability Contract
- Domain concepts: Patch releases should preserve these behaviours:
- Development workflows: # Stability Contract
- Testing expectations: # Stability Contract

## Source Outline
- # Stability Contract
- ## Public API
- ## Stable Surface
- ## Experimental Surface
- ## Unsupported Surface
- ## Versioning Rules
- ## Security Invariants
- ## Deprecation Policy
- ## Audit Evidence

## Evidence Excerpt
- # Stability Contract
- Milner is still pre-1.0, so some edges will move. Even so, the documented CLI
- should not feel like wet paint. If a behaviour is listed here, treat it as part
- of the product promise unless a release note says otherwise.
