// Unit tests for Project Knowledge Memory domain entities and ports (TASK-001).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle I: Zero External Dependencies (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use crate::errors::DomainViolation;
use crate::ports::KnowledgeRepositoryPort;

struct InMemoryKnowledgeRepo {
    docs: std::sync::Mutex<Vec<KnowledgeDocument>>,
}

impl InMemoryKnowledgeRepo {
    fn new() -> Self {
        Self {
            docs: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl KnowledgeRepositoryPort for InMemoryKnowledgeRepo {
    fn write_document(
        &self,
        document: &KnowledgeDocument,
    ) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        KnowledgeDocument::validate_slug(&document.slug)?;
        let mut guard = self.docs.lock().unwrap();
        guard.retain(|d| d.slug != document.slug);
        guard.push(document.clone());
        Ok(KnowledgeWriteOutcome::Success {
            slug: document.slug.clone(),
            file_path: format!(".trustkit/wiki/{}.md", document.slug),
            blake3_digest: document.blake3_digest.clone(),
            indexed_links_count: 0,
        })
    }

    fn read_document(&self, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        KnowledgeDocument::validate_slug(slug)?;
        let guard = self.docs.lock().unwrap();
        match guard.iter().find(|d| d.slug == slug) {
            Some(doc) => Ok(KnowledgeLookupOutcome::Found(doc.clone())),
            None => Ok(KnowledgeLookupOutcome::DocumentNotFound),
        }
    }

    fn search_documents(
        &self,
        query: &KnowledgeSearchQuery,
    ) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        let guard = self.docs.lock().unwrap();
        let needle = query.query_text.to_lowercase();
        let mut hits = Vec::new();
        for doc in guard.iter() {
            if doc.title.to_lowercase().contains(&needle)
                || doc.body_markdown.to_lowercase().contains(&needle)
            {
                hits.push(KnowledgeSearchHit {
                    slug: doc.slug.clone(),
                    title: doc.title.clone(),
                    bm25_score: 1.5,
                    snippet: doc.body_markdown.clone(),
                    adjacent_links: vec![],
                });
            }
            if hits.len() >= query.limit {
                break;
            }
        }
        Ok(KnowledgeSearchOutcome::from_hits(hits))
    }

    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        let guard = self.docs.lock().unwrap();
        Ok(guard.clone())
    }

    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        let guard = self.docs.lock().unwrap();
        Ok(KnowledgeRebuildSummary {
            notes_restored: guard.len(),
            links_restored: 0,
            duration_ms: 1,
        })
    }
}

#[test]
fn test_knowledge_document_preserves_attributes_without_nulls() {
    let doc = KnowledgeDocument::new(
        "adrs/adr-002-storage-strategy",
        "Estrategia de Armazenamento",
        vec![
            "adr".to_string(),
            "storage".to_string(),
            "sqlite".to_string(),
        ],
        "Decisao de persistencia usando [[adrs/adr-001-architecture]].",
        "2026-10-29T10:00:00Z",
        "blake3_digest_hex_002",
    );

    assert_eq!(doc.slug, "adrs/adr-002-storage-strategy");
    assert_eq!(doc.title, "Estrategia de Armazenamento");
    assert_eq!(doc.tags.len(), 3);
    assert!(doc.has_tag("sqlite"));
    assert!(!doc.has_tag("postgres"));
    assert!(doc.body_markdown.contains("[[adrs/adr-001-architecture]]"));
    assert_eq!(doc.updated_at, "2026-10-29T10:00:00Z");
    assert_eq!(doc.blake3_digest, "blake3_digest_hex_002");
}

#[test]
fn test_wiki_link_and_search_query_construction() {
    let link = WikiLink::new("specs/spec-005", "tasks/TASK-001");
    assert_eq!(link.source_slug, "specs/spec-005");
    assert_eq!(link.target_slug, "tasks/TASK-001");

    let query = KnowledgeSearchQuery::new("sqlite wal", 5)
        .with_tags(vec!["adr".to_string(), "storage".to_string()]);
    assert_eq!(query.query_text, "sqlite wal");
    assert_eq!(query.limit, 5);
    assert_eq!(query.required_tags, vec!["adr", "storage"]);
}

