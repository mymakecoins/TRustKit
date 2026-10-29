// Unit & Integration tests for Gate 4 (Task Complete) Knowledge Ingestion Hook (TASK-006).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::sync::Mutex;
use trustkit_core::entities::AtomicTask;
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeRebuildSummary, KnowledgeSearchHit,
    KnowledgeSearchOutcome, KnowledgeSearchQuery, KnowledgeWriteOutcome, WikiLink,
};
use trustkit_core::ports::KnowledgeRepositoryPort;
use trustkit_core::special_cases::{TddLifecycleState, WorktreeAllocation};
use trustkit_core::use_cases::ingest_completed_task_knowledge::{
    self, IngestCompletedTaskKnowledgeRequest,
};

struct InMemoryKnowledgeRepo {
    docs: Mutex<Vec<KnowledgeDocument>>,
    links: Mutex<Vec<WikiLink>>,
}

impl InMemoryKnowledgeRepo {
    fn new() -> Self {
        Self { docs: Mutex::new(Vec::new()), links: Mutex::new(Vec::new()) }
    }

    fn extract_links(source_slug: &str, body: &str) -> Vec<WikiLink> {
        let (mut out, mut rem) = (Vec::new(), body);
        while let Some(after) = rem.find("[[").map(|s| &rem[s + 2..]) {
            let Some(end) = after.find("]]") else { break };
            let target = after[..end].split(['|', '#']).next().unwrap_or("").trim().trim_matches('/').trim_end_matches(".md");
            if !target.is_empty() {
                let link = WikiLink::new(source_slug, target);
                if !out.contains(&link) {
                    out.push(link);
                }
            }
            rem = &after[end + 2..];
        }
        out
    }
}

impl KnowledgeRepositoryPort for InMemoryKnowledgeRepo {
    fn write_document(&self, document: &KnowledgeDocument) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        let clean_slug = KnowledgeDocument::validate_slug(&document.slug)?;
        let digest = if document.blake3_digest.is_empty() {
            "f0e1d2c3b4a50123456789abcdef0123456789abcdef0123456789abcdef0123".to_string()
        } else {
            document.blake3_digest.clone()
        };
        let extracted = Self::extract_links(&clean_slug, &document.body_markdown);
        let stored = KnowledgeDocument::new(
            clean_slug.clone(),
            document.title.clone(),
            document.tags.clone(),
            document.body_markdown.clone(),
            document.updated_at.clone(),
            digest.clone(),
        );

        let mut docs = self.docs.lock().unwrap();
        docs.retain(|d| d.slug != clean_slug);
        docs.push(stored);

        let mut links = self.links.lock().unwrap();
        links.retain(|l| l.source_slug != clean_slug);
        let count = extracted.len();
        links.extend(extracted);

        Ok(KnowledgeWriteOutcome::Success {
            slug: clean_slug.clone(),
            file_path: format!(".trustkit/wiki/{clean_slug}.md"),
            blake3_digest: digest,
            indexed_links_count: count,
        })
    }

    fn read_document(&self, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        let clean_slug = KnowledgeDocument::validate_slug(slug)?;
        let docs = self.docs.lock().unwrap();
        Ok(docs.iter().find(|d| d.slug == clean_slug).cloned().map_or(KnowledgeLookupOutcome::DocumentNotFound, KnowledgeLookupOutcome::Found))
    }

    fn search_documents(&self, query: &KnowledgeSearchQuery) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        let (docs, links) = (self.docs.lock().unwrap(), self.links.lock().unwrap());
        let terms: Vec<String> = query.query_text.split_whitespace().map(|t| t.to_ascii_lowercase()).collect();
        let mut hits = Vec::new();
        for doc in docs.iter() {
            let haystack = format!("{} {} {} {}", doc.slug, doc.title, doc.tags.join(" "), doc.body_markdown).to_ascii_lowercase();
            if terms.iter().any(|t| haystack.contains(t)) {
                let adj = links.iter().filter(|l| l.source_slug == doc.slug).map(|l| l.target_slug.clone()).collect();
                hits.push(KnowledgeSearchHit {
                    slug: doc.slug.clone(),
                    title: doc.title.clone(),
                    bm25_score: 5.0,
                    snippet: doc.body_markdown.lines().next().unwrap_or("").to_string(),
                    adjacent_links: adj,
                });
            }
        }
        Ok(KnowledgeSearchOutcome::from_hits(hits))
    }

    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        Ok(self.docs.lock().unwrap().clone())
    }

    fn query_adjacent_links(&self, slug: &str) -> Result<Vec<WikiLink>, DomainViolation> {
        let clean_slug = KnowledgeDocument::validate_slug(slug)?;
        Ok(self.links.lock().unwrap().iter().filter(|l| l.source_slug == clean_slug || l.target_slug == clean_slug).cloned().collect())
    }

    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        Ok(KnowledgeRebuildSummary {
            notes_restored: self.docs.lock().unwrap().len(),
            links_restored: self.links.lock().unwrap().len(),
            duration_ms: 1,
        })
    }
}

