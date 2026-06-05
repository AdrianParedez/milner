# Limitations

The project is intentionally small. That is a design boundary, not a temporary
documentation omission.

## Platform

Windows only.

## Shell Semantics

This is not PowerShell, `cmd.exe`, Bash, or a POSIX shell.

Unsupported:

- arbitrary-length pipelines
- variables
- glob expansion
- command substitution
- descriptor duplication such as `2>&1`
- here-documents
- background jobs
- shell functions
- shell profiles
- `.bat` or `.cmd` targets

## Terminal Hosting

ConPTY pseudoconsole hosting is not implemented. The `--pty` option is reserved
and returns an explicit unsupported-feature error.

Interactive console programs may work when ordinary stdin, stdout, and stderr
are sufficient. Programs that require terminal-hosting behaviour are outside
the current surface.

## Configuration

The configuration file is a strict small subset, not a full TOML
implementation. Unknown fields are rejected.

## Rust Library API

There is no stable Rust library API yet. Generated Rust docs describe
implementation details.
