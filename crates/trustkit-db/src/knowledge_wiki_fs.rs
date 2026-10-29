// Wiki Markdown file repository and deterministic FTS5 index rebuild (TASK-004).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::knowledge_ops;
use crate::sqlite_adapter::SqliteWalRepositoryAdapter;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeRebuildSummary, KnowledgeSearchOutcome,
    KnowledgeSearchQuery, KnowledgeWriteOutcome, WikiLink,
};
use trustkit_core::ports::KnowledgeRepositoryPort;

pub struct KnowledgeWikiFsRepository {
    wiki_dir: PathBuf,
    db: SqliteWalRepositoryAdapter,
}

impl KnowledgeWikiFsRepository {
    pub fn new(wiki_dir: impl Into<PathBuf>, db: SqliteWalRepositoryAdapter) -> Self {
        Self { wiki_dir: wiki_dir.into(), db }
    }

    pub fn open(wiki_dir: &Path, db_path: &Path) -> Result<Self, DomainViolation> {
        Ok(Self::new(wiki_dir.to_path_buf(), SqliteWalRepositoryAdapter::open(db_path)?))
    }

    pub fn open_from_root(repo_root: &Path) -> Result<Self, DomainViolation> {
        let tk = repo_root.join(".trustkit");
        Self::open(&tk.join("wiki"), &tk.join("state.db"))
    }

    pub fn wiki_dir(&self) -> &Path {
        &self.wiki_dir
    }

    pub fn db(&self) -> &SqliteWalRepositoryAdapter {
        &self.db
    }

    pub fn rebuild_from_wiki_directory(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        rebuild_from_wiki_directory(&self.wiki_dir, &self.db)
    }
}

impl KnowledgeRepositoryPort for KnowledgeWikiFsRepository {
    fn write_document(&self, document: &KnowledgeDocument) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        write_wiki_document(&self.wiki_dir, &self.db, document)
    }

    fn read_document(&self, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        read_wiki_document(&self.wiki_dir, slug)
    }

    fn search_documents(&self, query: &KnowledgeSearchQuery) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        self.db.search_knowledge_fts5(query)
    }

    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        list_wiki_documents(&self.wiki_dir)
    }

    fn query_adjacent_links(&self, slug: &str) -> Result<Vec<WikiLink>, DomainViolation> {
        self.db.query_adjacent_links(slug)
    }

    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        rebuild_from_wiki_directory(&self.wiki_dir, &self.db)
    }
}

pub fn write_wiki_document(
    wiki_dir: &Path,
    db: &SqliteWalRepositoryAdapter,
    document: &KnowledgeDocument,
) -> Result<KnowledgeWriteOutcome, DomainViolation> {
    let (clean_slug, target_path) = resolve_safe_wiki_path(wiki_dir, &document.slug)?;
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| boundary_err("Failed creating wiki directory", e))?;
    }

    let rendered = render_markdown(&document.title, &document.tags, &document.updated_at, &document.body_markdown);
    let blake3_digest = blake3::hash(rendered.as_bytes()).to_hex().to_string();

    let tmp_path = target_path.with_extension("md.tmp");
    fs::write(&tmp_path, rendered.as_bytes()).map_err(|e| boundary_err("Failed writing temp wiki file", e))?;
    fs::rename(&tmp_path, &target_path).map_err(|e| boundary_err("Failed renaming wiki file", e))?;

    let clean_body = document.body_markdown.trim().to_string();
    let links = parse_wikilinks(&clean_slug, &clean_body);
    let indexed_doc = KnowledgeDocument::new(
        clean_slug.clone(),
        document.title.clone(),
        document.tags.clone(),
        clean_body,
        document.updated_at.clone(),
        blake3_digest.clone(),
    );
    db.upsert_knowledge_index(&indexed_doc, &links)?;

    Ok(KnowledgeWriteOutcome::Success {
        slug: clean_slug,
        file_path: target_path.to_string_lossy().to_string(),
        blake3_digest,
        indexed_links_count: links.len(),
    })
}

pub fn read_wiki_document(wiki_dir: &Path, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
    let (clean_slug, target_path) = resolve_safe_wiki_path(wiki_dir, slug)?;
    if !target_path.exists() {
        return Ok(KnowledgeLookupOutcome::DocumentNotFound);
    }
    Ok(KnowledgeLookupOutcome::Found(load_document_from_path(&clean_slug, &target_path)?))
}

pub fn list_wiki_documents(wiki_dir: &Path) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
    if !wiki_dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_markdown_files(wiki_dir, wiki_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files.into_iter().map(|(slug, path)| load_document_from_path(&slug, &path)).collect()
}

