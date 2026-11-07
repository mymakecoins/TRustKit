// Plan Milestones Use Case
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{Milestone, MilestonePlan};
use crate::errors::DomainViolation;
use crate::ports::{MilestoneExtractorPort, MilestoneRepositoryPort};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanMilestonesRequest {
    pub prd_path: String,
    pub adr_path: String,
    pub output_dir: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedMilestoneArtifact {
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanMilestonesOutcome {
    pub plan: MilestonePlan,
    pub generated_artifacts: Vec<GeneratedMilestoneArtifact>,
}

pub fn render_milestone_markdown(milestone: &Milestone) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# {}: {}\n\n",
        milestone.milestone_id, milestone.title
    ));
    out.push_str(&format!("> **Status:** {}\n", milestone.status.as_str()));
    out.push_str(&format!("> **Slug:** {}\n", milestone.slug));
    if milestone.dependencies.is_empty() {
        out.push_str("> **Dependencias:** Nenhuma\n\n");
    } else {
        out.push_str(&format!(
            "> **Dependencias:** {}\n\n",
            milestone.dependencies.join(", ")
        ));
    }

    out.push_str("## 1. Objetivo Verificavel\n\n");
    out.push_str(&milestone.goal);
    out.push_str("\n\n");

    out.push_str("## 2. Capacidades de Negocio (PRD)\n\n");
    if milestone.capabilities.is_empty() {
        out.push_str("Nenhuma capacidade mapeada diretamente.\n\n");
    } else {
        for cap in &milestone.capabilities {
            out.push_str(&format!(
                "- **{}** {}: {} (Origem: `{}`)\n",
                cap.capability_id,
                cap.title,
                cap.description,
                cap.citation.format_citation()
            ));
        }
        out.push('\n');
    }

    out.push_str("## 3. Restricoes Tecnicas e Decisoes (ADR)\n\n");
    if milestone.technical_constraints.is_empty() {
        out.push_str("Nenhuma restricao tecnica associada.\n\n");
    } else {
        for tc in &milestone.technical_constraints {
            out.push_str(&format!(
                "- **{}** {}: {} (Origem: `{}`)\n",
                tc.constraint_id,
                tc.title,
                tc.description,
                tc.citation.format_citation()
            ));
        }
        out.push('\n');
    }

    out.push_str("## 4. Criterios de Conclusao Verificaveis\n\n");
    for (i, cap) in milestone.capabilities.iter().enumerate() {
        out.push_str(&format!(
            "- [ ] CC-{:02}: Comprovacao funcional de {} ({})\n",
            i + 1,
            cap.capability_id,
            cap.title
        ));
    }
    if milestone.capabilities.is_empty() {
        out.push_str("- [ ] CC-01: Entrega e validacao do objetivo do marco\n");
    }

    out
}

pub fn render_index_markdown(plan: &MilestonePlan) -> String {
    let mut out = String::new();
    out.push_str("# Indice de Milestones do Projeto\n\n");
    out.push_str(&format!("- **PRD de Origem:** `{}`\n", plan.prd_path));
    out.push_str(&format!("- **ADR de Origem:** `{}`\n", plan.adr_path));
    out.push_str(&format!(
        "- **Total de Marcos:** {}\n",
        plan.milestones.len()
    ));
    out.push_str(&format!(
        "- **Total de Capacidades:** {}\n\n",
        plan.total_capabilities()
    ));

    out.push_str("| ID | Titulo | Status | Dependencias | Arquivo |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
    for m in &plan.milestones {
        let deps = if m.dependencies.is_empty() {
            "Nenhuma".to_string()
        } else {
            m.dependencies.join(", ")
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | [{}]({}) |\n",
            m.milestone_id,
            m.title,
            m.status.as_str(),
            deps,
            m.file_path,
            m.file_path
        ));
    }

    out
}

pub fn execute(
    req: PlanMilestonesRequest,
    extractor: &dyn MilestoneExtractorPort,
    repository: Option<&dyn MilestoneRepositoryPort>,
) -> Result<PlanMilestonesOutcome, DomainViolation> {
    let mut plan = extractor.extract_milestone_plan(&req.prd_path, &req.adr_path)?;
    plan.validate_dag()?;

    let mut generated_artifacts = Vec::new();
    let out_dir = req.output_dir.trim_end_matches('/');

    // Render index
    let index_path = format!("{out_dir}/INDEX.md");
    let index_content = render_index_markdown(&plan);
    generated_artifacts.push(GeneratedMilestoneArtifact {
        file_path: index_path,
        content: index_content,
    });

    // Render each milestone
    for milestone in &mut plan.milestones {
        let file_path = format!("{out_dir}/{}-{}.md", milestone.milestone_id, milestone.slug);
        milestone.file_path = file_path.clone();
        let content = render_milestone_markdown(milestone);
        generated_artifacts.push(GeneratedMilestoneArtifact { file_path, content });
    }

    // Persist to disk if output_dir is configured
    if !req.output_dir.is_empty() {
        let dir_path = Path::new(&req.output_dir);
        let _ = fs::create_dir_all(dir_path);
        for artifact in &generated_artifacts {
            let _ = fs::write(&artifact.file_path, &artifact.content);
        }
    }

    // Persist to repository if present
    if let Some(repo) = repository {
        repo.save_milestone_plan(&plan)?;
    }

    Ok(PlanMilestonesOutcome {
        plan,
        generated_artifacts,
    })
}
