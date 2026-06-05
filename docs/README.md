# Milner Documentation

Milner is a Windows-first command runtime built around explicit native process
launch, typed command parsing, and a deliberately narrow execution policy.

Start with the user guide if you want to run it. Use the reference pages when
you need exact behaviour, exit codes, or data formats.

The public API is the CLI and the local data formats it writes when users opt
in. The Rust module layout is still implementation detail.

## User Guide

- [Install](user-guide/install.md)
- [Quick start](user-guide/quick-start.md)
- [Prompt mode](user-guide/prompt.md)

## Reference

- [CLI reference](reference/cli.md)
- [Command syntax](reference/syntax.md)
- [Configuration](reference/configuration.md)
- [Execution records](reference/execution-records.md)
- [Exit codes](reference/exit-codes.md)
- [Generated Rust docs](reference/generated-rustdoc.md)

## Safety And Operations

- [Security model](security/security-model.md)
- [Privacy notes](security/privacy.md)
- [Limitations](limitations.md)
- [Troubleshooting](troubleshooting.md)

## Status

This project is still experimental. It is useful for studying and testing a
hardened Windows process runner. It is not a production terminal replacement,
a PowerShell replacement, or a sandbox.
