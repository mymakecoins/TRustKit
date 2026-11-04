// Gate 2 (Spec Lock) automatic knowledge ingestion use case (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::knowledge_entities::{KnowledgeDocument, KnowledgeWriteOutcome};
use crate::ports::KnowledgeRepositoryPort;
use crate::use_cases::lock_specification::LockSpecificationOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestLockedSpecKnowledgeRequest {
    pub lock_outcome: LockSpecificationOutcome,
    pub spec_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestLockedSpecKnowledgeOutcome {
    pub cycle_id: String,
    pub slug: String,
    pub file_path: String,
    pub spec_blake3_digest: String,
    pub document_blake3_digest: String,
    pub task_links: Vec<String>,
    pub indexed_links_count: usize,
    pub write_outcome: KnowledgeWriteOutcome,
}

pub fn execute(
    req: IngestLockedSpecKnowledgeRequest,
    knowledge_repo: &dyn KnowledgeRepositoryPort,
) -> Result<IngestLockedSpecKnowledgeOutcome, DomainViolation> {
    let state = req.lock_outcome.spec_lock_state.trim();
    if state != "FROZEN" && state != "SPEC_FROZEN" {
        return Err(DomainViolation::SpecOrTraceabilityIncomplete {
            reason: format!(
                "Specification must be in FROZEN state before Gate 2 knowledge ingestion (got '{state}')"
            ),
            missing_artefact: req.lock_outcome.spec_file_path,
        });
    }

    if req.lock_outcome.blake3_digest.trim().is_empty() {
        return Err(DomainViolation::SpecOrTraceabilityIncomplete {
            reason: "Specification BLAKE3 digest is empty; cannot ingest unsealed spec".to_string(),
            missing_artefact: req.lock_outcome.spec_file_path,
        });
    }

    let cycle_id = req.lock_outcome.cycle_id.trim().to_string();
    let raw_slug = format!("specs/spec-{cycle_id}");
    let slug = KnowledgeDocument::validate_slug(&raw_slug)?;

    let title = extract_markdown_heading(&req.spec_content)
        .unwrap_or_else(|| format!("Especificacao Congelada: {cycle_id}"));

    let tags = vec!["spec".to_string(), "frozen".to_string(), cycle_id.clone()];

    let task_ids = extract_prefixed_tokens(&req.spec_content, "TASK-");
    let rule_ids = extract_prefixed_tokens(&req.spec_content, "RN-");
    let req_ids = extract_prefixed_tokens(&req.spec_content, "RF-");

    let task_links: Vec<String> = task_ids.iter().map(|tid| format!("tasks/{tid}")).collect();

    let body_markdown = synthesize_spec_knowledge_body(
        &req.lock_outcome,
        &rule_ids,
        &req_ids,
        &task_ids,
        &req.spec_content,
    );

    let doc = KnowledgeDocument::new(
        slug.clone(),
        title,
        tags,
        body_markdown,
        req.lock_outcome.locked_at.clone(),
        req.lock_outcome.blake3_digest.clone(),
    );

    let write_outcome = knowledge_repo.write_document(&doc)?;
    let (file_path, document_blake3_digest, indexed_links_count) = match &write_outcome {
        KnowledgeWriteOutcome::Success {
            file_path,
            blake3_digest,
            indexed_links_count,
            ..
        } => (
            file_path.clone(),
            blake3_digest.clone(),
            *indexed_links_count,
        ),
    };

    Ok(IngestLockedSpecKnowledgeOutcome {
        cycle_id,
        slug,
        file_path,
        spec_blake3_digest: req.lock_outcome.blake3_digest,
        document_blake3_digest,
        task_links,
        indexed_links_count,
        write_outcome,
    })
}

fn extract_markdown_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let heading = rest.trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

pub fn extract_prefixed_tokens(content: &str, prefix: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rem = content;
    while let Some(idx) = rem.find(prefix) {
        let start = &rem[idx..];
        let after_prefix = &start[prefix.len()..];
        let suffix_len = after_prefix
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .map(char::len_utf8)
            .sum::<usize>();

        if suffix_len > 0 {
            let suffix = after_prefix[..suffix_len].trim_end_matches(['-', '_']);
            if !suffix.is_empty() && suffix.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                let token = format!("{prefix}{suffix}");
                if !found.contains(&token) {
                    found.push(token);
                }
            }
            rem = &after_prefix[suffix_len..];
        } else {
            rem = after_prefix;
        }
    }
    found
}

fn synthesize_spec_knowledge_body(
    outcome: &LockSpecificationOutcome,
    rules: &[String],
    reqs: &[String],
    tasks: &[String],
    raw_spec: &str,
) -> String {
    let rules_line = if rules.is_empty() {
        "Nenhuma regra listada".to_string()
    } else {
        rules.join(", ")
    };
    let reqs_line = if reqs.is_empty() {
        "Nenhum requisito listado".to_string()
    } else {
        reqs.join(", ")
    };
    let tasks_wikilinks = if tasks.is_empty() {
        "- Nenhuma tarefa vinculada".to_string()
    } else {
        tasks
            .iter()
            .map(|t| format!("- [[tasks/{t}]]"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "## Metadados de Congelamento (Gate 2 - Spec Lock)\n\n\
         - **Cycle ID:** `{}`\n\
         - **Spec File:** `{}`\n\
         - **Estado:** `{}`\n\
         - **Selo BLAKE3:** `{}`\n\
         - **Congelado em:** `{}`\n\
         - **Totais:** Regras: {} | Requisitos: {} | Tarefas: {} | Testes E2E: {}\n\
         - **Regras Identificadas:** {}\n\
         - **Requisitos Identificados:** {}\n\n\
         ## Tarefas Rastreaveis\n\n\
         {}\n\n\
         ## Conteudo Integral da Especificacao\n\n\
         {}",
        outcome.cycle_id,
        outcome.spec_file_path,
        outcome.spec_lock_state,
        outcome.blake3_digest,
        outcome.locked_at,
        outcome.rules_count,
        outcome.requirements_count,
        outcome.tasks_count,
        outcome.e2e_tests_count,
        rules_line,
        reqs_line,
        tasks_wikilinks,
        raw_spec.trim()
    )
}
