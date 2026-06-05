# Generated Rust Docs

Generate local Rust HTML docs with:

```powershell
cargo doc --no-deps --document-private-items
```

Open:

```text
target\doc\milner\index.html
```

Generated Rust docs are useful when reviewing the source, but they describe the
implementation. They are not a stable public library API.

GitHub Actions also builds and uploads the generated docs as the
`milner-rustdoc-html` artifact in the `docs` workflow.

The stable public API is currently:

- the `milner.exe` command-line interface
- the configuration format
- the command syntax accepted by `--line` and prompt mode
- the execution-record schema when records are enabled
- documented exit-code behaviour
