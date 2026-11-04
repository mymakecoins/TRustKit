// Dispatcher for MCP `trustkit_spec_*` tools (TASK-006, RF-048).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::mcp_server::{McpTool, tool};
use serde_json::{Map, Value, json};
use std::fs;
use std::sync::Arc;
use trustkit_ast::{SpecLintVerdict, SpecLinter, TreeSitterAstAuditorAdapter};
use trustkit_core::entities::SpecScaffoldTemplate;
use trustkit_core::ports::KnowledgeRepositoryPort;
use trustkit_core::use_cases::compose_spec_context::{self, ComposeSpecContextRequest};

pub fn spec_tools() -> Vec<McpTool> {
    vec![
        tool(
            "trustkit_spec_template",
            "Gera o template e esqueleto deterministico do spec.md com regras constitucionais",
            json!({
                "type": "object",
                "properties": {
                    "feature_slug": { "type": "string", "description": "Slug identificador da feature" },
                    "topic_idea": { "type": "string", "description": "Descricao sucinta da ideia da feature" }
                }
            }),
        ),
        tool(
            "trustkit_spec_context",
            "Recupera contexto tecnico fatiado da memoria wiki e do grafo AST",
            json!({
                "type": "object",
                "properties": {
                    "idea": { "type": "string", "description": "Ideia ou termo para contextualizacao" },
                    "hops": { "type": "integer", "description": "Saltos de grafo AST (padrao: 2)" },
                    "max_snippets": { "type": "integer", "description": "Maximo de artigos wiki correlacionados (padrao: 5)" }
                },
                "required": ["idea"]
            }),
        ),
        tool(
            "trustkit_spec_lint",
            "Valida estaticamente documento de especificacao contra regras constitucionais e estruturais",
            json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "Conteudo Markdown da especificacao a validar" },
                    "spec_file": { "type": "string", "description": "Caminho do arquivo local de especificacao" }
                }
            }),
        ),
    ]
}

pub fn dispatch_spec_tool(
    name: &str,
    args: &Map<String, Value>,
    knowledge_opt: Option<&Arc<dyn KnowledgeRepositoryPort>>,
) -> (String, bool) {
    match name {
        "trustkit_spec_template" => handle_template(args),
        "trustkit_spec_context" => handle_context(args, knowledge_opt),
        "trustkit_spec_lint" => handle_lint(args),
        _ => (
            json!({ "verdict": "FAIL", "tool": name, "error": "Unsupported spec tool" })
                .to_string(),
            true,
        ),
    }
}

fn handle_template(args: &Map<String, Value>) -> (String, bool) {
    let slug = args
        .get("feature_slug")
        .and_then(Value::as_str)
        .unwrap_or("000-new-feature");
    let idea = args
        .get("topic_idea")
        .and_then(Value::as_str)
        .unwrap_or("Descricao da funcionalidade.");

    let markdown = SpecScaffoldTemplate::generate_markdown_skeleton(slug, idea);
    let system_instructions = SpecScaffoldTemplate::build_system_instructions();

    (
        json!({
            "verdict": "PASS",
            "tool": "trustkit_spec_template",
            "feature_slug": slug,
            "system_instructions": system_instructions,
            "scaffold_markdown": markdown
        })
        .to_string(),
        false,
    )
}

fn handle_context(
    args: &Map<String, Value>,
    knowledge_opt: Option<&Arc<dyn KnowledgeRepositoryPort>>,
) -> (String, bool) {
    let idea = args.get("idea").and_then(Value::as_str).unwrap_or("");
    let hops = args.get("hops").and_then(Value::as_u64).unwrap_or(2) as usize;
    let max_snippets = args
        .get("max_snippets")
        .and_then(Value::as_u64)
        .unwrap_or(5) as usize;

    let compose_req = match ComposeSpecContextRequest::new(idea, hops, max_snippets) {
        Ok(req) => req,
        Err(err) => {
            return (
                json!({ "verdict": "FAIL", "tool": "trustkit_spec_context", "error": err.to_string() })
                    .to_string(),
                true,
            );
        }
    };

    let Some(knowledge_repo) = knowledge_opt else {
        return (
            json!({
                "verdict": "PASS",
                "tool": "trustkit_spec_context",
                "wiki_articles": [],
                "ast_slices": [],
                "aggregated_context": "Nenhum repositorio de memoria wiki ativo."
            })
            .to_string(),
            false,
        );
    };

    let slicer = TreeSitterAstAuditorAdapter::new();
    match compose_spec_context::execute(compose_req, knowledge_repo.as_ref(), &slicer) {
        Ok(outcome) => {
            let wiki_json: Vec<Value> = outcome
                .wiki_articles
                .into_iter()
                .map(|a| json!({ "slug": a.slug, "title": a.title, "snippet": a.snippet }))
                .collect();
            let ast_json: Vec<Value> = outcome
                .ast_slices
                .into_iter()
                .map(|s| json!({ "file_path": s.file_path, "symbols": s.symbols, "sliced_code": s.sliced_code }))
                .collect();

            (
                json!({
                    "verdict": "PASS",
                    "tool": "trustkit_spec_context",
                    "wiki_articles": wiki_json,
                    "ast_slices": ast_json,
                    "aggregated_context": outcome.aggregated_context
                })
                .to_string(),
                false,
            )
        }
        Err(err) => (
            json!({ "verdict": "FAIL", "tool": "trustkit_spec_context", "error": err.to_string() })
                .to_string(),
            true,
        ),
    }
}

fn handle_lint(args: &Map<String, Value>) -> (String, bool) {
    let content_opt = args.get("content").and_then(Value::as_str);
    let file_opt = args.get("spec_file").and_then(Value::as_str);

    let content = match (content_opt, file_opt) {
        (Some(c), _) => c.to_string(),
        (None, Some(path)) => match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return (
                    json!({ "verdict": "FAIL", "tool": "trustkit_spec_lint", "error": format!("Failed reading file {}: {}", path, e) }).to_string(),
                    true,
                );
            }
        },
        (None, None) => {
            return (
                json!({ "verdict": "FAIL", "tool": "trustkit_spec_lint", "error": "Must provide 'content' or 'spec_file'" }).to_string(),
                true,
            );
        }
    };

    let verdict = SpecLinter::lint_markdown(&content);
    match verdict {
        SpecLintVerdict::CleanPass => (
            json!({ "verdict": "PASS", "tool": "trustkit_spec_lint", "violations_count": 0, "violations": [] }).to_string(),
            false,
        ),
        SpecLintVerdict::ViolationsFound(violations) => {
            let mapped: Vec<Value> = violations
                .into_iter()
                .map(|v| json!({ "code": v.code, "message": v.message, "line": v.line, "remediation": v.remediation }))
                .collect();
            (
                json!({ "verdict": "FAIL", "tool": "trustkit_spec_lint", "violations_count": mapped.len(), "violations": mapped }).to_string(),
                false,
            )
        }
    }
}
