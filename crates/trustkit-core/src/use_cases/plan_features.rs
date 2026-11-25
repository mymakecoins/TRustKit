// Plan Features Use Case (Level 2 Meso Decomposition from Milestone to Feature Specs V2).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{FeaturePlan, PlannedFeature};
use crate::errors::DomainViolation;
use crate::ports::{FeatureExtractorPort, FeatureRepositoryPort, MilestoneRepositoryPort};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanFeaturesRequest {
    pub milestone_ref: String,
    pub output_dir: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFeatureArtifact {
    pub feature_id: String,
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanFeaturesOutcome {
    pub plan: FeaturePlan,
    pub generated_artifacts: Vec<GeneratedFeatureArtifact>,
}

pub fn render_feature_spec_markdown(feature: &PlannedFeature) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Especificacao: {} - {}\n\n",
        feature.feature_id, feature.title
    ));
    out.push_str(&format!(
        "> **Milestone de Origem:** `{}` | **Feature ID:** `{}` | **Slug:** `{}`\n",
        feature.milestone_id, feature.feature_id, feature.slug
    ));
    out.push_str(&format!("> **Objetivo:** {}\n\n", feature.goal));

    out.push_str("## 1. Product Trio\n\n");
    out.push_str(&format!(
        "- **PM**: Entrega verificavel das capacidades do marco {} ({}).\n",
        feature.milestone_id, feature.goal
    ));
    let adr_summary = if feature.technical_constraints.is_empty() {
        "Isolamento hexagonal, persistencia transacional e zero panics.".to_string()
    } else {
        feature
            .technical_constraints
            .iter()
            .map(|tc| format!("{} ({})", tc.constraint_id, tc.description))
            .collect::<Vec<_>>()
            .join("; ")
    };
    out.push_str(&format!("- **Tech Lead**: {}\n", adr_summary));
    out.push_str(
        "- **Design / QA**: Cobertura TDD estrita (RED -> GREEN) nas 4 camadas arquiteturais.\n\n",
    );

    out.push_str("## 2. Diagrama de Fluxo (Mermaid)\n\n");
    out.push_str("```mermaid\nflowchart LR\n");
    let mut emitted_edges = 0usize;
    for bp in &feature.task_blueprints {
        let dst_title = bp.title.replace('"', "'");
        for dep_id in &bp.depends_on {
            if let Some(src) = feature
                .task_blueprints
                .iter()
                .find(|t| &t.task_id == dep_id)
            {
                let src_title = src.title.replace('"', "'");
                out.push_str(&format!(
                    "    {}[\"{}\"] --> {}[\"{}\"]\n",
                    src.mermaid_node, src_title, bp.mermaid_node, dst_title
                ));
                emitted_edges += 1;
            }
        }
    }
    if emitted_edges == 0 {
        if feature.task_blueprints.len() >= 2 {
            for window in feature.task_blueprints.windows(2) {
                let left = &window[0];
                let right = &window[1];
                let left_title = left.title.replace('"', "'");
                let right_title = right.title.replace('"', "'");
                out.push_str(&format!(
                    "    {}[\"{}\"] --> {}[\"{}\"]\n",
                    left.mermaid_node, left_title, right.mermaid_node, right_title
                ));
            }
        } else if let Some(first) = feature.task_blueprints.first() {
            let title = first.title.replace('"', "'");
            out.push_str(&format!(
                "    {}[\"{}\"] --> DONE[\"Concluido\"]\n",
                first.mermaid_node, title
            ));
        } else {
            out.push_str("    TASK_001[\"Schema e Contrato\"] --> TASK_002[\"Core e Dominio\"]\n");
        }
    }
    out.push_str("```\n\n");

    out.push_str("## 3. Criterios de Aceitacao (Gherkin)\n\n");
    if feature.task_blueprints.is_empty() {
        out.push_str("### Cenario 1: Conformidade Funcional da Feature\n\n");
        out.push_str("- **Dado** o ambiente inicializado para a feature\n");
        out.push_str("- **Quando** o fluxo principal for executado\n");
        out.push_str("- **Entao** todas as regras de negocio e invariantes sao atendidos\n\n");
    } else {
        for (idx, bp) in feature.task_blueprints.iter().enumerate() {
            out.push_str(&format!(
                "### Cenario {}: {} - {} ({})\n\n",
                idx + 1,
                bp.task_id,
                bp.title,
                bp.functional_req_id
            ));
            out.push_str(&format!(
                "- **Dado** a camada {} configurada para {}\n",
                bp.layer.display_label(),
                bp.business_rule_id
            ));
            out.push_str(&format!(
                "- **Quando** a implementacao de {} for validada em `{}`\n",
                bp.task_id, bp.target_test_file
            ));
            out.push_str(&format!(
                "- **Entao** o requisito {} deve ser comprovado com evidencia `{}`\n\n",
                bp.functional_req_id, bp.evidence_artifact
            ));
        }
    }

    out.push_str("## 4. Matriz de Rastreabilidade\n\n");
    out.push_str(
        "| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |\n",
    );
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for bp in &feature.task_blueprints {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            bp.business_rule_id,
            bp.functional_req_id,
            bp.task_id,
            bp.mermaid_node,
            bp.target_test_file,
            bp.evidence_artifact
        ));
    }

    out
}

