# Troubleshooting

## `milner: usage: ...`

The command did not match one of the accepted forms. Check option order first:
options go before the program, `--line`, or `--prompt`.

## `parse error ... unsupported operator`

This usually means shell syntax slipped into a Milner command line.

Examples:

```text
&&
||
;
`command`
$(command)
```

If you wanted PowerShell semantics, run PowerShell explicitly:

```powershell
milner.exe --no-config powershell -NoProfile -Command "cargo test; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }; cargo clippy"
```

## `batch targets are not supported`

The target is a `.bat` or `.cmd` file. That is intentional policy, not a missing
file. Batch files are not routed through `cmd.exe`.

## `did not search the current directory for bare names`

Bare executable names search `PATH`, not the child current directory.

Make the current-directory lookup explicit:

```powershell
milner.exe --no-config --cwd C:\tools --line ".\tool.exe"
```

## Config Errors

Configuration errors include the file path and line number:

```text
milner: config `C:\Users\you\AppData\Roaming\milner\config.toml` line 2: unknown prompt key
```

Check that settings are inside supported sections, and that booleans are
exactly `true` or `false`.

## Timeout Or Interrupt Exit `130`

Exit code `130` means foreground execution was cancelled or interrupted.

For a timeout, increase or remove `--timeout-ms`:

```powershell
milner.exe --no-config --timeout-ms 5000 powershell -NoProfile -Command "Start-Sleep 1"
```

## Records Do Not Appear

Check:

- `[records] enabled = true`
- the configured record path is writable
- the command does not contain an obvious secret word such as `token` or
  `password`

If persistence itself fails, a warning is printed to stderr. A successful child
command still keeps its own exit code.
