# Keel

Keel is a from-scratch Rust shell exploration for Windows. The current public
surface is a small process runner and a minimal prompt loop built on direct
Win32 process creation.

```text
run.exe <program> <args...>
run.exe --line <command-line>
run.exe --prompt
```

Example targets:

```text
run.exe notepad.exe
run.exe cargo --version
run.exe powershell -NoProfile -Command "Get-Date"
run.exe --line "cargo --version"
run.exe --line "powershell -NoProfile -Command \"Get-Date\" > date.txt"
run.exe --prompt
```

Batch targets (`.bat` and `.cmd`) are intentionally rejected until Keel has a
cmd-safe invocation policy.

## Current Surface

`run.exe` launches native executables with direct Win32 calls:

- `CreateProcessW`
- `WaitForSingleObject`
- `GetExitCodeProcess`
- inherited `stdin`, `stdout`, and `stderr`
- current working directory propagation
- environment block handling
- exit-code propagation

`run.exe --prompt` starts a plain `keel>` loop. It reads one command line at a
time, uses Keel's parser, executes one native command, records the previous exit
code internally, and exits cleanly on EOF.

The prompt resolves these built-ins before external commands:

- `cd <path>` changes the shell current directory.
- `pwd` prints the shell current directory.
- `exit [code]` exits the prompt with the provided code, or with the previous
  command status when no code is provided.

The prompt intentionally has no line editing, history, completion, syntax
highlighting, arbitrary-length pipelines, stderr redirection, or custom Ctrl+C
behaviour yet.

Command-line parsing supports:

- `>` to redirect stdout and replace the target file.
- `>>` to redirect stdout and append to the target file.
- `<` to read stdin from a file.
- `|` to connect exactly two external commands.

Each child receives only its intended stdin, stdout, and stderr handles. Pipeline
support is intentionally limited to two commands until longer ownership chains
are designed and tested.

## Build And Run

```text
cargo build
.\target\debug\run.exe cargo --version
.\target\debug\run.exe --line "cargo --version"
.\target\debug\run.exe --prompt
.\target\debug\run.exe powershell -NoProfile -Command "Get-Date"
```

Verification:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Optional combined test runner:

```text
cargo install cargo-make cargo-nextest
cargo make verify-tests
```
