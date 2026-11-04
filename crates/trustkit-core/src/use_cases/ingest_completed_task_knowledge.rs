// Gate 4 (Task Complete) automatic knowledge ingestion use case (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::AtomicTask;
use crate::errors::DomainViolation;
use crate::knowledge_entities::{KnowledgeDocument, KnowledgeWriteOutcome};
use crate::ports::KnowledgeRepositoryPort;
use crate::special_cases::TddLifecycleState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestCompletedTaskKnowledgeRequest {
    pub task: AtomicTask,
    pub commit_sha: String,
    pub red_evidence_summary: String,
    pub green_evidence_summary: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestCompletedTaskKnowledgeOutcome {
    pub cycle_id: String,
    pub task_id: String,
    pub slug: String,
    pub file_path: String,
    pub spec_wikilink: String,
    pub commit_sha: String,
    pub document_blake3_digest: String,
    pub indexed_links_count: usize,
    pub write_outcome: KnowledgeWriteOutcome,
}

pub fn execute(
    req: IngestCompletedTaskKnowledgeRequest,
    knowledge_repo: &dyn KnowledgeRepositoryPort,
) -> Result<IngestCompletedTaskKnowledgeOutcome, DomainViolation> {
    if !is_task_tdd_verified(&req.task.tdd_lifecycle_state) {
        return Err(DomainViolation::TddLifecycleViolation {
            task_id: req.task.task_id.clone(),
            reason: format!(
                "Task '{}' must reach GreenVerified or Committed state before Gate 4 knowledge ingestion (current state: {:?})",
                req.task.task_id, req.task.tdd_lifecycle_state
            ),
        });
    }

    let task_id = req.task.task_id.trim().to_string();
    let cycle_id = req.task.cycle_id.trim().to_string();
    let raw_slug = format!("tasks/{task_id}");
    let slug = KnowledgeDocument::validate_slug(&raw_slug)?;
    let spec_wikilink = format!("specs/spec-{cycle_id}");

    let layer_tag = if req.task.architectural_layer.trim().is_empty() {
        "CORE".to_string()
    } else {
        req.task.architectural_layer.trim().to_string()
    };

    let tags = vec![
        "task".to_string(),
        "tdd-verified".to_string(),
        layer_tag.clone(),
    ];

    let effective_sha = if req.commit_sha.trim().is_empty() {
        req.task.commit_sha.trim().to_string()
    } else {
        req.commit_sha.trim().to_string()
    };

    let title = if req.task.title.trim().is_empty() {
        format!("{task_id}: Tarefa Concluida")
    } else {
        format!("{task_id}: {}", req.task.title.trim())
    };

    let body_markdown = synthesize_task_knowledge_body(
        &req.task,
        &spec_wikilink,
        &effective_sha,
        &req.red_evidence_summary,
        &req.green_evidence_summary,
    );

    let doc = KnowledgeDocument::new(
        slug.clone(),
        title,
        tags,
        body_markdown,
        req.now_iso,
        String::new(),
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

    Ok(IngestCompletedTaskKnowledgeOutcome {
        cycle_id,
        task_id,
        slug,
        file_path,
        spec_wikilink,
        commit_sha: effective_sha,
        document_blake3_digest,
        indexed_links_count,
        write_outcome,
    })
}

const fn is_task_tdd_verified(state: &TddLifecycleState) -> bool {
    matches!(
        state,
        TddLifecycleState::GreenVerified { .. } | TddLifecycleState::Committed { .. }
    )
}

fn synthesize_task_knowledge_body(
    task: &AtomicTask,
    spec_wikilink: &str,
    commit_sha: &str,
    red_summary: &str,
    green_summary: &str,
) -> String {
    let deps_section = if task.depends_on.is_empty() {
        "- Nenhuma dependencia anterior".to_string()
    } else {
        task.depends_on
            .iter()
            .map(|dep| {
                let clean = dep.trim().trim_start_matches("tasks/");
                format!("- [[tasks/{clean}]]")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut files_lines: Vec<String> = task
        .expected_files
        .iter()
        .map(|f| format!("- `{f}`"))
        .collect();
    if !task.target_test_file.trim().is_empty() {
        files_lines.push(format!("- `{}` (test_file)", task.target_test_file.trim()));
    }
    let files_section = if files_lines.is_empty() {
        "- Nenhum arquivo listado".to_string()
    } else {
        files_lines.join("\n")
    };

    format!(
        "## Resumo de Entrega (Gate 4 - Task Complete)\n\n\
         - **Task ID:** `{}`\n\
         - **Titulo:** {}\n\
         - **Ciclo SDD:** `{}` ([[{}]])\n\
         - **Camada Arquitetural:** `{}`\n\
         - **Onda de Execucao:** `{}`\n\
         - **Commit SHA:** `{}`\n\n\
         ## Dependencias\n\n\
         {}\n\n\
         ## Arquivos Implementados\n\n\
         {}\n\n\
         ## Evidencias do Ciclo TDD\n\n\
         - **RED Phase (Evidence #{}):** {}\n\
         - **GREEN Phase (Evidence #{}):** {}",
        task.task_id,
        task.title,
        task.cycle_id,
        spec_wikilink,
        task.architectural_layer,
        task.wave_index,
        commit_sha,
        deps_section,
        files_section,
        task.red_evidence_id,
        red_summary.trim(),
        task.green_evidence_id,
        green_summary.trim()
    )
}
