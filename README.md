# Keel

Keel is a from-scratch Rust shell exploration for Windows. The first milestone is not an interactive prompt. It is a small process runner that calls the Win32 process creation path directly.

```text
run.exe <program> <args...>
```

Example targets:

```text
run.exe notepad.exe
run.exe cargo --version
run.exe powershell -NoProfile -Command "Get-Date"
```

## Current Milestone

Build `run.exe` with direct Win32 calls:

- `CreateProcessW`
- `WaitForSingleObject`
- `GetExitCodeProcess`
- inherited `stdin`, `stdout`, and `stderr`
- current working directory propagation
- environment block handling
- exit-code propagation

The point is to learn the lowest practical shell layer on Windows before building command parsing, an interactive prompt, or a terminal UI.

## Build And Run

```text
cargo build
.\target\debug\run.exe cargo --version
.\target\debug\run.exe powershell -NoProfile -Command "Get-Date"
```

Verification:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Documentation

- [Win32 process runner research](docs/research/win32-process-runner.md)
- [`windows-sys` feature research](docs/research/windows-sys-features.md)
- [`run.exe` design](docs/design/run-exe.md)
- [Implementation checklist](docs/checklists/process-runner-acceptance.md)

## Source Policy

Use primary sources first. For this milestone that means Microsoft Learn, Rust standard library documentation, and docs.rs for the selected Win32 binding crate.