pub fn rebuild_from_wiki_directory(
    wiki_dir: &Path,
    db: &SqliteWalRepositoryAdapter,
) -> Result<KnowledgeRebuildSummary, DomainViolation> {
    let start = Instant::now();
    let docs = list_wiki_documents(wiki_dir)?;
    let mut conn = db.lock_conn()?;
    let tx = conn.transaction().map_err(|e| boundary_err("Failed starting rebuild transaction", e))?;

    knowledge_ops::clear_knowledge_index(&tx)?;
    let mut links_restored = 0usize;
    for doc in &docs {
        let links = parse_wikilinks(&doc.slug, &doc.body_markdown);
        links_restored += links.len();
        knowledge_ops::upsert_knowledge_index(&tx, doc, &links)?;
    }

    tx.commit().map_err(|e| boundary_err("Failed committing rebuild transaction", e))?;
    Ok(KnowledgeRebuildSummary {
        notes_restored: docs.len(),
        links_restored,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn resolve_safe_wiki_path(wiki_dir: &Path, raw_slug: &str) -> Result<(String, PathBuf), DomainViolation> {
    let clean_slug = KnowledgeDocument::validate_slug(raw_slug)?;
    if clean_slug.split('/').any(|seg| seg.is_empty() || seg == "." || seg == "..") {
        return Err(boundary_err("Unsafe path segment in slug", raw_slug));
    }
    Ok((clean_slug.clone(), wiki_dir.join(format!("{clean_slug}.md"))))
}

fn collect_markdown_files(root: &Path, current: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), DomainViolation> {
    for entry in fs::read_dir(current).map_err(|e| boundary_err("Failed reading wiki directory", e))? {
        let path = entry.map_err(|e| boundary_err("Failed reading directory entry", e))?.path();
        if path.is_dir() {
            collect_markdown_files(root, &path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md")
            && let Ok(rel) = path.strip_prefix(root)
        {
            let slug = rel.to_string_lossy().replace('\\', "/").trim_end_matches(".md").to_string();
            if !slug.is_empty() {
                out.push((slug, path));
            }
        }
    }
    Ok(())
}

fn load_document_from_path(slug: &str, path: &Path) -> Result<KnowledgeDocument, DomainViolation> {
    let raw = fs::read_to_string(path).map_err(|e| boundary_err("Failed reading markdown file", e))?;
    let digest = blake3::hash(raw.as_bytes()).to_hex().to_string();
    let (title, tags, updated_at, body) = parse_frontmatter_and_body(&raw);
    Ok(KnowledgeDocument::new(slug, title, tags, body, updated_at, digest))
}

fn render_markdown(title: &str, tags: &[String], updated_at: &str, body: &str) -> String {
    let esc_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let tags_csv = tags.iter().map(|t| t.trim()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(", ");
    format!("---\ntitle: \"{esc_title}\"\ntags: [{tags_csv}]\nupdated_at: \"{updated_at}\"\n---\n\n{}\n", body.trim())
}

fn parse_frontmatter_and_body(raw: &str) -> (String, Vec<String>, String, String) {
    let norm = raw.trim_start_matches('\u{feff}');
    if let Some(after) = norm.strip_prefix("---\n").or_else(|| norm.strip_prefix("---\r\n"))
        && let Some(end_idx) = after.find("\n---")
    {
        let (fm, rest) = (&after[..end_idx], after[end_idx + 4..].trim_start_matches(['\r', '\n']));
        let (mut title, mut updated_at, mut tags) = (String::new(), String::new(), Vec::new());
        for line in fm.lines() {
            if let Some((k, v)) = line.trim().split_once(':') {
                let val = v.trim().trim_matches(['"', '\'']).replace("\\\"", "\"");
                match k.trim() {
                    "title" => title = val,
                    "updated_at" | "updated" | "date" => updated_at = val,
                    "tags" => {
                        tags = v.trim().trim_matches(['[', ']']).split(',')
                            .map(|s| s.trim().trim_matches(['"', '\'']).to_string())
                            .filter(|s| !s.is_empty()).collect();
                    }
                    _ => {}
                }
            }
        }
        return (title, tags, updated_at, rest.trim().to_string());
    }
    (String::new(), Vec::new(), String::new(), norm.trim().to_string())
}

fn parse_wikilinks(source_slug: &str, body: &str) -> Vec<WikiLink> {
    let (mut links, mut in_fence) = (Vec::new(), false);
    for line in body.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            let mut rem = line;
            while let Some(after) = rem.find("[[").map(|s| &rem[s + 2..]) {
                let Some(e) = after.find("]]") else { break };
                let target = after[..e].split(['|', '#']).next().unwrap_or("").trim().trim_matches('/').trim_end_matches(".md");
                if !target.is_empty() && !target.contains("..") {
                    let link = WikiLink::new(source_slug, target);
                    if !links.contains(&link) {
                        links.push(link);
                    }
                }
                rem = &after[e + 2..];
            }
        }
    }
    links
}

fn boundary_err(ctx: &str, err: impl std::fmt::Display) -> DomainViolation {
    DomainViolation::InfrastructureBoundaryFault { subsystem: "knowledge_wiki_fs".to_string(), details: format!("{ctx}: {err}") }
}
