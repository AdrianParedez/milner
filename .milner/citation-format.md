# Citation Format

Milner context citations use this format:

```text
milner:sha256:<content-prefix>:<path-prefix>
```

- `content-prefix` is the first 16 hexadecimal characters of the source SHA-256 content hash.
- `path-prefix` is the first 12 hexadecimal characters of the SHA-256 hash of the repository-relative path.
- The content prefix keeps citations stable across regenerations when file contents are unchanged.
- The path prefix keeps citations unique when two files have identical contents.

Future agents should cite the `citation_id` and use `.milner/citations.json` to resolve it to a path and full content hash.
