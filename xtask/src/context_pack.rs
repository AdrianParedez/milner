use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Write as _};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

const OUTPUT_DIR: &str = ".milner";
const SUMMARY_DIR: &str = ".milner/summaries";

#[derive(Debug, Clone)]
pub struct ContextPackConfig {
    root: PathBuf,
}

impl ContextPackConfig {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Debug, Clone)]
pub struct ContextPack {
    root: PathBuf,
    pub manifest: Manifest,
    pub citations: Citations,
    pub summaries: Vec<SummaryEntry>,
}

impl ContextPack {
    pub fn empty_for_test() -> Self {
        Self {
            root: PathBuf::new(),
            manifest: Manifest::empty(),
            citations: Citations::empty(),
            summaries: Vec::new(),
        }
    }

    pub fn write_to_disk(&self) -> Result<(), ContextPackError> {
        validate_context_pack(self)?;

        let output_dir = self.root.join(OUTPUT_DIR);
        let summary_dir = self.root.join(SUMMARY_DIR);
        fs::create_dir_all(&summary_dir)?;

        remove_stale_summary_files(&summary_dir, &self.summaries)?;

        fs::write(output_dir.join("manifest.json"), self.manifest.to_json())?;
        fs::write(output_dir.join("citations.json"), self.citations.to_json())?;
        fs::write(output_dir.join("index.md"), render_index(self))?;
        fs::write(
            output_dir.join("manifest-schema.md"),
            manifest_schema_markdown(),
        )?;
        fs::write(
            output_dir.join("citation-format.md"),
            citation_format_markdown(),
        )?;

        for summary in &self.summaries {
            fs::write(
                self.root.join(&summary.summary_path),
                summary.structured_text.as_bytes(),
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub schema_version: u8,
    pub generated_at: String,
    pub generator: String,
    pub source_selection: SourceSelection,
    pub sources: Vec<SourceEntry>,
}

impl Manifest {
    fn empty() -> Self {
        Self {
            schema_version: 1,
            generated_at: "1970-01-01T00:00:00Z".to_string(),
            generator: "milner-xtask context-pack v1".to_string(),
            source_selection: SourceSelection::default(),
            sources: Vec::new(),
        }
    }

    pub fn contains_citation(&self, citation_id: &str) -> bool {
        self.sources
            .iter()
            .any(|source| source.citation_id == citation_id)
    }

    fn to_json(&self) -> String {
        let mut out = String::new();
        writeln!(&mut out, "{{").unwrap();
        writeln!(&mut out, "  \"schema_version\": {},", self.schema_version).unwrap();
        writeln!(
            &mut out,
            "  \"generated_at\": \"{}\",",
            json_escape(&self.generated_at)
        )
        .unwrap();
        writeln!(
            &mut out,
            "  \"generator\": \"{}\",",
            json_escape(&self.generator)
        )
        .unwrap();
        out.push_str("  \"source_selection\": ");
        out.push_str(&self.source_selection.to_json(2));
        out.push_str(",\n");
        writeln!(&mut out, "  \"sources\": [").unwrap();
        for (index, source) in self.sources.iter().enumerate() {
            out.push_str(&source.to_json(4));
            if index + 1 != self.sources.len() {
                out.push(',');
            }
            out.push('\n');
        }
        writeln!(&mut out, "  ]").unwrap();
        writeln!(&mut out, "}}").unwrap();
        out
    }
}

#[derive(Debug, Clone)]
pub struct SourceSelection {
    pub included_roots: Vec<String>,
    pub excluded_corpus_patterns: Vec<String>,
}

impl Default for SourceSelection {
    fn default() -> Self {
        Self {
            included_roots: vec![
                "README.md".to_string(),
                "docs/**".to_string(),
                "tests/**".to_string(),
                "architecture-decision-corpus/docs/checklists/*.md".to_string(),
                "architecture-decision-corpus/docs/design/*.md".to_string(),
                "architecture-decision-corpus/docs/milestones/*.md".to_string(),
                "architecture-decision-corpus/docs/research/*.md".to_string(),
            ],
            excluded_corpus_patterns: vec![
                "architecture-decision-corpus/docs/design/draft-readme.md".to_string(),
                "architecture-decision-corpus/docs/design/public-readme-release-draft.md"
                    .to_string(),
                "architecture-decision-corpus/docs/checklists/security-patch-v0.6.1-acceptance.md"
                    .to_string(),
                "architecture-decision-corpus/docs/milestones/13-security-patch-v0.6.1.md"
                    .to_string(),
                "architecture-decision-corpus/docs/templates/**".to_string(),
                "architecture-decision-corpus/milner/**".to_string(),
            ],
        }
    }
}

impl SourceSelection {
    fn to_json(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let inner = " ".repeat(indent + 2);
        let mut out = String::new();
        writeln!(&mut out, "{{").unwrap();
        write_string_array(
            &mut out,
            &inner,
            "included_roots",
            &self.included_roots,
            true,
        );
        write_string_array(
            &mut out,
            &inner,
            "excluded_corpus_patterns",
            &self.excluded_corpus_patterns,
            false,
        );
        write!(&mut out, "{pad}}}").unwrap();
        out
    }
}

#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub citation_id: String,
    pub relative_path: String,
    pub content_hash: String,
    pub last_modified: String,
    pub freshness: String,
    pub file_size: u64,
    pub category: String,
}

impl SourceEntry {
    fn to_json(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let inner = " ".repeat(indent + 2);
        let mut out = String::new();
        writeln!(&mut out, "{pad}{{").unwrap();
        write_json_string_field(&mut out, &inner, "citation_id", &self.citation_id, true);
        write_json_string_field(&mut out, &inner, "relative_path", &self.relative_path, true);
        write_json_string_field(&mut out, &inner, "content_hash", &self.content_hash, true);
        write_json_string_field(&mut out, &inner, "last_modified", &self.last_modified, true);
        write_json_string_field(&mut out, &inner, "freshness", &self.freshness, true);
        writeln!(&mut out, "{inner}\"file_size\": {},", self.file_size).unwrap();
        write_json_string_field(&mut out, &inner, "category", &self.category, false);
        write!(&mut out, "{pad}}}").unwrap();
        out
    }
}

#[derive(Debug, Clone)]
pub struct Citations {
    pub schema_version: u8,
    pub citation_format: String,
    pub entries: BTreeMap<String, CitationEntry>,
}

impl Citations {
    fn empty() -> Self {
        Self {
            schema_version: 1,
            citation_format: "milner:sha256:<content-prefix>:<path-prefix>".to_string(),
            entries: BTreeMap::new(),
        }
    }

    fn to_json(&self) -> String {
        let mut out = String::new();
        writeln!(&mut out, "{{").unwrap();
        writeln!(&mut out, "  \"schema_version\": {},", self.schema_version).unwrap();
        writeln!(
            &mut out,
            "  \"citation_format\": \"{}\",",
            json_escape(&self.citation_format)
        )
        .unwrap();
        writeln!(&mut out, "  \"entries\": {{").unwrap();
        for (index, (citation_id, entry)) in self.entries.iter().enumerate() {
            writeln!(&mut out, "    \"{}\": {{", json_escape(citation_id)).unwrap();
            write_json_string_field(&mut out, "      ", "citation_id", &entry.citation_id, true);
            write_json_string_field(
                &mut out,
                "      ",
                "relative_path",
                &entry.relative_path,
                true,
            );
            write_json_string_field(
                &mut out,
                "      ",
                "content_hash",
                &entry.content_hash,
                false,
            );
            write!(&mut out, "    }}").unwrap();
            if index + 1 != self.entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        writeln!(&mut out, "  }}").unwrap();
        writeln!(&mut out, "}}").unwrap();
        out
    }
}

#[derive(Debug, Clone)]
pub struct CitationEntry {
    pub citation_id: String,
    pub relative_path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone)]
pub struct SummaryEntry {
    pub citation_id: String,
    pub relative_path: String,
    pub category: String,
    pub summary_path: String,
    pub trace_citation_ids: Vec<String>,
    pub structured_text: String,
}

#[derive(Debug)]
pub enum ContextPackError {
    Io(io::Error),
    Validation(String),
}

impl fmt::Display for ContextPackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Validation(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for ContextPackError {}

impl From<io::Error> for ContextPackError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn generate_context_pack(config: &ContextPackConfig) -> Result<ContextPack, ContextPackError> {
    let source_paths = discover_source_paths(&config.root)?;
    let mut source_inputs = Vec::new();
    let mut latest_modified = UNIX_EPOCH;

    for relative_path in source_paths {
        let absolute_path = config.root.join(&relative_path);
        let metadata = fs::metadata(&absolute_path)?;
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        latest_modified = latest_modified.max(modified);
        let bytes = fs::read(&absolute_path)?;
        source_inputs.push(SourceInput {
            relative_path,
            bytes,
            modified,
            file_size: metadata.len(),
        });
    }

    let generated_at = format_system_time(latest_modified);
    let mut sources = Vec::new();
    let mut citation_entries = BTreeMap::new();
    let mut summaries = Vec::new();

    for source in source_inputs {
        let relative_text = path_to_slash(&source.relative_path);
        let content_hash = format!("sha256:{}", sha256_hex(&source.bytes));
        let citation_id = citation_id_for(&relative_text, &content_hash);
        let category = category_for(&relative_text);
        let entry = SourceEntry {
            citation_id: citation_id.clone(),
            relative_path: relative_text.clone(),
            content_hash: content_hash.clone(),
            last_modified: format_system_time(source.modified),
            freshness: generated_at.clone(),
            file_size: source.file_size,
            category: category.clone(),
        };
        sources.push(entry);
        citation_entries.insert(
            citation_id.clone(),
            CitationEntry {
                citation_id: citation_id.clone(),
                relative_path: relative_text.clone(),
                content_hash,
            },
        );
        summaries.push(summarize_source(
            &relative_text,
            &category,
            &citation_id,
            &String::from_utf8_lossy(&source.bytes),
        ));
    }

    let pack = ContextPack {
        root: config.root.clone(),
        manifest: Manifest {
            schema_version: 1,
            generated_at,
            generator: "milner-xtask context-pack v1".to_string(),
            source_selection: SourceSelection::default(),
            sources,
        },
        citations: Citations {
            schema_version: 1,
            citation_format: "milner:sha256:<content-prefix>:<path-prefix>".to_string(),
            entries: citation_entries,
        },
        summaries,
    };

    validate_context_pack(&pack)?;
    Ok(pack)
}

pub fn validate_context_pack(pack: &ContextPack) -> Result<(), ContextPackError> {
    let mut manifest_ids = BTreeSet::new();
    let mut manifest_paths = BTreeSet::new();

    for source in &pack.manifest.sources {
        if !manifest_ids.insert(source.citation_id.clone()) {
            return Err(ContextPackError::Validation(format!(
                "duplicate citation `{}`",
                source.citation_id
            )));
        }
        if !manifest_paths.insert(source.relative_path.clone()) {
            return Err(ContextPackError::Validation(format!(
                "duplicate source path `{}`",
                source.relative_path
            )));
        }
    }

    for (citation_id, entry) in &pack.citations.entries {
        if citation_id != &entry.citation_id {
            return Err(ContextPackError::Validation(format!(
                "citation key `{citation_id}` does not match entry id `{}`",
                entry.citation_id
            )));
        }
        let Some(source) = pack
            .manifest
            .sources
            .iter()
            .find(|source| source.citation_id == *citation_id)
        else {
            return Err(ContextPackError::Validation(format!(
                "unknown citation `{citation_id}`"
            )));
        };
        if source.relative_path != entry.relative_path || source.content_hash != entry.content_hash
        {
            return Err(ContextPackError::Validation(format!(
                "citation `{citation_id}` does not match manifest source"
            )));
        }
    }

    for source in &pack.manifest.sources {
        if !pack.citations.entries.contains_key(&source.citation_id) {
            return Err(ContextPackError::Validation(format!(
                "manifest source `{}` has no citation entry",
                source.relative_path
            )));
        }
    }

    for summary in &pack.summaries {
        if !pack.manifest.contains_citation(&summary.citation_id) {
            return Err(ContextPackError::Validation(format!(
                "summary for `{}` has unknown citation `{}`",
                summary.relative_path, summary.citation_id
            )));
        }
        for citation_id in &summary.trace_citation_ids {
            if !pack.manifest.contains_citation(citation_id) {
                return Err(ContextPackError::Validation(format!(
                    "summary for `{}` references unknown citation `{citation_id}`",
                    summary.relative_path
                )));
            }
        }
    }

    Ok(())
}

struct SourceInput {
    relative_path: PathBuf,
    bytes: Vec<u8>,
    modified: SystemTime,
    file_size: u64,
}

fn discover_source_paths(root: &Path) -> Result<Vec<PathBuf>, ContextPackError> {
    let mut paths = BTreeSet::new();

    add_if_file(root, Path::new("README.md"), &mut paths);
    add_tree(root, Path::new("docs"), &mut paths)?;
    add_tree(root, Path::new("tests"), &mut paths)?;
    add_selected_corpus(root, &mut paths)?;

    Ok(paths.into_iter().collect())
}

fn add_if_file(root: &Path, relative_path: &Path, paths: &mut BTreeSet<PathBuf>) {
    if root.join(relative_path).is_file() {
        paths.insert(relative_path.to_path_buf());
    }
}

fn add_tree(
    root: &Path,
    relative_dir: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<(), ContextPackError> {
    let absolute_dir = root.join(relative_dir);
    if !absolute_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&absolute_dir)? {
        let entry = entry?;
        let child_name = entry.file_name();
        let child_relative = relative_dir.join(child_name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            add_tree(root, &child_relative, paths)?;
        } else if file_type.is_file() {
            paths.insert(child_relative);
        }
    }
    Ok(())
}

fn add_selected_corpus(root: &Path, paths: &mut BTreeSet<PathBuf>) -> Result<(), ContextPackError> {
    for dir in [
        "architecture-decision-corpus/docs/checklists",
        "architecture-decision-corpus/docs/design",
        "architecture-decision-corpus/docs/milestones",
        "architecture-decision-corpus/docs/research",
    ] {
        add_markdown_files(root, Path::new(dir), paths)?;
    }
    Ok(())
}

fn add_markdown_files(
    root: &Path,
    relative_dir: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<(), ContextPackError> {
    let absolute_dir = root.join(relative_dir);
    if !absolute_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(absolute_dir)? {
        let entry = entry?;
        let relative_path = relative_dir.join(entry.file_name());
        if entry.file_type()?.is_file()
            && relative_path.extension().and_then(|value| value.to_str()) == Some("md")
            && include_corpus_path(&path_to_slash(&relative_path))
        {
            paths.insert(relative_path);
        }
    }

    Ok(())
}

fn include_corpus_path(path: &str) -> bool {
    !matches!(
        path,
        "architecture-decision-corpus/docs/design/draft-readme.md"
            | "architecture-decision-corpus/docs/design/public-readme-release-draft.md"
            | "architecture-decision-corpus/docs/checklists/security-patch-v0.6.1-acceptance.md"
            | "architecture-decision-corpus/docs/milestones/13-security-patch-v0.6.1.md"
    )
}

fn category_for(path: &str) -> String {
    if path == "README.md" {
        "readme"
    } else if path.starts_with("tests/") {
        "test"
    } else if path.starts_with("docs/") {
        "documentation"
    } else if path.contains("/design/") || path.contains("/checklists/") {
        "specification"
    } else if path.contains("/milestones/") || path.contains("/research/") {
        "corpus"
    } else {
        "source"
    }
    .to_string()
}

fn summarize_source(
    relative_path: &str,
    category: &str,
    citation_id: &str,
    contents: &str,
) -> SummaryEntry {
    let title = first_heading(contents).unwrap_or(relative_path);
    let headings = heading_lines(contents);
    let tests = test_names(contents);
    let excerpt = first_meaningful_lines(contents, 4);

    let mut structured = String::new();
    writeln!(&mut structured, "# {title}").unwrap();
    writeln!(&mut structured).unwrap();
    writeln!(&mut structured, "- Citation: `{citation_id}`").unwrap();
    writeln!(&mut structured, "- Source: `{relative_path}`").unwrap();
    writeln!(&mut structured, "- Category: `{category}`").unwrap();
    writeln!(&mut structured, "- Trace: `{citation_id}`").unwrap();
    writeln!(&mut structured).unwrap();
    writeln!(&mut structured, "## Structured Summary").unwrap();
    writeln!(
        &mut structured,
        "- Architectural decisions: {}",
        focus_line(contents, &["Design", "Architecture", "Policy", "Boundary"])
    )
    .unwrap();
    writeln!(
        &mut structured,
        "- System behavior: {}",
        focus_line(
            contents,
            &["Behaviour", "Behavior", "Current behaviour", "Supported"]
        )
    )
    .unwrap();
    writeln!(
        &mut structured,
        "- Domain concepts: {}",
        focus_line(contents, &["Purpose", "Scope", "Surface", "Model"])
    )
    .unwrap();
    writeln!(
        &mut structured,
        "- Development workflows: {}",
        focus_line(
            contents,
            &["Verification", "Workflow", "Release", "Acceptance"]
        )
    )
    .unwrap();
    writeln!(
        &mut structured,
        "- Testing expectations: {}",
        if tests.is_empty() {
            focus_line(contents, &["Tests", "Testing", "Acceptance Gates"])
        } else {
            tests.join(", ")
        }
    )
    .unwrap();
    writeln!(&mut structured).unwrap();
    writeln!(&mut structured, "## Source Outline").unwrap();
    if headings.is_empty() {
        writeln!(&mut structured, "- No markdown headings detected.").unwrap();
    } else {
        for heading in headings.into_iter().take(12) {
            writeln!(&mut structured, "- {heading}").unwrap();
        }
    }
    writeln!(&mut structured).unwrap();
    writeln!(&mut structured, "## Evidence Excerpt").unwrap();
    for line in excerpt {
        writeln!(&mut structured, "- {line}").unwrap();
    }

    SummaryEntry {
        citation_id: citation_id.to_string(),
        relative_path: relative_path.to_string(),
        category: category.to_string(),
        summary_path: format!("{}/{}.md", SUMMARY_DIR, safe_summary_name(relative_path)),
        trace_citation_ids: vec![citation_id.to_string()],
        structured_text: structured,
    }
}

fn focus_line(contents: &str, markers: &[&str]) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let normalized = line.trim().trim_start_matches('#').trim();
        if markers.iter().any(|marker| normalized.contains(marker))
            && let Some(next) = lines
                .iter()
                .skip(index + 1)
                .map(|value| value.trim())
                .find(|value| !value.is_empty() && !value.starts_with('#'))
        {
            return clean_summary_line(next);
        }
    }
    first_meaningful_lines(contents, 1)
        .into_iter()
        .next()
        .unwrap_or_else(|| "No durable statement found in source.".to_string())
}

fn first_heading(contents: &str) -> Option<&str> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn heading_lines(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                Some(clean_summary_line(trimmed))
            } else {
                None
            }
        })
        .collect()
}

