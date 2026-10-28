// Unit tests for new domain ports (TASK-002).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle I: Zero External Dependencies.

use trustkit_core::entities::{
    AstAuditVerdict, ContextSliceRequest, ContextSliceResult,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{AstAuditorPort, ContextSlicerPort};

struct MockAstAuditor;

impl AstAuditorPort for MockAstAuditor {
    fn audit_source_tree(
        &self,
        _files: &[String],
        _reference_seals: &[trustkit_core::entities::ArtifactSeal],
    ) -> Result<AstAuditVerdict, DomainViolation> {
        Ok(AstAuditVerdict::CleanPass { violations: vec![] })
    }
}

struct MockContextSlicer;

impl ContextSlicerPort for MockContextSlicer {
    fn slice_context(
        &self,
        request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation> {
        Ok(ContextSliceResult {
            sliced_content: "pub fn mock() {}".to_string(),
            preserved_symbols: request.symbol_filters.clone(),
            total_lines: 1,
        })
    }
}

#[test]
fn test_mock_ast_auditor_port() {
    let auditor = MockAstAuditor;
    let verdict = auditor
        .audit_source_tree(&["src/lib.rs".to_string()], &[])
        .expect("mock audit should succeed");
    assert!(verdict.is_pass());
}

#[test]
fn test_mock_context_slicer_port() {
    let slicer = MockContextSlicer;
    let req = ContextSliceRequest {
        target_files: vec!["src/lib.rs".to_string()],
        symbol_filters: vec!["mock".to_string()],
        max_line_budget: 50,
    };
    let res = slicer.slice_context(&req).expect("mock slice should succeed");
    assert_eq!(res.preserved_symbols, vec!["mock".to_string()]);
    assert_eq!(res.total_lines, 1);
}
