// Slice AST Context Use Case (TASK-012).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::ContextSliceRequest;
use crate::errors::DomainViolation;
use crate::ports::ContextSlicerPort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceAstContextRequest {
    pub target_files: Vec<String>,
    pub symbol_filters: Vec<String>,
    pub max_line_budget: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceAstContextOutcome {
    pub target_files: Vec<String>,
    pub preserved_symbols: Vec<String>,
    pub total_lines: usize,
    pub sliced_content: String,
}

pub fn execute(
    req: SliceAstContextRequest,
    slicer: &dyn ContextSlicerPort,
) -> Result<SliceAstContextOutcome, DomainViolation> {
    let domain_req = ContextSliceRequest {
        target_files: req.target_files.clone(),
        symbol_filters: req.symbol_filters.clone(),
        max_line_budget: req.max_line_budget,
    };

    let result = slicer.slice_context(&domain_req)?;

    Ok(SliceAstContextOutcome {
        target_files: req.target_files,
        preserved_symbols: result.preserved_symbols,
        total_lines: result.total_lines,
        sliced_content: result.sliced_content,
    })
}
