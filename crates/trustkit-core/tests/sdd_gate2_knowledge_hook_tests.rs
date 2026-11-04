// Unit & Integration tests for Gate 2 (Spec Lock) Knowledge Ingestion Hook (TASK-005).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::sync::Mutex;
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeRebuildSummary, KnowledgeSearchHit,
    KnowledgeSearchOutcome, KnowledgeSearchQuery, KnowledgeWriteOutcome, WikiLink,
};
use trustkit_core::ports::KnowledgeRepositoryPort;
use trustkit_core::use_cases::ingest_locked_spec_knowledge::{
    self, IngestLockedSpecKnowledgeRequest,
};
use trustkit_core::use_cases::lock_specification::LockSpecificationOutcome;

struct InMemoryKnowledgeRepo {
    docs: Mutex<Vec<KnowledgeDocument>>,
    links: Mutex<Vec<WikiLink>>,
}

impl InMemoryKnowledgeRepo {
    fn new() -> Self {
        Self {
            docs: Mutex::new(Vec::new()),
            links: Mutex::new(Vec::new()),
        }
    }

    fn extract_links(source_slug: &str, body: &str) -> Vec<WikiLink> {
        let (mut out, mut rem) = (Vec::new(), body);
        while let Some(after) = rem.find("[[").map(|s| &rem[s + 2..]) {
            let Some(end) = after.find("]]") else { break };
            let target = after[..end]
                .split(['|', '#'])
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('/')
                .trim_end_matches(".md");
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
    fn write_document(
        &self,
        document: &KnowledgeDocument,
    ) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        let clean_slug = KnowledgeDocument::validate_slug(&document.slug)?;
        let digest = if document.blake3_digest.is_empty() {
            "a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123".to_string()
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
        Ok(docs.iter().find(|d| d.slug == clean_slug).cloned().map_or(
            KnowledgeLookupOutcome::DocumentNotFound,
            KnowledgeLookupOutcome::Found,
        ))
    }

    fn search_documents(
        &self,
        query: &KnowledgeSearchQuery,
    ) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        let (docs, links) = (self.docs.lock().unwrap(), self.links.lock().unwrap());
        let terms: Vec<String> = query
            .query_text
            .split_whitespace()
            .map(|t| t.to_ascii_lowercase())
            .collect();
        let mut hits = Vec::new();
        for doc in docs.iter() {
            let haystack = format!(
                "{} {} {} {}",
                doc.slug,
                doc.title,
                doc.tags.join(" "),
                doc.body_markdown
            )
            .to_ascii_lowercase();
            if terms.iter().any(|t| haystack.contains(t)) {
                let adj = links
                    .iter()
                    .filter(|l| l.source_slug == doc.slug)
                    .map(|l| l.target_slug.clone())
                    .collect();
                hits.push(KnowledgeSearchHit {
                    slug: doc.slug.clone(),
                    title: doc.title.clone(),
                    bm25_score: 4.5,
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
        Ok(self
            .links
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.source_slug == clean_slug || l.target_slug == clean_slug)
            .cloned()
            .collect())
    }

    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        Ok(KnowledgeRebuildSummary {
            notes_restored: self.docs.lock().unwrap().len(),
            links_restored: self.links.lock().unwrap().len(),
            duration_ms: 1,
        })
    }
}

#[test]
fn test_gate2_spec_lock_ingests_frozen_spec_with_tags_blake3_and_task_wikilinks() {
    let repo = InMemoryKnowledgeRepo::new();
    let lock_outcome = LockSpecificationOutcome {
        cycle_id: "005-project-knowledge-memory".to_string(),
        spec_file_path: "specs/005-project-knowledge-memory/spec.md".to_string(),
        spec_lock_state: "FROZEN".to_string(),
        blake3_digest: "d40f1e6488fcb73977502a02bb9057c906e8b9ff62295af8fbc2a6f537a6d336"
            .to_string(),
        locked_at: "2026-10-29T14:00:00Z".to_string(),
        rules_count: 3,
        requirements_count: 3,
        tasks_count: 3,
        e2e_tests_count: 3,
    };

    let spec_content = "\
# Especificacao: 005-project-knowledge-memory

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-005 | RF-021 | TASK-005 | Node5 | tests/sdd_gate2_knowledge_hook_tests.rs | EV-005.json |
| RN-006 | RF-022 | TASK-006 | Node6 | tests/sdd_gate4_knowledge_hook_tests.rs | EV-006.json |
| RN-007 | RF-023 | TASK-007 | Node7 | tests/sdd_gate5_incident_hook_tests.rs | EV-007.json |
"
    .to_string();

    let req = IngestLockedSpecKnowledgeRequest {
        lock_outcome,
        spec_content,
    };
    let outcome = ingest_locked_spec_knowledge::execute(req, &repo)
        .expect("Gate 2 spec knowledge ingestion must succeed");

    assert_eq!(outcome.cycle_id, "005-project-knowledge-memory");
    assert_eq!(outcome.slug, "specs/spec-005-project-knowledge-memory");
    assert_eq!(
        outcome.file_path,
        ".trustkit/wiki/specs/spec-005-project-knowledge-memory.md"
    );
    assert_eq!(
        outcome.task_links,
        vec![
            "tasks/TASK-005".to_string(),
            "tasks/TASK-006".to_string(),
            "tasks/TASK-007".to_string()
        ]
    );
    assert_eq!(outcome.indexed_links_count, 3);

    let lookup = repo
        .read_document("specs/spec-005-project-knowledge-memory")
        .expect("read spec doc");
    let doc = lookup.as_document().expect("document must exist");
    assert_eq!(
        doc.tags,
        vec!["spec", "frozen", "005-project-knowledge-memory"]
    );
    assert!(
        doc.body_markdown
            .contains("d40f1e6488fcb73977502a02bb9057c906e8b9ff62295af8fbc2a6f537a6d336")
    );
    assert!(doc.body_markdown.contains("[[tasks/TASK-005]]"));
    assert!(doc.body_markdown.contains("[[tasks/TASK-006]]"));
    assert!(doc.body_markdown.contains("[[tasks/TASK-007]]"));

    let search_res = repo
        .search_documents(&KnowledgeSearchQuery::new("RN-005 RF-021", 5))
        .expect("search spec");
    assert!(!search_res.is_empty());
    let hit = &search_res.as_slice()[0];
    assert_eq!(hit.slug, "specs/spec-005-project-knowledge-memory");
    assert!(hit.adjacent_links.contains(&"tasks/TASK-005".to_string()));
    assert!(hit.adjacent_links.contains(&"tasks/TASK-006".to_string()));
    assert!(hit.adjacent_links.contains(&"tasks/TASK-007".to_string()));
}

#[test]
fn test_gate2_spec_lock_rejects_unfrozen_specification() {
    let repo = InMemoryKnowledgeRepo::new();
    let lock_outcome = LockSpecificationOutcome {
        cycle_id: "005-draft".to_string(),
        spec_file_path: "specs/005-draft/spec.md".to_string(),
        spec_lock_state: "DRAFT".to_string(),
        blake3_digest: String::new(),
        locked_at: "2026-10-29T14:00:00Z".to_string(),
        rules_count: 1,
        requirements_count: 1,
        tasks_count: 1,
        e2e_tests_count: 1,
    };

    let req = IngestLockedSpecKnowledgeRequest {
        lock_outcome,
        spec_content: "# Draft spec".to_string(),
    };
    let err = ingest_locked_spec_knowledge::execute(req, &repo)
        .expect_err("unfrozen spec must be rejected");
    assert_eq!(err.posix_exit_code(), 2);
}
