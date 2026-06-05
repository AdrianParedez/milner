# Prompt Mode

Start the prompt:

```powershell
milner.exe --no-config --prompt
```

It reads one command per line. End the session with `exit [code]`, or send EOF
with `Ctrl+Z` followed by `Enter` in a Windows console.

## Built-Ins

The prompt handles a small set of built-ins itself:

| Built-in | Behaviour |
| --- | --- |
| `cd <path>` | Changes the prompt current directory. |
| `pwd` | Prints the prompt current directory. |
| `complete [prefix]` | Prints built-in and alias completions matching the prefix. |
| `exit [code]` | Exits prompt mode. Without a code, uses the last status. |

Built-ins do not take redirection or pipelines. If you try, the prompt reports
the error and stays open.

## External Commands

External commands use the same typed execution policy as direct and `--line`
mode. A prompt `cd` changes the directory used by later external commands.

```text
milner> cd C:\Windows
milner> powershell -NoProfile -Command "[System.IO.Directory]::GetCurrentDirectory()"
milner> exit 0
```

## Errors

Parser errors and failed launches are non-fatal. The prompt prints the error,
updates the last status, and waits for the next line.

## History

History is off by default. When enabled, accepted prompt lines are appended to
the configured history file. Lines with obvious secret words are skipped.
