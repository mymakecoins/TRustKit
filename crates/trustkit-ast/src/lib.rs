//! TRustKit AST: Syntactic inspection and DAG wave topology.

pub mod ast_adapter;
pub mod ast_graph_extractor;
pub mod context_slicer;
pub mod fowler_smells_lexical;
pub mod graph_context_slicer;
pub mod graph_health_auditor;
pub mod knowledge_link_extractor;
pub mod lexical_scanner;
pub mod milestone_citation;
pub mod milestone_extractor;
pub mod module_depth_analyzer;
pub mod prd_ingestor;
pub mod red_test_scaffolder;
pub mod rust_assertion_counter;
pub mod rust_graph_extractor;
pub mod seam_classifier;
pub mod spec_linter;
pub mod spec_matrix_atomizer;
pub mod spec_validator;
pub mod task_parser;
pub mod test_integrity_auditor;
pub mod ts_assertion_counter;
pub mod ts_graph_extractor;
pub mod wave_coupling_analyzer;
pub mod wave_planner;

pub use ast_adapter::TreeSitterAstAuditorAdapter;
pub use ast_graph_extractor::{AstGraphExtractor, detect_layer};
pub use context_slicer::AstContextSlicer;
pub use fowler_smells_lexical::{
    CodeSmellCandidate, FowlerSmellsLexicalEvaluator, RepositoryStandardRule,
};
pub use graph_context_slicer::GraphContextSlicer;
pub use graph_health_auditor::GraphHealthAuditor;
pub use lexical_scanner::InvariantLexicalScanner;
pub use milestone_citation::{
    CitationValidationResult, CitationValidator, FormattedCitationBlock, MilestoneCitationReport,
    ParsedCitation,
};
pub use milestone_extractor::{ExtractedMarkdownBlock, PrdAdrMilestoneExtractor};
pub use module_depth_analyzer::ModuleDepthAnalyzer;
pub use prd_ingestor::PrdSpecificationIngestor;
pub use red_test_scaffolder::{RedTestScaffolder, ScaffoldingTarget};
pub use rust_assertion_counter::RustAssertionCounter;
pub use spec_linter::{SpecLintVerdict, SpecLintViolation, SpecLinter};
pub use spec_matrix_atomizer::SpecMatrixAtomizer;
pub use spec_validator::SpecValidator;
pub use test_integrity_auditor::TestIntegrityAuditor;
pub use ts_assertion_counter::{TsAssertionCounter, TsTestAnalysis};
pub use wave_coupling_analyzer::WaveCouplingAnalyzer;
pub use wave_planner::{ConfiguredWavePlanner, WaveExecutionPlanner};
