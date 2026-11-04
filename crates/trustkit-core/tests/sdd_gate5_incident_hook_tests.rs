// Unit & Integration tests for Gate 5 (Gauntlet & Doctor) Incident Ingestion Hook (TASK-007).
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
use trustkit_core::use_cases::ingest_gauntlet_incident_knowledge::{
    self, IngestGauntletIncidentKnowledgeRequest, sanitize_incident_output,
};

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
            "9988776655440123456789abcdef0123456789abcdef0123456789abcdef0123".to_string()
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
                    bm25_score: 4.8,
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
fn test_gate5_ingests_gauntlet_incident_with_sanitization_and_spec_wikilink() {
    let repo = InMemoryKnowledgeRepo::new();
    let raw_output = "\
Critic STATIC_SECURITY failed on crates/trustkit-db/src/sqlite_adapter.rs:42
Used GITHUB_TOKEN=ghp_SecretTokenValue1234567890abcdef and Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
Also API_KEY=sk-proj-9876543210abcdef1234567890abcdef failed connection to https://admin:supersecretpass@db.internal.local/prod";

    let req = IngestGauntletIncidentKnowledgeRequest {
        cycle_id: "005-project-knowledge-memory".to_string(),
        attempt: 5,
        circuit_breaker_tripped: true,
        violated_invariants: vec!["RN-003".to_string(), "RN-007".to_string()],
        raw_diagnostic_output: raw_output.to_string(),
        remediation_hint:
            "Circuit breaker tripped after 5 consecutive failures. Run trustkit doctor.".to_string(),
        now_iso: "2026-10-29T14:30:00Z".to_string(),
    };

    let outcome = ingest_gauntlet_incident_knowledge::execute(req, &repo)
        .expect("Gate 5 incident ingestion must succeed");

    assert_eq!(outcome.cycle_id, "005-project-knowledge-memory");
    assert_eq!(outcome.attempt, 5);
    assert_eq!(
        outcome.slug,
        "incidents/incident-005-project-knowledge-memory-attempt-5"
    );
    assert_eq!(
        outcome.file_path,
        ".trustkit/wiki/incidents/incident-005-project-knowledge-memory-attempt-5.md"
    );
    assert_eq!(
        outcome.spec_wikilink,
        "specs/spec-005-project-knowledge-memory"
    );

    assert!(
        !outcome
            .sanitized_diagnostic
            .contains("ghp_SecretTokenValue")
    );
    assert!(
        !outcome
            .sanitized_diagnostic
            .contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9")
    );
    assert!(
        !outcome
            .sanitized_diagnostic
            .contains("sk-proj-9876543210abcdef")
    );
    assert!(!outcome.sanitized_diagnostic.contains("supersecretpass"));
    assert!(outcome.sanitized_diagnostic.contains("[REDACTED]"));

    let lookup = repo
        .read_document("incidents/incident-005-project-knowledge-memory-attempt-5")
        .expect("read incident note");
    let doc = lookup.as_document().expect("incident document must exist");
    assert_eq!(doc.tags, vec!["incident", "gauntlet", "post-mortem"]);
    assert!(
        doc.body_markdown
            .contains("[[specs/spec-005-project-knowledge-memory]]")
    );
    assert!(doc.body_markdown.contains("RN-003"));
    assert!(doc.body_markdown.contains("RN-007"));
    assert!(!doc.body_markdown.contains("ghp_SecretTokenValue"));
    assert!(!doc.body_markdown.contains("supersecretpass"));

    let search_res = repo
        .search_documents(&KnowledgeSearchQuery::new("STATIC_SECURITY RN-007", 5))
        .expect("search incident");
    assert!(!search_res.is_empty());
    let hit = &search_res.as_slice()[0];
    assert_eq!(
        hit.slug,
        "incidents/incident-005-project-knowledge-memory-attempt-5"
    );
    assert!(
        hit.adjacent_links
            .contains(&"specs/spec-005-project-knowledge-memory".to_string())
    );
}

#[test]
fn test_sanitize_incident_output_redacts_common_credential_patterns() {
    let sample = "token=gho_abc123XYZ789 password=\"my_secret_pw\" AKIAIOSFODNN7EXAMPLE sk-ant-api03-secret123";
    let clean = sanitize_incident_output(sample);
    assert!(!clean.contains("gho_abc123XYZ789"));
    assert!(!clean.contains("my_secret_pw"));
    assert!(!clean.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(!clean.contains("sk-ant-api03-secret123"));
    assert!(clean.contains("[REDACTED]"));
}
