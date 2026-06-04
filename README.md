<div align="center">

# Milner

**A Windows-first Rust shell experiment built around explicit process launch,
typed parsing, and narrow execution policy.**

`CreateProcessW` over `cmd.exe` fallback. Explicit handles over ambient
inheritance. Small shell syntax over accidental compatibility.

<p>
  <a href="https://github.com/AdrianParedez/milner/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/AdrianParedez/milner/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/AdrianParedez/milner/actions/workflows/docs.yml"><img alt="Docs" src="https://github.com/AdrianParedez/milner/actions/workflows/docs.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue"></a>
  <img alt="Rust 2024" src="https://img.shields.io/badge/rust-2024-f74c00">
  <img alt="Windows" src="https://img.shields.io/badge/platform-Windows-0078D4">
  <img alt="Experimental" src="https://img.shields.io/badge/status-experimental-6f42c1">
</p>

<p>
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#capabilities">Capabilities</a> ·
  <a href="#supported-syntax">Syntax</a> ·
  <a href="#configuration">Configuration</a> ·
  <a href="#verification">Verification</a>
</p>

<p><em>Created by <a href="https://paredez.dev">Adrian Paredez</a>.</em></p>

</div>

<!-- Keep the public README practical. Longer architecture notes belong in
dedicated docs once the AI-assisted runtime design settles. -->

M.I.L.N.E.R. stands for **Managed Intent Layer for Native Execution Runtime**.
It is named in homage to Robin Milner, whose work on typed systems and
communicating processes reflects the project's focus on structured intent,
bounded execution, and process interaction.

> [!IMPORTANT]
> Milner is experimental. It is useful for studying a hardened Windows process
> runner and a deliberately small shell surface. Treat it as a research-grade
> command runtime, not as a production terminal environment.

> [!WARNING]
> Milner intentionally rejects `.bat` and `.cmd` targets. Batch files have
> command processor quoting and metacharacter semantics that Milner does not yet
> claim to make safe.

## Install

Download the latest Windows release binary:

```powershell
$asset = "milner-x86_64-pc-windows-msvc.zip"
$base = "https://github.com/AdrianParedez/milner/releases/latest/download"

Invoke-WebRequest "$base/$asset" -OutFile $asset
Invoke-WebRequest "$base/$asset.sha256" -OutFile "$asset.sha256"

$expected = (Get-Content "$asset.sha256").Split(" ")[0]
$actual = (Get-FileHash $asset -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "checksum mismatch" }

Expand-Archive $asset -DestinationPath .\milner
.\milner\milner.exe --prompt
```

Install from source:

```powershell
cargo install --git https://github.com/AdrianParedez/milner --locked
milner --prompt
```

Release binaries are published when a `vX.Y.Z` tag is pushed.

## Quick Start

```powershell
cargo build
.\target\debug\milner.exe --no-config cargo --version
.\target\debug\milner.exe --no-config --line "cargo --version"
.\target\debug\milner.exe --no-config --prompt
```

Prompt mode:

```text
milner> cargo --version
milner> pwd
milner> cd C:\Windows
milner> exit 0
```

Use `exit [code]` to leave prompt mode. In a Windows console, <kbd>Ctrl</kbd> +
<kbd>Z</kbd> then <kbd>Enter</kbd> sends EOF.

## Command Surface

```text
milner.exe <program> <args...>
milner.exe [options] <program> <args...>
milner.exe [options] --line <command-line>
milner.exe [options] --prompt

options:
  --no-config
  --config <file>
  --cwd <dir>
  --set-env NAME=VALUE
  --unset-env NAME
  --timeout-ms <ms>
```

Examples:

```powershell
milner.exe --no-config notepad.exe
milner.exe --no-config powershell -NoProfile -Command "Get-Date"
milner.exe --no-config --line "cargo --version > version.txt"
milner.exe --no-config --timeout-ms 1000 powershell -NoProfile -Command "Start-Sleep 30"
```

Policy shape at a glance:

```diff
+ milner.exe --line "cargo --version"
+ milner.exe --line "cargo --version > version.txt"
+ milner.exe --line "where powershell | findstr powershell"
- milner.exe --line "build.cmd"
- milner.exe --line "echo %PATH%"
- milner.exe --line "cargo test && cargo clippy"
```

