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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSliceKind {
    VerticalTracerBullet,
    WideRefactorExpand,
    WideRefactorMigrateBatch,
    WideRefactorContract,
}

impl TaskSliceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VerticalTracerBullet => "VERTICAL_TRACER_BULLET",
            Self::WideRefactorExpand => "WIDE_REFACTOR_EXPAND",
            Self::WideRefactorMigrateBatch => "WIDE_REFACTOR_MIGRATE_BATCH",
            Self::WideRefactorContract => "WIDE_REFACTOR_CONTRACT",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "WIDE_REFACTOR_EXPAND" | "EXPAND" => Self::WideRefactorExpand,
            "WIDE_REFACTOR_MIGRATE_BATCH" | "MIGRATE" | "MIGRATE_BATCH" => {
                Self::WideRefactorMigrateBatch
            }
            "WIDE_REFACTOR_CONTRACT" | "CONTRACT" => Self::WideRefactorContract,
            _ => Self::VerticalTracerBullet,
        }
    }

    pub fn is_wide_refactor(&self) -> bool {
        !matches!(self, Self::VerticalTracerBullet)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskBlastRadiusReport {
    pub task_id: String,
    pub slice_kind: TaskSliceKind,
    pub blast_radius_score: u32,
    pub in_degree: usize,
    pub touched_high_degree_nodes: Vec<String>,
}

impl TaskBlastRadiusReport {
    pub fn new(
        task_id: String,
        slice_kind: TaskSliceKind,
        blast_radius_score: u32,
        in_degree: usize,
        touched_high_degree_nodes: Vec<String>,
    ) -> Self {
        Self {
            task_id,
            slice_kind,
            blast_radius_score,
            in_degree,
            touched_high_degree_nodes,
        }
    }

    pub fn is_high_risk(&self) -> bool {
        self.blast_radius_score >= 8 || self.in_degree >= 8
    }
}

#[cfg(test)]
#[path = "ast_entities_tests.rs"]
mod tests;
