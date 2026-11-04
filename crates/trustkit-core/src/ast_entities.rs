// AST Layer 0 & Context Slicer Domain Entities (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestIntegrityViolation {
    pub file_path: String,
    pub test_name: String,
    pub expected_assertions: u32,
    pub actual_assertions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstViolation {
    TestDilution(TestIntegrityViolation),
    ForbiddenEmoji {
        file_path: String,
        line: usize,
        character: String,
    },
    AiTrailer {
        file_path: String,
        line: usize,
        trailer: String,
    },
    UntypedAny {
        file_path: String,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstAuditVerdict {
    CleanPass { violations: Vec<AstViolation> },
    Rejected { violations: Vec<AstViolation> },
}

impl AstAuditVerdict {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::CleanPass { .. })
    }

    pub fn violations(&self) -> &[AstViolation] {
        match self {
            Self::CleanPass { violations } => violations,
            Self::Rejected { violations } => violations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSliceRequest {
    pub target_files: Vec<String>,
    pub symbol_filters: Vec<String>,
    pub max_line_budget: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSliceResult {
    pub sliced_content: String,
    pub preserved_symbols: Vec<String>,
    pub total_lines: usize,
}

#[cfg(test)]
#[path = "ast_entities_tests.rs"]
mod tests;