## Capabilities

| Area | Current behaviour |
| --- | --- |
| Native launch | Resolves executables and launches with `CreateProcessW`.[^createprocess] |
| Command parsing | Parses commands into typed structures before execution. |
| Prompt mode | Provides `cd`, `pwd`, `complete`, and `exit` built-ins. |
| Redirection | Supports stdin redirection and stdout write/append redirection. |
| Pipelines | Supports exactly one two-command external pipeline. |
| Execution policy | Rejects unsupported operators, `.bat`, and `.cmd` targets. |
| Handles | Gives children only intended `stdin`, `stdout`, and `stderr`. |
| Configuration | Reads a small config subset for prompt, history, and aliases. |
| Cancellation | Cancels foreground commands with `--timeout-ms <ms>`. |

<details>
<summary><strong>More command examples</strong></summary>

```powershell
milner.exe --no-config --cwd C:\Windows notepad.exe
milner.exe --no-config --set-env MILNER_DEMO=1 powershell -NoProfile -Command "$env:MILNER_DEMO"
milner.exe --no-config --line "cargo --version >> output.txt"
milner.exe --no-config --timeout-ms 1000 powershell -NoProfile -Command "Start-Sleep 30"
```

</details>

## Supported Syntax

| Form | Status | Notes |
| --- | --- | --- |
| `program arg` | Supported | Bare words and quoted arguments. |
| `"empty" ""` | Supported | Empty quoted arguments are preserved. |
| `stdout > file` | Supported | Redirect stdout, replacing the target. |
| `stdout >> file` | Supported | Redirect stdout, appending to the target. |
| `stdin < file` | Supported | Redirect stdin from a file. |
| `left \| right` | Supported | Exactly one external two-command pipeline. |
| `&&`, `\|\|`, `;` | Rejected | Explicit parse errors. |
| Variables, globs, substitution | Not supported | No silent expansion. |
| `.bat`, `.cmd` | Rejected | No `cmd.exe` fallback. |

## Configuration

Milner starts without configuration.

Default config path:

```text
%APPDATA%\milner\config.toml
```

Use:

```powershell
.\target\debug\milner.exe --config "$env:APPDATA\milner\config.toml" --prompt
.\target\debug\milner.exe --no-config --prompt
```

Current config subset:

```ini
[prompt]
text = "milner> "

[history]
enabled = false
path = C:\Users\you\AppData\Roaming\milner\history.txt

[aliases]
gs = git status
```

> [!IMPORTANT]
> Unknown sections and keys are rejected with a file path and line number.

*History is disabled by default.* When enabled without an explicit path, Milner
writes history to:

```text
%APPDATA%\milner\history.txt
```

Milner avoids recording command lines that contain obvious secret words such as
`password`, `secret`, or `token`.

<details>
<summary><strong>Alias boundaries</strong></summary>

- Alias cycles are detected.
- Alias values cannot include redirection.
- Alias values cannot include pipelines.
- Alias expansion cannot bypass executable resolution.
- Alias expansion cannot bypass `.bat` or `.cmd` rejection.

</details>

## Verification

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
.\target\debug\milner.exe --no-config powershell -NoProfile -Command "exit 0"
```

Optional combined runner:

```powershell
cargo install cargo-make cargo-nextest
cargo make verify-tests
```

Generate local Rust HTML documentation:

```powershell
cargo doc --no-deps --document-private-items
```

Open:

```text
target\doc\milner\index.html
```

GitHub Actions also uploads generated Rust HTML docs as the
`milner-rustdoc-html` artifact.

<details>
<summary><strong>Known limitations</strong></summary>

- Windows-only.
- No arbitrary-length pipelines.
- No stderr redirection.
- No variables, globbing, aliases with redirection, or command substitution.
- No background jobs.
- No custom Ctrl+C handling.
- No Windows Job Object process-tree cleanup yet.
- No line editor crate, persistent completion engine, or theme system.

</details>

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

[^createprocess]: `CreateProcessW` still receives a Windows command-line string,
    but Milner resolves the executable path first and passes it separately as
    `lpApplicationName`.
