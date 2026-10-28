// Unit tests for SliceAstContextUseCase (TASK-012).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_core::entities::{ContextSliceRequest, ContextSliceResult};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::ContextSlicerPort;
use trustkit_core::use_cases::slice_ast_context::{self, SliceAstContextRequest};

struct MockSlicer;

impl ContextSlicerPort for MockSlicer {
    fn slice_context(
        &self,
        request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation> {
        Ok(ContextSliceResult {
            sliced_content: "pub struct User { pub id: u64 }".to_string(),
            preserved_symbols: request.symbol_filters.clone(),
            total_lines: 1,
        })
    }
}

#[test]
fn test_slice_ast_context_use_case_success() {
    let slicer = MockSlicer;
    let req = SliceAstContextRequest {
        target_files: vec!["src/user.rs".to_string()],
        symbol_filters: vec!["User".to_string()],
        max_line_budget: 80,
    };

    let outcome = slice_ast_context::execute(req, &slicer)
        .expect("slice use case execution should succeed");

    assert_eq!(outcome.preserved_symbols, vec!["User".to_string()]);
    assert_eq!(outcome.total_lines, 1);
    assert!(outcome.sliced_content.contains("pub struct User"));
}
