// Deterministic synthesis of tasks/TASK-NNN.md on disk (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::{Path, PathBuf};
use trustkit_core::entities::AtomicTaskDraft;
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisReport {
    pub tasks_dir: PathBuf,
    pub created_count: usize,
    pub preserved_count: usize,
    pub created_paths: Vec<PathBuf>,
}

pub struct TaskSynthesizer;

impl TaskSynthesizer {
    pub fn format_task_markdown(draft: &AtomicTaskDraft) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}: {}\n\n", draft.task_id, draft.title));
        out.push_str(&format!("layer: {}\n", draft.architectural_layer));
        let deps = draft
            .depends_on
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("depends_on: [{}]\n", deps));
        out.push_str(&format!("target_test_file: {}\n", draft.target_test_file));
        out.push_str("expected_files:\n");
        for file in &draft.expected_files {
            out.push_str(&format!("  - {}\n", file));
        }
        out.push_str("\n## Contexto\n");
        out.push_str(&draft.context_summary);
        out.push_str("\n\n## Criterios de Aceitacao (Gherkin)\n");
        out.push_str(&format!("- **Dado** {}\n", draft.given_clause));
        out.push_str(&format!("- **Quando** {}\n", draft.when_clause));
        out.push_str(&format!("- **Entao** {}\n", draft.then_clause));
        out.push_str("\n## Rastreabilidade\n");
        out.push_str(&format!("- Regra de Negocio: {}\n", draft.business_rule_id));
        out.push_str(&format!(
            "- Requisito Funcional: {}\n",
            draft.functional_req_id
        ));
        out.push_str(&format!("- Evidencia: {}\n", draft.evidence_artifact));
        out
    }

    pub fn materialize_tasks_dir(
        base_or_tasks_dir: &Path,
        drafts: &[AtomicTaskDraft],
    ) -> Result<Vec<PathBuf>, DomainViolation> {
        let report = Self::synthesize(base_or_tasks_dir, drafts)?;
        Ok(report.created_paths)
    }

    pub fn synthesize(
        base_or_tasks_dir: &Path,
        drafts: &[AtomicTaskDraft],
    ) -> Result<SynthesisReport, DomainViolation> {
        let tasks_dir = if base_or_tasks_dir.file_name().and_then(|s| s.to_str()) == Some("tasks") {
            base_or_tasks_dir.to_path_buf()
        } else {
            base_or_tasks_dir.join("tasks")
        };

        if !tasks_dir.exists() {
            fs::create_dir_all(&tasks_dir).map_err(|err| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "task_synthesizer".to_string(),
                    details: format!(
                        "Failed creating tasks directory '{}': {}",
                        tasks_dir.display(),
                        err
                    ),
                }
            })?;
        }

        let mut created_count = 0;
        let mut preserved_count = 0;
        let mut created_paths = Vec::new();

        for draft in drafts {
            let task_filename = format!("{}.md", draft.task_id);
            let task_path = tasks_dir.join(&task_filename);

            if task_path.exists() {
                preserved_count += 1;
                continue;
            }

            let markdown_content = Self::format_task_markdown(draft);
            fs::write(&task_path, markdown_content).map_err(|err| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "task_synthesizer".to_string(),
                    details: format!(
                        "Failed writing task file '{}': {}",
                        task_path.display(),
                        err
                    ),
                }
            })?;

            created_count += 1;
            created_paths.push(task_path);
        }

        Ok(SynthesisReport {
            tasks_dir,
            created_count,
            preserved_count,
            created_paths,
        })
    }
}
