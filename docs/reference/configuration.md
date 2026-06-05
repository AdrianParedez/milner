# Configuration

Configuration is optional. By default, the lookup path is:

```text
%APPDATA%\milner\config.toml
```

Pass `--config <file>` to load a specific file. Pass `--no-config` to ignore
configuration entirely.

The format is intentionally strict. Unknown sections, unknown keys, malformed
sections, malformed key-value pairs, and invalid values are rejected with a file
path and line number.

## Example

```ini
[prompt]
text = "milner> "

[history]
enabled = false
path = C:\Users\you\AppData\Roaming\milner\history.txt

[records]
enabled = false
path = C:\Users\you\AppData\Roaming\milner\records.ndjson

[aliases]
gs = git status
```

## `[prompt]`

| Key | Value | Default |
| --- | --- | --- |
| `text` | Non-empty text | `milner> ` |

Quoted values cannot contain embedded quotes.

## `[history]`

| Key | Value | Default |
| --- | --- | --- |
| `enabled` | `true` or `false` | `false` |
| `path` | File path | `%APPDATA%\milner\history.txt` |

History applies only to prompt mode. Lines containing these case-insensitive
words are skipped:

```text
password
passwd
secret
token
apikey
api_key
credential
```

## `[records]`

| Key | Value | Default |
| --- | --- | --- |
| `enabled` | `true` or `false` | `false` |
| `path` | File path | `%APPDATA%\milner\records.ndjson` |

Records are local newline-delimited JSON. See
[execution records](execution-records.md).

## `[aliases]`

Aliases map one command name to one typed command:

```ini
[aliases]
gs = git status
psdate = powershell -NoProfile -Command "Get-Date"
```

Alias names cannot be empty and cannot contain whitespace or these characters:

```text
\ / : | < >
```

Alias values cannot include redirection or pipelines. Expansion cannot bypass
executable resolution or `.bat` and `.cmd` rejection. Cycles are reported
explicitly.
