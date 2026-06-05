# Command Syntax

The syntax is deliberately small. Unsupported shell forms are rejected instead
of being routed through `cmd.exe`.

## Words And Quotes

| Form | Behaviour |
| --- | --- |
| `cargo --version` | Parses bare words separated by whitespace. |
| `"C:\Program Files\tool.exe"` | Double quotes group spaces. |
| `tool ""` | Empty quoted arguments are preserved. |
| `tool "say \"hello\""` | `\"` inside quotes becomes a literal quote. |
| `tool "C:\\path"` | `\\` inside quotes becomes a literal backslash. |

Backslash escaping is only special inside double quotes when escaping `"` or
`\`. Other backslashes are preserved.

## Redirection

| Form | Behaviour |
| --- | --- |
| `< file` | Reads stdin from `file`. |
| `> file` | Writes stdout to `file`, replacing existing content. |
| `>> file` | Appends stdout to `file`. |
| `2> file` | Writes stderr to `file`, replacing existing content. |
| `2>> file` | Appends stderr to `file`. |

Redirection paths are parsed as words. Quote paths that contain spaces.

## Pipelines

Exactly one pipe is supported:

```text
left command | right command
```

The left command's stdout is connected to the right command's stdin. Longer
pipeline chains are rejected.

## Rejected Syntax

Rejected forms:

| Form | Reason |
| --- | --- |
| `&&` | No shell sequencing. |
| `||` | No conditional shell sequencing. |
| `;` | No command list syntax. |
| `` `command` `` | No command substitution. |
| `$(command)` | No command substitution. |
| variables | No Milner variable expansion. |
| globs | No Milner wildcard expansion. |
| `.bat`, `.cmd` | Batch targets remain unsupported. |

If a child program such as PowerShell expands its own variables or syntax after
launch, that behaviour belongs to the child program.
