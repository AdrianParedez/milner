# Manifest Schema

The context pack manifest is `.milner/manifest.json` with `schema_version` 1.

## Top-Level Fields

- `schema_version`: integer schema identifier. Current value is `1`.
- `generated_at`: deterministic freshness timestamp for the indexed source set.
- `generator`: generator name and schema family.
- `source_selection`: included roots and explicit exclusions used by the generator.
- `sources`: ordered array of indexed source metadata.

## Source Fields

- `citation_id`: stable citation identifier for the source.
- `relative_path`: repository-relative path using `/` separators.
- `content_hash`: SHA-256 content hash in `sha256:<hex>` form.
- `last_modified`: filesystem last modified timestamp in UTC.
- `freshness`: pack freshness timestamp copied from the manifest generation timestamp.
- `file_size`: byte length of the source file.
- `category`: deterministic source class such as `readme`, `documentation`, `test`, `corpus`, or `specification`.
