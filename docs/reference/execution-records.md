# Execution Records

Execution records are opt-in local NDJSON. They are meant for debugging,
auditing local command behaviour, and future local automation. They are not
remote telemetry.

Enable them in configuration:

```ini
[records]
enabled = true
path = C:\Users\you\AppData\Roaming\milner\records.ndjson
```

If `path` is omitted, the default is:

```text
%APPDATA%\milner\records.ndjson
```

## Schema Version

Current serialized shape:

```json
{
  "schema_version": 1,
  "started_unix_ms": 1780644058860,
  "ended_unix_ms": 1780644059024,
  "cwd": "C:\\work",
  "plan_kind": "command",
  "commands": [
    {
      "program": "powershell",
      "args": ["-NoProfile", "-Command", "exit 7"],
      "stdin": { "kind": "inherit" },
      "stdout": { "kind": "inherit" },
      "stderr": { "kind": "inherit" },
      "resolved_executable": "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
    }
  ],
  "status": { "kind": "success", "exit_code": 7 },
  "policy": [
    "no_cmd_fallback",
    "batch_targets_rejected",
    "explicit_executable_resolution",
    "narrow_stdio_handles",
    "job_object_cleanup_boundary"
  ]
}
```

Each line is one complete JSON object.

## Top-Level Fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | number | Record schema version. Current value is `1`. |
| `started_unix_ms` | number | Start time in Unix milliseconds. |
| `ended_unix_ms` | number or null | End time in Unix milliseconds. |
| `cwd` | string | Working directory Milner used for the execution. |
| `plan_kind` | string | `command` or `pipeline`. |
| `commands` | array | Parsed command structures. |
| `status` | object or null | Success or error status. |
| `policy` | array of strings | Policy facts applied during execution. |

## Command Fields

| Field | Type | Meaning |
| --- | --- | --- |
| `program` | string | Parsed program text. |
| `args` | array of strings | Parsed arguments. |
| `stdin` | object | `inherit` or file input. |
| `stdout` | object | `inherit` or file output. |
| `stderr` | object | `inherit` or file output. |
| `resolved_executable` | string or null | Resolved executable path, when resolution succeeded. |

File outputs include `append: true` or `append: false`.

## Status Fields

Success:

```json
{ "kind": "success", "exit_code": 0 }
```

Error:

```json
{ "kind": "error", "exit_code": 125, "message": "executable `tool` not found" }
```

## Privacy Boundaries

Records do not include environment values, stdout content, stderr content, or
remote telemetry.

Records are skipped entirely when the parsed program or any parsed argument
contains these case-insensitive words:

```text
password
passwd
secret
token
apikey
api_key
credential
```

Persistence failures print a warning to stderr and do not change a successful
child command's exit status.
