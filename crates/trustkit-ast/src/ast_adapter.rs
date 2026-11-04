// AST Layer 0 unified adapter implementation (TASK-009).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use trustkit_core::entities::{
    ArtifactSeal, AstAuditVerdict, ContextSliceRequest, ContextSliceResult,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{AstAuditorPort, ContextSlicerPort};

use crate::context_slicer::AstContextSlicer;
use crate::lexical_scanner::InvariantLexicalScanner;
use crate::test_integrity_auditor::TestIntegrityAuditor;

#[derive(Debug, Default, Clone, Copy)]
pub struct TreeSitterAstAuditorAdapter {
    lexical_scanner: InvariantLexicalScanner,
    test_auditor: TestIntegrityAuditor,
    context_slicer: AstContextSlicer,
}

impl TreeSitterAstAuditorAdapter {
    pub const fn new() -> Self {
        Self {
            lexical_scanner: InvariantLexicalScanner::new(),
            test_auditor: TestIntegrityAuditor::new(),
            context_slicer: AstContextSlicer::new(),
        }
    }
}

impl AstAuditorPort for TreeSitterAstAuditorAdapter {
    fn audit_source_tree(
        &self,
        files: &[String],
        reference_seals: &[ArtifactSeal],
    ) -> Result<AstAuditVerdict, DomainViolation> {
        let mut all_violations = Vec::new();

        for file_path in files {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(err) => {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "ast_auditor".to_string(),
                        details: format!("Unable to read file '{file_path}': {err}"),
                    });
                }
            };

            let lexical_violations = self.lexical_scanner.scan_content(file_path, &content);
            all_violations.extend(lexical_violations);

            let matching_seal = reference_seals.iter().find(|seal| {
                seal.file_path == *file_path
                    || file_path.ends_with(&seal.file_path)
                    || seal.file_path.ends_with(file_path)
            });

            let test_violations = self
                .test_auditor
                .audit_file(file_path, &content, matching_seal);
            all_violations.extend(test_violations);
        }

        if all_violations.is_empty() {
            Ok(AstAuditVerdict::CleanPass {
                violations: Vec::new(),
            })
        } else {
            Ok(AstAuditVerdict::Rejected {
                violations: all_violations,
            })
        }
    }
}

impl ContextSlicerPort for TreeSitterAstAuditorAdapter {
    fn slice_context(
        &self,
        request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation> {
        let mut combined_content = String::new();
        let mut all_preserved = Vec::new();
        let mut total_lines = 0;

        for file_path in &request.target_files {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(err) => {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "ast_context_slicer".to_string(),
                        details: format!("Unable to read target file '{file_path}': {err}"),
                    });
                }
            };

            let single_slice = self.context_slicer.slice_code(file_path, &content, request);
            if !combined_content.is_empty() {
                combined_content.push('\n');
            }
            combined_content.push_str(&single_slice.sliced_content);
            all_preserved.extend(single_slice.preserved_symbols);
            total_lines += single_slice.total_lines;
        }

        all_preserved.sort();
        all_preserved.dedup();

        Ok(ContextSliceResult {
            sliced_content: combined_content,
            preserved_symbols: all_preserved,
            total_lines,
        })
    }
}
