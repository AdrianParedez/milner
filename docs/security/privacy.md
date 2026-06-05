# Privacy Notes

No telemetry is sent.

The persistence features are local and off by default. Turn them on only when
you want the files they create.

## History

History applies only to prompt mode. When enabled, accepted prompt lines are
appended to a local file.

Entries containing these obvious secret words are skipped:

```text
password
passwd
secret
token
apikey
api_key
credential
```

This is a conservative filter, not a promise that every secret-shaped value is
caught.

## Execution Records

Records are also disabled by default. When enabled, they are written as local
newline-delimited JSON.

Records include:

- parsed program and arguments
- redirection structure
- resolved executable path when available
- working directory
- timing
- exit or error status
- policy facts

Records do not include:

- environment values
- stdout content
- stderr content
- remote telemetry

If the parsed program or arguments contain the same obvious secret words used
by history, the record is skipped entirely.

## Local File Responsibility

History and records are ordinary local files. Choose storage paths that match
your retention and privacy expectations.