fn test_names(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("fn "))
        .filter_map(|line| line.split('(').next())
        .filter(|name| !name.is_empty())
        .take(8)
        .map(|name| format!("`{name}`"))
        .collect()
}

fn first_meaningful_lines(contents: &str, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for line in contents.lines().map(str::trim) {
        if line.is_empty()
            || line.starts_with("```")
            || line.starts_with("<div")
            || line.starts_with("</div")
            || line.starts_with("<p")
            || line.starts_with("</p")
        {
            continue;
        }
        lines.push(clean_summary_line(line));
        if lines.len() == limit {
            break;
        }
    }
    if lines.is_empty() {
        lines.push("No non-empty text found in source.".to_string());
    }
    lines
}

fn clean_summary_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_index(pack: &ContextPack) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# Milner Context Pack").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Generated from deterministic repository sources at `{}`.",
        pack.manifest.generated_at
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Sources").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "| Citation | Category | Source | Summary | Content Hash |"
    )
    .unwrap();
    writeln!(&mut out, "| --- | --- | --- | --- | --- |").unwrap();
    for source in &pack.manifest.sources {
        let summary_path = pack
            .summaries
            .iter()
            .find(|summary| summary.citation_id == source.citation_id)
            .map(|summary| summary.summary_path.as_str())
            .unwrap_or("");
        writeln!(
            &mut out,
            "| `{}` | {} | `{}` | [{}]({}) | `{}` |",
            source.citation_id,
            source.category,
            source.relative_path,
            summary_path,
            summary_path
                .strip_prefix(".milner/")
                .unwrap_or(summary_path),
            source.content_hash
        )
        .unwrap();
    }
    out
}