fn sample_task(state: TddLifecycleState) -> AtomicTask {
    AtomicTask {
        cycle_id: "005-project-knowledge-memory".to_string(),
        task_id: "TASK-006".to_string(),
        title: "Implementar Caso de Uso de Ingestao Automatica no Gate 4".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 4,
        depends_on: vec!["TASK-004".to_string()],
        expected_files: vec!["crates/trustkit-core/src/use_cases/ingest_completed_task_knowledge.rs".to_string()],
        target_test_file: "crates/trustkit-core/tests/sdd_gate4_knowledge_hook_tests.rs".to_string(),
        tdd_lifecycle_state: state,
        red_evidence_id: 11,
        green_evidence_id: 12,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
    }
}

#[test]
fn test_gate4_ingests_green_verified_task_and_makes_searchable_by_file_or_task_id() {
    let repo = InMemoryKnowledgeRepo::new();
    let task = sample_task(TddLifecycleState::GreenVerified { red_evidence_id: 11, green_evidence_id: 12 });

    let req = IngestCompletedTaskKnowledgeRequest {
        task,
        commit_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        red_evidence_summary: "EV-11: test failed with exit code 101 as expected in RED phase".to_string(),
        green_evidence_summary: "EV-12: 2 passed, 0 failed in GREEN phase".to_string(),
        now_iso: "2026-10-29T14:20:00Z".to_string(),
    };

    let outcome = ingest_completed_task_knowledge::execute(req, &repo)
        .expect("Gate 4 task ingestion must succeed");

    assert_eq!(outcome.task_id, "TASK-006");
    assert_eq!(outcome.slug, "tasks/TASK-006");
    assert_eq!(outcome.file_path, ".trustkit/wiki/tasks/TASK-006.md");
    assert_eq!(outcome.spec_wikilink, "specs/spec-005-project-knowledge-memory");

    let lookup = repo.read_document("tasks/TASK-006").expect("read task document");
    let doc = lookup.as_document().expect("task document must exist");
    assert_eq!(doc.tags, vec!["task", "tdd-verified", "CORE"]);
    assert!(doc.body_markdown.contains("[[specs/spec-005-project-knowledge-memory]]"));
    assert!(doc.body_markdown.contains("[[tasks/TASK-004]]"));
    assert!(doc.body_markdown.contains("crates/trustkit-core/src/use_cases/ingest_completed_task_knowledge.rs"));
    assert!(doc.body_markdown.contains("abcdef1234567890abcdef1234567890abcdef12"));

    let by_file = repo.search_documents(&KnowledgeSearchQuery::new("ingest_completed_task_knowledge.rs", 5)).expect("search by file");
    assert!(!by_file.is_empty());
    assert_eq!(by_file.as_slice()[0].slug, "tasks/TASK-006");
    assert!(by_file.as_slice()[0].adjacent_links.contains(&"specs/spec-005-project-knowledge-memory".to_string()));

    let by_id = repo.search_documents(&KnowledgeSearchQuery::new("TASK-006", 5)).expect("search by task id");
    assert!(!by_id.is_empty());
    assert_eq!(by_id.as_slice()[0].slug, "tasks/TASK-006");
}

#[test]
fn test_gate4_rejects_task_without_green_or_committed_verification() {
    let repo = InMemoryKnowledgeRepo::new();
    let unverified_task = sample_task(TddLifecycleState::RedVerified { failing_evidence_id: 11 });

    let req = IngestCompletedTaskKnowledgeRequest {
        task: unverified_task,
        commit_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        red_evidence_summary: "EV-11: red".to_string(),
        green_evidence_summary: String::new(),
        now_iso: "2026-10-29T14:20:00Z".to_string(),
    };

    let err = ingest_completed_task_knowledge::execute(req, &repo)
        .expect_err("task without green verification must fail");
    assert_eq!(err.posix_exit_code(), 3);
}