#[test]
fn test_special_case_empty_knowledge_list_serializes_without_null() {
    let empty_outcome = KnowledgeSearchOutcome::from_hits(vec![]);
    assert_eq!(empty_outcome, KnowledgeSearchOutcome::EmptyKnowledgeList);
    assert!(empty_outcome.is_empty());
    assert_eq!(empty_outcome.as_slice().len(), 0);
    assert_eq!(empty_outcome.variant_label(), "EMPTY_KNOWLEDGE_LIST");
    assert_eq!(empty_outcome.as_json_array(), "[]");

    let json = empty_outcome.to_json();
    assert!(json.contains("\"outcome\":\"EMPTY_KNOWLEDGE_LIST\""));
    assert!(json.contains("\"hits\":[]"));
    assert!(!json.contains("null"));
}

#[test]
fn test_knowledge_search_outcome_with_results_serializes_without_null() {
    let hit = KnowledgeSearchHit {
        slug: "adrs/adr-002-storage-strategy".to_string(),
        title: "Estrategia de Armazenamento".to_string(),
        bm25_score: 2.75,
        snippet: "Persistencia deterministica em <mark>SQLite</mark> FTS5".to_string(),
        adjacent_links: vec!["adrs/adr-001-architecture".to_string()],
    };

    let outcome = KnowledgeSearchOutcome::from_hits(vec![hit]);
    assert!(!outcome.is_empty());
    assert_eq!(outcome.as_slice().len(), 1);
    assert_eq!(outcome.variant_label(), "RESULTS");

    let json = outcome.to_json();
    assert!(json.contains("\"outcome\":\"RESULTS\""));
    assert!(json.contains("adrs/adr-002-storage-strategy"));
    assert!(!json.contains("null"));
}

#[test]
fn test_knowledge_lookup_outcome_special_cases() {
    let not_found = KnowledgeLookupOutcome::DocumentNotFound;
    assert!(!not_found.is_found());
    assert!(not_found.as_document().is_none());
    assert_eq!(not_found.variant_label(), "DOCUMENT_NOT_FOUND");

    let doc = KnowledgeDocument::new(
        "specs/spec-005",
        "Spec 005",
        vec!["spec".to_string()],
        "Conteudo",
        "2026-10-29T10:00:00Z",
        "hash123",
    );
    let found = KnowledgeLookupOutcome::Found(doc.clone());
    assert!(found.is_found());
    assert_eq!(found.as_document(), Some(&doc));
    assert_eq!(found.variant_label(), "FOUND");
}

#[test]
fn test_slug_validation_rejects_path_traversal_and_empty_slug() {
    assert!(KnowledgeDocument::validate_slug("adrs/adr-001").is_ok());
    assert!(KnowledgeDocument::validate_slug("../secrets").is_err());
    assert!(KnowledgeDocument::validate_slug("adrs/../../etc/passwd").is_err());
    assert!(KnowledgeDocument::validate_slug("/absolute/path").is_err());
    assert!(KnowledgeDocument::validate_slug("").is_err());
}

#[test]
fn test_knowledge_repository_port_contract() {
    let repo = InMemoryKnowledgeRepo::new();
    let doc = KnowledgeDocument::new(
        "adrs/adr-002-storage-strategy",
        "Estrategia de Armazenamento",
        vec!["adr".to_string()],
        "SQLite FTS5 com indice derivado reconstruivel",
        "2026-10-29T10:00:00Z",
        "digest_abc",
    );

    let write_res = repo.write_document(&doc).expect("write should succeed");
    assert_eq!(
        write_res,
        KnowledgeWriteOutcome::Success {
            slug: "adrs/adr-002-storage-strategy".to_string(),
            file_path: ".trustkit/wiki/adrs/adr-002-storage-strategy.md".to_string(),
            blake3_digest: "digest_abc".to_string(),
            indexed_links_count: 0,
        }
    );

    let lookup = repo
        .read_document("adrs/adr-002-storage-strategy")
        .expect("read should succeed");
    assert_eq!(lookup, KnowledgeLookupOutcome::Found(doc));

    let missing = repo
        .read_document("adrs/inexistente")
        .expect("read missing should return special case");
    assert_eq!(missing, KnowledgeLookupOutcome::DocumentNotFound);

    let search_hit = repo
        .search_documents(&KnowledgeSearchQuery::new("reconstruivel", 5))
        .expect("search should succeed");
    assert_eq!(search_hit.as_slice().len(), 1);

    let search_empty = repo
        .search_documents(&KnowledgeSearchQuery::new("termo-ausente", 5))
        .expect("empty search should succeed");
    assert_eq!(search_empty, KnowledgeSearchOutcome::EmptyKnowledgeList);

    let list = repo.list_documents().expect("list should succeed");
    assert_eq!(list.len(), 1);

    let rebuild = repo.rebuild_index().expect("rebuild should succeed");
    assert_eq!(rebuild.notes_restored, 1);
}