fn manifest_schema_markdown() -> &'static str {
    "# Manifest Schema\n\n\
The context pack manifest is `.milner/manifest.json` with `schema_version` 1.\n\n\
## Top-Level Fields\n\n\
- `schema_version`: integer schema identifier. Current value is `1`.\n\
- `generated_at`: deterministic freshness timestamp for the indexed source set.\n\
- `generator`: generator name and schema family.\n\
- `source_selection`: included roots and explicit exclusions used by the generator.\n\
- `sources`: ordered array of indexed source metadata.\n\n\
## Source Fields\n\n\
- `citation_id`: stable citation identifier for the source.\n\
- `relative_path`: repository-relative path using `/` separators.\n\
- `content_hash`: SHA-256 content hash in `sha256:<hex>` form.\n\
- `last_modified`: filesystem last modified timestamp in UTC.\n\
- `freshness`: pack freshness timestamp copied from the manifest generation timestamp.\n\
- `file_size`: byte length of the source file.\n\
- `category`: deterministic source class such as `readme`, `documentation`, `test`, `corpus`, or `specification`.\n"
}

fn citation_format_markdown() -> &'static str {
    "# Citation Format\n\n\
Milner context citations use this format:\n\n\
```text\n\
milner:sha256:<content-prefix>:<path-prefix>\n\
```\n\n\
- `content-prefix` is the first 16 hexadecimal characters of the source SHA-256 content hash.\n\
- `path-prefix` is the first 12 hexadecimal characters of the SHA-256 hash of the repository-relative path.\n\
- The content prefix keeps citations stable across regenerations when file contents are unchanged.\n\
- The path prefix keeps citations unique when two files have identical contents.\n\n\
Future agents should cite the `citation_id` and use `.milner/citations.json` to resolve it to a path and full content hash.\n"
}

