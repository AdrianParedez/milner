# Exit Codes

When a child process launches successfully, its exit code is Milner's exit code
unless Milner has to report a runner-level failure.

## Runner-Level Codes

| Code | Meaning |
| --- | --- |
| `2` | Usage error, parse error, invalid option value, unsupported feature, or alias cycle. |
| `125` | Runner setup or policy failure before successful execution, including invalid config, invalid cwd, missing executable, unsupported batch target, I/O setup failure, invalid handle, or Win32 process creation failure. |
| `126` | Wait or exit-code retrieval failure after process creation. |
| `130` | Foreground command cancelled by timeout or interrupted by `Ctrl+C` or `Ctrl+Break`. |

## Child Exit Codes

If the child exits with `7`, Milner exits with `7`:

```powershell
milner.exe --no-config powershell -NoProfile -Command "exit 7"
$LASTEXITCODE
```

For a two-command pipeline, Milner returns the last command's exit code after
both children exit.

## Prompt Mode

The prompt tracks the last command status:

- EOF exits with the last status.
- `exit` without an argument exits with the last status.
- `exit <code>` exits with the requested code when it is in `0..255`.
- Parser and built-in errors update the last status and return to the prompt.
