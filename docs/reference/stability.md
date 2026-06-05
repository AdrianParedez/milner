# Stability Contract

Milner is still pre-1.0, so some edges will move. Even so, the documented CLI
should not feel like wet paint. If a behaviour is listed here, treat it as part
of the product promise unless a release note says otherwise.

## Public API

The stable API is deliberately small:

- the `milner.exe` command-line interface
- the command syntax accepted by `--line` and prompt mode
- the configuration format
- the execution-record schema
- documented exit-code behaviour
- documented security policy

The Rust modules are not a library contract. Generated Rust docs are useful for
reading the implementation, but they do not promise that internal names or
module boundaries will stay put.

## Stable Surface

Patch releases should preserve these behaviours:

| Surface | Contract |
| --- | --- |
| CLI forms | Direct command form, `--line`, and `--prompt` stay supported. |
| Options | `--no-config`, `--config`, `--cwd`, `--set-env`, `--unset-env`, and `--timeout-ms` keep their documented meaning. |
| `--pty` | The option stays reserved and explicitly unsupported until terminal hosting is deliberately designed. |
| Command syntax | Words, double quotes, stdin redirection, stdout redirection, stderr redirection, and one two-command external pipeline stay supported. |
| Prompt built-ins | `cd`, `pwd`, `complete`, and `exit` stay prompt built-ins. |
| Configuration | `[prompt]`, `[history]`, `[records]`, and `[aliases]` stay the supported sections. |
| Records schema | `schema_version: 1` records stay newline-delimited JSON with documented top-level fields. |
| Exit codes | Documented runner-level exit codes retain their meanings. |
| Batch policy | `.bat` and `.cmd` targets stay rejected. |
| Resolution policy | Bare executable names search `PATH`, not the child current directory. |
| Handle policy | Child processes receive only intended stdin, stdout, and stderr handles. |
| Telemetry policy | Milner does not send telemetry. |

## Experimental Surface

Some details are intentionally less fixed before v1.0:

- exact wording of non-security error messages
- completion output formatting beyond one suggestion per line
- ordering or wording of execution-record `policy` labels
- generated Rust documentation structure
- prompt display details that are not part of command semantics

If one of these changes in a way a user would notice, it belongs in release
notes.

## Unsupported Surface

These are outside the current product contract:

- `.bat` and `.cmd` execution
- implicit `cmd.exe` fallback
- arbitrary shell syntax such as `&&`, `||`, and `;`
- Milner-side variable expansion
- glob expansion
- command substitution
- arbitrary-length pipelines
- descriptor duplication such as `2>&1`
- background jobs
- ConPTY pseudoconsole hosting
- sandboxing untrusted code
- remote telemetry
- a stable Rust library API

Unsupported does not mean "coming soon." It means Milner should reject or avoid
the behaviour instead of half-supporting it.

## Versioning Rules

While Milner is pre-1.0:

- patch releases preserve documented behaviour except for security fixes
  or narrowly scoped bug fixes;
- minor releases may add compatible features or refine the pre-1.0 contract;
- breaking changes are called out in release notes.

After v1.0, breaking the stable surface requires a major version bump.

## Security Invariants

These rules are not convenience settings. They are the point of the tool:

- unsupported syntax must not fall back to `cmd.exe`;
- `.bat` and `.cmd` support must not be added without a dedicated policy and
  tests;
- handle inheritance must remain narrow and explicit;
- child-only environment changes must not mutate the parent process
  environment;
- local history and records must remain opt-in;
- remote telemetry must not be introduced silently.

## Deprecation Policy

Before v1.0, deprecated behaviour may move faster than it would in a mature
product, but documented changes still need to be visible in release notes.
After v1.0, removing or changing stable behaviour should go through a
deprecation period unless the change is required for security.

## Audit Evidence

This contract is not just prose. It is backed by source and tests:

| Contract area | Evidence |
| --- | --- |
| CLI forms and options | `src/process/mod.rs`, `tests/execution_policy.rs` |
| Parser syntax and rejected operators | `src/process/parser.rs`, unit tests in that module |
| Redirection and pipelines | `tests/redirection_pipeline.rs` |
| Prompt built-ins and prompt recovery | `src/process/prompt.rs`, `tests/prompt_regression.rs` |
| Configuration and aliases | `src/process/config.rs`, `tests/config_ergonomics.rs` |
| Execution records | `src/process/records.rs`, `tests/execution_records.rs` |
| Batch rejection and handle boundaries | `src/process/command_line.rs`, `tests/security_regression.rs` |
| Timeouts and interrupt cleanup | `src/process/mod.rs`, `src/process/win32.rs`, `tests/foreground_cancellation.rs` |

If the public docs and implementation disagree, one of them is wrong. The next
change should make them agree again.
