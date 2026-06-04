# Keel

Keel is a from-scratch Rust shell exploration for Windows. The current public
surface is a small process runner and a minimal prompt loop built on direct
Win32 process creation.

```text
run.exe <program> <args...>
run.exe [--no-config] [--config <file>] [--cwd <dir>] [--set-env NAME=VALUE] [--unset-env NAME] [--timeout-ms <ms>] <program> <args...>
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
run.exe --timeout-ms 1000 powershell -NoProfile -Command "Start-Sleep -Seconds 30"
run.exe --config "%APPDATA%\keel\config.toml" --prompt
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
- `complete [prefix]` prints matching built-in and alias names without running
  them.
- `pwd` prints the shell current directory.
- `exit [code]` exits the prompt with the provided code, or with the previous
  command status when no code is provided.

The prompt intentionally has no line editing, syntax highlighting,
arbitrary-length pipelines, stderr redirection, or custom Ctrl+C behaviour yet.

Command-line parsing supports:

- `>` to redirect stdout and replace the target file.
- `>>` to redirect stdout and append to the target file.
- `<` to read stdin from a file.
- `|` to connect exactly two external commands.

Each child receives only its intended stdin, stdout, and stderr handles. Pipeline
support is intentionally limited to two commands until longer ownership chains
are designed and tested.

External commands are resolved before launch:

- Bare executable names are searched through `PATH` only. The current directory
  is not searched for bare names unless it is explicitly present in `PATH`.
- Relative paths such as `.\tool.exe` resolve against the configured child
  current directory, or against Keel's current directory when no child directory
  is configured.
- Absolute paths launch directly.
- `.bat` and `.cmd` targets remain rejected.

Keel passes the resolved native executable as `lpApplicationName` to
`CreateProcessW`. `--cwd <dir>` sets the child current directory without
changing Keel's process-global current directory. `--set-env NAME=VALUE` and
`--unset-env NAME` build a child-only environment block instead of mutating
Keel's process environment.

`--timeout-ms <ms>` applies to each foreground external command or two-command
pipeline. If the timeout expires, Keel terminates unfinished direct foreground
children, records status `130`, reports the cancellation, closes its process
handles, and returns to the prompt when running interactively. Keel does not
install a custom Ctrl+C handler yet, and it does not claim POSIX-style job
control or background process management. Descendant processes created by a
child are not yet grouped or terminated with Windows Job Objects.

## Configuration

Keel starts without a configuration file. By default it looks for:

```text
%APPDATA%\keel\config.toml
```

Use `--config <file>` to load a specific file, or `--no-config` to ignore the
default path. The current parser accepts a deliberately small config subset:

```text
[prompt]
text = "keel> "

[history]
enabled = false
path = C:\Users\you\AppData\Roaming\keel\history.txt

[aliases]
gs = git status
```

Unknown sections and keys are rejected with the config file path and line
number. History is disabled by default. When enabled without an explicit path,
history is written to:

```text
%APPDATA%\keel\history.txt
```

Keel avoids recording command lines containing obvious secret words such as
`password`, `secret`, or `token`. Aliases are parsed into Keel command
structures before execution; they cannot include redirection or pipelines, and
they still pass through executable resolution and batch-target rejection.

No configuration, history, completion, or alias dependency was added in this
milestone. Keel keeps these behaviours local until a Windows-focused line editor
or config-path crate is justified.

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
