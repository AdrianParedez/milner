# Quick Start

Run a native executable directly:

```powershell
milner.exe --no-config powershell -NoProfile -Command "Get-Date"
```

Ask Milner to parse a small command line with `--line`:

```powershell
milner.exe --no-config --line "powershell -NoProfile -Command \"Get-Date\""
```

Or start the prompt:

```powershell
milner.exe --no-config --prompt
```

Inside prompt mode:

```text
milner> powershell -NoProfile -Command "Get-Date"
milner> pwd
milner> cd C:\Windows
milner> exit 0
```

## Redirection

Redirect files explicitly. There is no hidden shell layer here:

```powershell
milner.exe --no-config --line "powershell -NoProfile -Command \"[Console]::Out.Write('ok')\" > out.txt"
milner.exe --no-config --line "powershell -NoProfile -Command \"[Console]::Error.Write('err')\" 2> err.txt"
milner.exe --no-config --line "powershell -NoProfile -Command \"$input\" < input.txt"
```

Use `>>` and `2>>` when the file should be appended to instead of replaced.

## Pipelines

One pipe is supported:

```powershell
milner.exe --no-config --line "powershell -NoProfile -Command \"[Console]::Out.Write('ok')\" | powershell -NoProfile -Command \"$input | ForEach-Object { [Console]::Out.Write($_.ToUpperInvariant()) }\""
```

That means two external commands. Longer chains are rejected on purpose.

## Current Directory And Environment

Set a child current directory without changing your parent shell:

```powershell
milner.exe --no-config --cwd C:\Windows powershell -NoProfile -Command "[System.IO.Directory]::GetCurrentDirectory()"
```

Set or remove child-only environment variables:

```powershell
milner.exe --no-config --set-env MILNER_DEMO=1 powershell -NoProfile -Command "$env:MILNER_DEMO"
milner.exe --no-config --set-env MILNER_DEMO=1 --unset-env MILNER_DEMO powershell -NoProfile -Command "if ($env:MILNER_DEMO) { exit 7 } else { exit 0 }"
```

The parent process environment is left alone.