pub fn render_features_index_markdown(plan: &FeaturePlan) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Indice de Features do Milestone {}: {}\n\n",
        plan.milestone_id, plan.milestone_title
    ));
    out.push_str(&format!("- **Milestone ID:** `{}`\n", plan.milestone_id));
    out.push_str(&format!(
        "- **Total de Features:** {}\n",
        plan.features.len()
    ));
    out.push_str(&format!(
        "- **Total de Tasks Planejadas:** {}\n\n",
        plan.total_tasks()
    ));

    out.push_str("| Feature ID | Titulo | Status | Dependencias | Tasks | Spec File |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for f in &plan.features {
        let deps = if f.dependencies.is_empty() {
            "Nenhuma".to_string()
        } else {
            f.dependencies.join(", ")
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | `{}` |\n",
            f.feature_id,
            f.title,
            f.status.as_str(),
            deps,
            f.task_blueprints.len(),
            f.spec_file_path
        ));
    }
    out
}

pub fn execute(
    req: PlanFeaturesRequest,
    milestone_repo: &dyn MilestoneRepositoryPort,
    extractor: &dyn FeatureExtractorPort,
    feature_repo: Option<&dyn FeatureRepositoryPort>,
) -> Result<PlanFeaturesOutcome, DomainViolation> {
    let milestone = milestone_repo
        .find_milestone_by_id(&req.milestone_ref)?
        .ok_or_else(|| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "plan_features".to_string(),
            details: format!("Milestone '{}' not found", req.milestone_ref),
        })?;

    let mut plan =
        extractor.extract_features_from_milestone(&milestone, &req.output_dir, &req.now_iso)?;
    plan.validate_dag()?;

    let milestone_out_dir = Path::new(&req.output_dir).join(&plan.milestone_id);
    fs::create_dir_all(&milestone_out_dir).map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "plan_features".to_string(),
            details: format!(
                "Failed creating features directory '{}': {e}",
                milestone_out_dir.display()
            ),
        }
    })?;

    let mut generated_artifacts = Vec::new();
    for f in &mut plan.features {
        if let Some(parent) = Path::new(&f.spec_file_path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "plan_features".to_string(),
                    details: format!("Failed creating feature spec directory: {e}"),
                }
            })?;
        }
        let content = render_feature_spec_markdown(f);
        fs::write(&f.spec_file_path, &content).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "plan_features".to_string(),
                details: format!("Failed writing feature spec '{}': {e}", f.spec_file_path),
            }
        })?;
        generated_artifacts.push(GeneratedFeatureArtifact {
            feature_id: f.feature_id.clone(),
            file_path: f.spec_file_path.clone(),
            content,
        });
    }

    let index_path = milestone_out_dir.join("INDEX.md");
    let index_content = render_features_index_markdown(&plan);
    fs::write(&index_path, &index_content).map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "plan_features".to_string(),
            details: format!(
                "Failed writing features index '{}': {e}",
                index_path.display()
            ),
        }
    })?;

    if let Some(repo) = feature_repo {
        repo.save_feature_plan(&plan)?;
    }

    Ok(PlanFeaturesOutcome {
        plan,
        generated_artifacts,
    })
}
