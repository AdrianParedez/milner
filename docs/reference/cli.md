# CLI Reference

Accepted forms:

```text
milner.exe <program> <args...>
milner.exe [options] <program> <args...>
milner.exe [options] --line <command-line>
milner.exe [options] --prompt
```

## Options

| Option | Value | Behaviour |
| --- | --- | --- |
| `--no-config` | none | Starts with default configuration only. |
| `--config` | `<file>` | Loads configuration from the supplied file. |
| `--cwd` | `<dir>` | Sets the child or prompt current directory. |
| `--set-env` | `NAME=VALUE` | Sets a variable in the child environment. |
| `--unset-env` | `NAME` | Removes a variable from the child environment. |
| `--timeout-ms` | `<ms>` | Cancels foreground execution after the timeout. |
| `--pty` | none | Reserved and explicitly unsupported. |

Options must appear before the direct program, `--line`, or `--prompt`.

## Direct Command Form

Direct form bypasses the Milner line parser. Everything after the options is
treated as the program and its arguments:

```powershell
milner.exe --no-config powershell -NoProfile -Command "Get-Date"
```

The direct path still applies executable resolution, batch-target rejection,
handle policy, current-directory policy, environment policy, timeout policy,
and execution-record policy.

## `--line` Form

`--line` accepts one Milner command line and parses it into typed command data:

```powershell
milner.exe --no-config --line "cargo --version > version.txt"
```

Use this form when you need Milner syntax such as redirection or one pipeline.

## Prompt Form

Prompt form starts the small interactive loop:

```powershell
milner.exe --no-config --prompt
```

The prompt supports built-ins, aliases, history, and repeated external
execution.

## Configuration Loading

Without `--no-config` or `--config`, the default config path is:

```text
%APPDATA%\milner\config.toml
```

If the default file is absent, Milner starts with defaults.
