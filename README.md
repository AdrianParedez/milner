# Milner

Milner is a from-scratch Rust shell exploration for Windows. The current public
surface is a small process runner and a minimal prompt loop built on direct
Win32 process creation.

```text
milner.exe <program> <args...>
milner.exe [--no-config] [--config <file>] [--cwd <dir>] [--set-env NAME=VALUE] [--unset-env NAME] [--timeout-ms <ms>] <program> <args...>
milner.exe --line <command-line>
milner.exe --prompt
```

Example targets:

```text
milner.exe notepad.exe
milner.exe cargo --version
milner.exe powershell -NoProfile -Command "Get-Date"
milner.exe --line "cargo --version"
milner.exe --line "powershell -NoProfile -Command \"Get-Date\" > date.txt"
milner.exe --timeout-ms 1000 powershell -NoProfile -Command "Start-Sleep -Seconds 30"
milner.exe --config "%APPDATA%\milner\config.toml" --prompt
milner.exe --prompt
```

Batch targets (`.bat` and `.cmd`) are intentionally rejected until Milner has a
cmd-safe invocation policy.

## Current Surface

`milner.exe` launches native executables with direct Win32 calls:

- `CreateProcessW`
- `WaitForSingleObject`
- `GetExitCodeProcess`
- inherited `stdin`, `stdout`, and `stderr`
- current working directory propagation
- environment block handling
- exit-code propagation

`milner.exe --prompt` starts a plain `milner>` loop. It reads one command line at a
time, uses Milner's parser, executes one native command, records the previous exit
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
  current directory, or against Milner's current directory when no child directory
  is configured.
- Absolute paths launch directly.
- `.bat` and `.cmd` targets remain rejected.

Milner passes the resolved native executable as `lpApplicationName` to
`CreateProcessW`. `--cwd <dir>` sets the child current directory without
changing Milner's process-global current directory. `--set-env NAME=VALUE` and
`--unset-env NAME` build a child-only environment block instead of mutating
Milner's process environment.

`--timeout-ms <ms>` applies to each foreground external command or two-command
pipeline. If the timeout expires, Milner terminates unfinished direct foreground
children, records status `130`, reports the cancellation, closes its process
handles, and returns to the prompt when running interactively. Milner does not
install a custom Ctrl+C handler yet, and it does not claim POSIX-style job
control or background process management. Descendant processes created by a
child are not yet grouped or terminated with Windows Job Objects.

## Configuration

Milner starts without a configuration file. By default it looks for:

```text
%APPDATA%\milner\config.toml
```

Use `--config <file>` to load a specific file, or `--no-config` to ignore the
default path. The current parser accepts a deliberately small config subset:

```text
[prompt]
text = "milner> "

[history]
enabled = false
path = C:\Users\you\AppData\Roaming\milner\history.txt

[aliases]
gs = git status
```

Unknown sections and keys are rejected with the config file path and line
number. History is disabled by default. When enabled without an explicit path,
history is written to:

```text
%APPDATA%\milner\history.txt
```

Milner avoids recording command lines containing obvious secret words such as
`password`, `secret`, or `token`. Aliases are parsed into Milner command
structures before execution; they cannot include redirection or pipelines, and
they still pass through executable resolution and batch-target rejection.

No configuration, history, completion, or alias dependency was added in this
milestone. Milner keeps these behaviours local until a Windows-focused line editor
or config-path crate is justified.

## Build And Run

```text
cargo build
.\target\debug\milner.exe cargo --version
.\target\debug\milner.exe --line "cargo --version"
.\target\debug\milner.exe --prompt
.\target\debug\milner.exe powershell -NoProfile -Command "Get-Date"
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
