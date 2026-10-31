// Unit tests for ComposeSpecContextUseCase (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    ContextSliceRequest, ContextSliceResult, KnowledgeDocument, KnowledgeLookupOutcome,
    KnowledgeRebuildSummary, KnowledgeSearchHit, KnowledgeSearchOutcome, KnowledgeSearchQuery,
    KnowledgeWriteOutcome,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{ContextSlicerPort, KnowledgeRepositoryPort};
use trustkit_core::use_cases::compose_spec_context::{self, ComposeSpecContextRequest};

struct MockKnowledgeRepo {
    hits: Vec<KnowledgeSearchHit>,
}

impl KnowledgeRepositoryPort for MockKnowledgeRepo {
    fn write_document(
        &self,
        _document: &KnowledgeDocument,
    ) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        unimplemented!()
    }

    fn read_document(&self, _slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        Ok(KnowledgeLookupOutcome::DocumentNotFound)
    }

    fn search_documents(
        &self,
        _query: &KnowledgeSearchQuery,
    ) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        Ok(KnowledgeSearchOutcome::from_hits(self.hits.clone()))
    }

    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        Ok(vec![])
    }

    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        unimplemented!()
    }
}

struct MockSlicer;

impl ContextSlicerPort for MockSlicer {
    fn slice_context(
        &self,
        _request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation> {
        Ok(ContextSliceResult {
            sliced_content: "pub fn compose() -> bool { true }".to_string(),
            preserved_symbols: vec!["compose".to_string()],
            total_lines: 1,
        })
    }
}

#[test]
fn test_compose_spec_context_validation_rejects_empty_idea() {
    let err = ComposeSpecContextRequest::new("   ", 2, 5).unwrap_err();
    match err {
        DomainViolation::AstIntegrityOrInvariantViolation { invariant_id, .. } => {
            assert_eq!(invariant_id, "RN-002");
        }
        other => panic!("Expected AstIntegrityOrInvariantViolation, got {other:?}"),
    }
}

#[test]
fn test_compose_spec_context_success_aggregates_wiki_and_ast_snippets() {
    let repo = MockKnowledgeRepo {
        hits: vec![KnowledgeSearchHit {
            slug: "arch-core".to_string(),
            title: "Especificacao e Arquitetura".to_string(),
            bm25_score: 1.25,
            snippet: "Documento de arquitetura do nucleo.".to_string(),
            adjacent_links: vec!["crates/trustkit-core/src/lib.rs".to_string()],
        }],
    };
    let slicer = MockSlicer;

    let req = ComposeSpecContextRequest::new("Especificacao", 2, 5).expect("valid request");
    let outcome = compose_spec_context::execute(req, &repo, &slicer).expect("compose success");

    assert_eq!(outcome.wiki_articles.len(), 1);
    assert_eq!(outcome.wiki_articles[0].slug, "arch-core");
    assert_eq!(outcome.ast_slices.len(), 1);
    assert!(outcome.aggregated_context.contains("=== Contexto Relevante Extraido ==="));
    assert!(outcome.aggregated_context.contains("[[arch-core]]"));
}
