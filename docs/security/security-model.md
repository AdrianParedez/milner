# Security Model

The security model is intentionally modest: launch native processes explicitly,
keep shell syntax small, and reject behaviour that would require pretending to
be `cmd.exe`.

This is not a sandbox. A child process still runs with the user's privileges.

## Core Policies

Current policy is built around a few hard boundaries:

| Policy | Behaviour |
| --- | --- |
| No `cmd.exe` fallback | Unsupported syntax is rejected instead of being passed to `cmd.exe`. |
| Batch targets rejected | `.bat` and `.cmd` targets are not supported. |
| Explicit executable resolution | Milner resolves the executable before launch and passes it as `lpApplicationName`. |
| PATH-only bare-name search | Bare executable names search `PATH`, not the child current directory. |
| Narrow handle inheritance | Children receive only intended stdin, stdout, and stderr handles. |
| Job Object cleanup | Foreground children run inside a Windows Job Object cleanup boundary. |
| Timeout and interrupt cancellation | Timeout, `Ctrl+C`, and `Ctrl+Break` cancel foreground execution through the cleanup path. |

## Batch Target Policy

`.bat` and `.cmd` targets are rejected because they belong to the Windows
command processor, with its own quoting and metacharacter rules. Milner does
not define a safe `cmd.exe` policy yet, so it refuses the target instead of
guessing.

The rejection accounts for Windows-ignored trailing spaces and dots.

## Executable Resolution

Bare names search `PATH`. They do not search the child current directory. That
choice is visible in the error message when resolution fails.

Use an explicit relative path when the current directory is the point:

```powershell
milner.exe --no-config --cwd C:\tools --line ".\tool.exe"
```

## Handle Inheritance

Only intended stdin, stdout, and stderr handles are duplicated for the child.
The launch path uses an explicit inherited-handle list so unrelated inheritable
handles in the parent do not leak across the boundary.

## What Milner Does Not Claim

It does not claim to:

- sandbox untrusted code
- enforce filesystem, network, or registry permissions
- hide secrets passed to child processes
- make PowerShell commands safe
- emulate full PowerShell, `cmd.exe`, Bash, or POSIX shell behaviour
- host ConPTY or terminal pseudoconsoles
