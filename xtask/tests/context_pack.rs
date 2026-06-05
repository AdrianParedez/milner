use std::fs;
use std::path::Path;

use milner_xtask::context_pack::{
    ContextPack, ContextPackConfig, generate_context_pack, validate_context_pack,
};

#[test]
fn manifest_sources_are_ordered_and_classified() {
    let root = fixture_root("ordered");
    write_file(&root, "README.md", "# Milner\n\nRoot overview.\n");
    write_file(
        &root,
        "docs/reference/stability.md",
        "# Stability\n\n## Security Invariants\n\nNo telemetry.\n",
    );
    write_file(
        &root,
        "tests/execution_policy.rs",
        "#![cfg(windows)]\n\n#[test]\nfn bare_names_use_path() {}\n",
    );
    write_file(
        &root,
        "architecture-decision-corpus/docs/design/command-parser.md",
        "# Command Parser Design\n\n## Behaviour\n\nTyped parsing.\n",
    );

    let pack = generate_context_pack(&ContextPackConfig::new(root.clone())).unwrap();

    let paths: Vec<&str> = pack
        .manifest
        .sources
        .iter()
        .map(|source| source.relative_path.as_str())
        .collect();
    assert_eq!(
        paths,
        vec![
            "README.md",
            "architecture-decision-corpus/docs/design/command-parser.md",
            "docs/reference/stability.md",
            "tests/execution_policy.rs",
        ]
    );

    assert_eq!(pack.manifest.sources[0].category, "readme");
    assert_eq!(pack.manifest.sources[1].category, "specification");
    assert_eq!(pack.manifest.sources[2].category, "documentation");
    assert_eq!(pack.manifest.sources[3].category, "test");
}

#[test]
fn citation_ids_survive_when_content_is_unchanged() {
    let root = fixture_root("citation-stability");
    write_file(&root, "README.md", "# Milner\n\nRoot overview.\n");

    let first = generate_context_pack(&ContextPackConfig::new(root.clone())).unwrap();
    write_file(&root, "README.md", "# Milner\n\nRoot overview.\n");
    let second = generate_context_pack(&ContextPackConfig::new(root)).unwrap();

    assert_eq!(
        first.manifest.sources[0].citation_id,
        second.manifest.sources[0].citation_id
    );
}

#[test]
fn written_pack_is_identical_when_sources_are_unchanged() {
    let root = fixture_root("deterministic");
    write_file(
        &root,
        "docs/user-guide/quick-start.md",
        "# Quick Start\n\n## Verification\n\nRun cargo test.\n",
    );

    let first = generate_context_pack(&ContextPackConfig::new(root.clone())).unwrap();
    first.write_to_disk().unwrap();
    let first_manifest = fs::read_to_string(root.join(".milner/manifest.json")).unwrap();
    let first_index = fs::read_to_string(root.join(".milner/index.md")).unwrap();
    let first_citations = fs::read_to_string(root.join(".milner/citations.json")).unwrap();

    let second = generate_context_pack(&ContextPackConfig::new(root.clone())).unwrap();
    second.write_to_disk().unwrap();

    assert_eq!(
        first_manifest,
        fs::read_to_string(root.join(".milner/manifest.json")).unwrap()
    );
    assert_eq!(
        first_index,
        fs::read_to_string(root.join(".milner/index.md")).unwrap()
    );
    assert_eq!(
        first_citations,
        fs::read_to_string(root.join(".milner/citations.json")).unwrap()
    );
}

#[test]
fn summaries_are_traceable_to_manifest_citations() {
    let root = fixture_root("summary-traceability");
    write_file(
        &root,
        "docs/security/security-model.md",
        "# Security Model\n\n## Security Invariants\n\nNo cmd fallback.\n",
    );

    let pack = generate_context_pack(&ContextPackConfig::new(root)).unwrap();

    for summary in &pack.summaries {
        assert!(summary.trace_citation_ids.len() == 1);
        assert!(
            pack.manifest
                .contains_citation(&summary.trace_citation_ids[0])
        );
        assert!(summary.structured_text.contains("Trace:"));
    }
}

#[test]
fn summary_paths_are_written_under_summaries_directory() {
    let root = fixture_root("summary-directory");
    write_file(&root, "README.md", "# Milner\n\nRoot overview.\n");

    let pack = generate_context_pack(&ContextPackConfig::new(root)).unwrap();

    assert_eq!(
        pack.summaries[0].summary_path,
        ".milner/summaries/README_md.md"
    );
}

#[test]
fn validation_rejects_broken_citation_references() {
    let mut pack = ContextPack::empty_for_test();
    pack.manifest
        .sources
        .push(milner_xtask::context_pack::SourceEntry {
            citation_id: "milner:sha256:abcdef123456".to_string(),
            relative_path: "README.md".to_string(),
            content_hash: "sha256:abcdef123456".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            freshness: "2026-01-01T00:00:00Z".to_string(),
            file_size: 10,
            category: "readme".to_string(),
        });
    pack.citations.entries.insert(
        "milner:sha256:missing".to_string(),
        milner_xtask::context_pack::CitationEntry {
            citation_id: "milner:sha256:missing".to_string(),
            relative_path: "README.md".to_string(),
            content_hash: "sha256:missing".to_string(),
        },
    );

    let error = validate_context_pack(&pack).unwrap_err();

    assert!(error.to_string().contains("unknown citation"));
}

fn fixture_root(name: &str) -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("milner-context-pack-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}