fn remove_stale_summary_files(
    summary_dir: &Path,
    summaries: &[SummaryEntry],
) -> Result<(), ContextPackError> {
    let expected: BTreeSet<String> = summaries
        .iter()
        .filter_map(|summary| {
            Path::new(&summary.summary_path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect();

    for entry in fs::read_dir(summary_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && !expected.contains(&name) {
                fs::remove_file(entry.path())?;
            }
        }
    }

    Ok(())
}

fn write_string_array(
    out: &mut String,
    indent: &str,
    key: &str,
    values: &[String],
    trailing_comma: bool,
) {
    writeln!(out, "{indent}\"{key}\": [").unwrap();
    for (index, value) in values.iter().enumerate() {
        write!(out, "{indent}  \"{}\"", json_escape(value)).unwrap();
        if index + 1 != values.len() {
            out.push(',');
        }
        out.push('\n');
    }
    write!(out, "{indent}]").unwrap();
    if trailing_comma {
        out.push(',');
    }
    out.push('\n');
}

fn write_json_string_field(
    out: &mut String,
    indent: &str,
    key: &str,
    value: &str,
    trailing_comma: bool,
) {
    write!(out, "{indent}\"{key}\": \"{}\"", json_escape(value)).unwrap();
    if trailing_comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value.is_control() => {
                write!(&mut escaped, "\\u{:04x}", value as u32).unwrap();
            }
            value => escaped.push(value),
        }
    }
    escaped
}

fn path_to_slash(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut out, "{byte:02x}").unwrap();
    }
    out
}

fn citation_id_for(relative_path: &str, content_hash: &str) -> String {
    let content_prefix = content_hash
        .strip_prefix("sha256:")
        .unwrap_or(content_hash)
        .chars()
        .take(16)
        .collect::<String>();
    let path_hash = sha256_hex(relative_path.as_bytes());
    let path_prefix = path_hash.chars().take(12).collect::<String>();
    format!("milner:sha256:{content_prefix}:{path_prefix}")
}

fn safe_summary_name(relative_path: &str) -> String {
    relative_path
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn format_system_time(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = seconds_of_day % 3_600 / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
