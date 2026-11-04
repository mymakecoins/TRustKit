// Handler for `trustkit memory` subcommands (TASK-008).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    MemoryArgs, MemoryListArgs, MemoryReadArgs, MemoryRebuildArgs, MemorySearchArgs,
    MemorySubcommand, MemoryWriteArgs,
};
use crate::error_handler::DomainOutcome;
use serde::Serialize;
use serde_json::json;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeSearchOutcome, KnowledgeSearchQuery,
    KnowledgeWriteOutcome,
};
use trustkit_core::ports::KnowledgeRepositoryPort;
use trustkit_db::KnowledgeWikiFsRepository;

#[derive(Debug, Serialize)]
pub struct JsonMemoryWriteResponse {
    pub verdict: String,
    pub outcome: String,
    pub slug: String,
    pub file_path: String,
    pub blake3_digest: String,
    pub indexed_links_count: usize,
}

#[derive(Debug, Serialize)]
pub struct JsonMemorySearchHit {
    pub slug: String,
    pub title: String,
    pub bm25_score: f64,
    pub snippet: String,
    pub adjacent_links: Vec<String>,
}

pub fn execute(args: &MemoryArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    execute_in_root(args, json_mode, Path::new("."))
}

pub fn execute_in_root(
    args: &MemoryArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let repo = KnowledgeWikiFsRepository::open_from_root(repo_root)?;
    execute_with_repo(args, json_mode, &repo)
}

pub fn execute_with_repo(
    args: &MemoryArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        MemorySubcommand::Write(w) => handle_write(w, json_mode, repo),
        MemorySubcommand::Read(r) => handle_read(r, json_mode, repo),
        MemorySubcommand::Search(s) => handle_search(s, json_mode, repo),
        MemorySubcommand::List(l) => handle_list(l, json_mode, repo),
        MemorySubcommand::Rebuild(rb) => handle_rebuild(rb, json_mode, repo),
    }
}

fn clean_tags(raw: &[String]) -> Vec<String> {
    raw.iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn handle_write(
    args: &MemoryWriteArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    let ts = format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let doc = KnowledgeDocument::new(
        &args.slug,
        &args.title,
        clean_tags(&args.tags),
        &args.content,
        ts,
        String::new(),
    );
    let KnowledgeWriteOutcome::Success {
        slug,
        file_path,
        blake3_digest,
        indexed_links_count,
    } = repo.write_document(&doc)?;

    if json_mode {
        let payload = JsonMemoryWriteResponse {
            verdict: "PASS".to_string(),
            outcome: "SUCCESS".to_string(),
            slug,
            file_path,
            blake3_digest,
            indexed_links_count,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!(
            "[PASS] Knowledge document written: {slug} ({file_path}) [BLAKE3: {blake3_digest}] (links: {indexed_links_count})"
        );
    }
    Ok(DomainOutcome::Success)
}

fn handle_read(
    args: &MemoryReadArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    match repo.read_document(&args.slug)? {
        KnowledgeLookupOutcome::Found(doc) => {
            let mut adjacent_links = Vec::new();
            for l in repo.query_adjacent_links(&doc.slug)? {
                let other = if l.source_slug == doc.slug {
                    l.target_slug
                } else {
                    l.source_slug
                };
                if !other.is_empty() && other != doc.slug && !adjacent_links.contains(&other) {
                    adjacent_links.push(other);
                }
            }
            if json_mode {
                let payload = json!({
                    "verdict": "PASS",
                    "outcome": "FOUND",
                    "document": {
                        "slug": doc.slug, "title": doc.title, "tags": doc.tags,
                        "body_markdown": doc.body_markdown, "updated_at": doc.updated_at,
                        "blake3_digest": doc.blake3_digest, "adjacent_links": adjacent_links,
                    }
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
            } else {
                println!(
                    "[PASS] Knowledge Document: {}\nTitle: {}\nTags: {}\nUpdated: {}\n\n{}",
                    doc.slug,
                    doc.title,
                    doc.tags.join(", "),
                    doc.updated_at,
                    doc.body_markdown
                );
            }
        }
        KnowledgeLookupOutcome::DocumentNotFound => {
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&json!({"verdict": "PASS", "outcome": "DOCUMENT_NOT_FOUND", "slug": args.slug})).unwrap_or_default());
            } else {
                println!(
                    "[PASS] [DOCUMENT_NOT_FOUND] No document for slug: {}",
                    args.slug
                );
            }
        }
    }
    Ok(DomainOutcome::Success)
}

fn handle_search(
    args: &MemorySearchArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    let query =
        KnowledgeSearchQuery::new(&args.query, args.limit).with_tags(clean_tags(&args.tags));
    let outcome = repo.search_documents(&query)?;
    let hits: Vec<JsonMemorySearchHit> = outcome
        .as_slice()
        .iter()
        .map(|h| JsonMemorySearchHit {
            slug: h.slug.clone(),
            title: h.title.clone(),
            bm25_score: h.bm25_score,
            snippet: h.snippet.clone(),
            adjacent_links: h.adjacent_links.clone(),
        })
        .collect();

    if json_mode {
        let payload = json!({"verdict": "PASS", "outcome": outcome.variant_label(), "query": args.query, "hits": hits});
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else if matches!(outcome, KnowledgeSearchOutcome::EmptyKnowledgeList) {
        println!(
            "[PASS] [EMPTY_KNOWLEDGE_LIST] No hits for query: {}",
            args.query
        );
    } else {
        for h in &hits {
            println!(
                "[PASS] {} ({:.4}) - {}: {}",
                h.slug, h.bm25_score, h.title, h.snippet
            );
        }
    }
    Ok(DomainOutcome::Success)
}

fn handle_list(
    args: &MemoryListArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    let req_tags = clean_tags(&args.tags);
    let docs: Vec<serde_json::Value> = repo
        .list_documents()?
        .into_iter()
        .filter(|d| req_tags.iter().all(|req| d.tags.iter().any(|t| t.eq_ignore_ascii_case(req))))
        .map(|d| json!({"slug": d.slug, "title": d.title, "tags": d.tags, "updated_at": d.updated_at, "blake3_digest": d.blake3_digest}))
        .collect();
    let outcome_label = if docs.is_empty() {
        "EMPTY_KNOWLEDGE_LIST"
    } else {
        "RESULTS"
    };
    if json_mode {
        let payload = json!({"verdict": "PASS", "outcome": outcome_label, "total": docs.len(), "documents": docs});
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!(
            "[PASS] [{}] {} knowledge documents",
            outcome_label,
            docs.len()
        );
    }
    Ok(DomainOutcome::Success)
}

fn handle_rebuild(
    _args: &MemoryRebuildArgs,
    json_mode: bool,
    repo: &dyn KnowledgeRepositoryPort,
) -> Result<DomainOutcome, DomainViolation> {
    let s = repo.rebuild_index()?;
    if json_mode {
        let payload = json!({"verdict": "PASS", "outcome": "REBUILT", "notes_restored": s.notes_restored, "links_restored": s.links_restored, "duration_ms": s.duration_ms});
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!(
            "[PASS] Knowledge index rebuilt: {} notes and {} links restored in {}ms",
            s.notes_restored, s.links_restored, s.duration_ms
        );
    }
    Ok(DomainOutcome::Success)
}
