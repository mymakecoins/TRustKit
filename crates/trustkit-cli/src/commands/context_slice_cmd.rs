// Handler for `trustkit context slice` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::ContextSliceArgs;
use crate::error_handler::DomainOutcome;
use serde::Serialize;
use trustkit_ast::TreeSitterAstAuditorAdapter;
use trustkit_core::errors::DomainViolation;
use trustkit_core::use_cases::slice_ast_context::{self, SliceAstContextRequest};

#[derive(Debug, Serialize)]
pub struct JsonContextSliceResponse {
    pub status: String,
    pub target_files: Vec<String>,
    pub preserved_symbols: Vec<String>,
    pub total_lines: usize,
    pub sliced_content: String,
}

pub fn execute(
    args: &ContextSliceArgs,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    let mut targets = args.files.clone();
    if let Some(single) = &args.file
        && !targets.contains(single)
    {
        targets.push(single.clone());
    }

    if targets.is_empty() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "context_slice".to_string(),
            details: "No target file specified. Provide --file <PATH> or --files <PATHS...>".to_string(),
        });
    }

    let slicer = TreeSitterAstAuditorAdapter::new();
    let request = SliceAstContextRequest {
        target_files: targets,
        symbol_filters: args.symbols.clone(),
        max_line_budget: args.budget.unwrap_or(0),
    };

    let outcome = slice_ast_context::execute(request, &slicer)?;

    if json_mode {
        let response = JsonContextSliceResponse {
            status: "CONTEXT_SLICED".to_string(),
            target_files: outcome.target_files,
            preserved_symbols: outcome.preserved_symbols,
            total_lines: outcome.total_lines,
            sliced_content: outcome.sliced_content,
        };
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            println!("{json_str}");
        }
    } else {
        println!("{}", outcome.sliced_content);
    }

    Ok(DomainOutcome::Success)
}
