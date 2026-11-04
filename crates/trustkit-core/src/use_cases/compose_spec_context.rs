// Compose Spec Context Use Case (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{ContextSliceRequest, KnowledgeSearchOutcome, KnowledgeSearchQuery};
use crate::errors::DomainViolation;
use crate::ports::{ContextSlicerPort, KnowledgeRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiSnippet {
    pub slug: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstSliceSnippet {
    pub file_path: String,
    pub symbols: Vec<String>,
    pub sliced_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeSpecContextRequest {
    pub idea: String,
    pub hops: usize,
    pub max_snippets: usize,
}

impl ComposeSpecContextRequest {
    pub fn new(
        idea: impl Into<String>,
        hops: usize,
        max_snippets: usize,
    ) -> Result<Self, DomainViolation> {
        let idea = idea.into();
        let trimmed = idea.trim();
        if trimmed.is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-002".to_string(),
                intercepted_command: "spec context compose".to_string(),
                reason: "ComposeSpecContextRequest idea cannot be empty".to_string(),
                remediation: "Provide a descriptive feature idea for context extraction"
                    .to_string(),
            });
        }
        let cap = if max_snippets == 0 { 5 } else { max_snippets };
        Ok(Self {
            idea: trimmed.to_string(),
            hops,
            max_snippets: cap,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeSpecContextOutcome {
    pub wiki_articles: Vec<WikiSnippet>,
    pub ast_slices: Vec<AstSliceSnippet>,
    pub aggregated_context: String,
}

pub fn execute(
    req: ComposeSpecContextRequest,
    knowledge_repo: &dyn KnowledgeRepositoryPort,
    slicer: &dyn ContextSlicerPort,
) -> Result<ComposeSpecContextOutcome, DomainViolation> {
    let search_query = KnowledgeSearchQuery::new(req.idea.clone(), req.max_snippets);
    let search_outcome = knowledge_repo.search_documents(&search_query)?;

    let mut wiki_articles = Vec::new();
    let mut candidate_files = Vec::new();

    if let KnowledgeSearchOutcome::Results(hits) = search_outcome {
        for hit in hits.into_iter().take(req.max_snippets) {
            wiki_articles.push(WikiSnippet {
                slug: hit.slug.clone(),
                title: hit.title.clone(),
                snippet: hit.snippet.clone(),
            });

            for link in &hit.adjacent_links {
                if link.ends_with(".rs") || link.ends_with(".ts") {
                    candidate_files.push(link.clone());
                }
            }
        }
    }

    let mut ast_slices = Vec::new();
    if !candidate_files.is_empty() {
        let slice_req = ContextSliceRequest {
            target_files: candidate_files.clone(),
            symbol_filters: vec![],
            max_line_budget: 120,
        };
        if let Ok(slice_res) = slicer.slice_context(&slice_req) {
            for file in candidate_files {
                ast_slices.push(AstSliceSnippet {
                    file_path: file,
                    symbols: slice_res.preserved_symbols.clone(),
                    sliced_code: slice_res.sliced_content.clone(),
                });
            }
        }
    }

    let mut summary = String::new();
    summary.push_str("=== Contexto Relevante Extraido ===\n\n");
    if wiki_articles.is_empty() {
        summary.push_str("Nenhum artigo wiki correlacionado encontrado.\n");
    } else {
        summary.push_str("Artigos Wiki:\n");
        for art in &wiki_articles {
            summary.push_str(&format!("- [[{}]] - {}\n", art.slug, art.title));
        }
    }

    if !ast_slices.is_empty() {
        summary.push_str("\nFatias de Codigo AST:\n");
        for ast in &ast_slices {
            summary.push_str(&format!(
                "Arquivo: {}\n{}\n",
                ast.file_path, ast.sliced_code
            ));
        }
    }

    Ok(ComposeSpecContextOutcome {
        wiki_articles,
        ast_slices,
        aggregated_context: summary,
    })
}
