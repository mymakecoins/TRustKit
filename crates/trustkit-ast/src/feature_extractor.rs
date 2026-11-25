// 4-Layer Feature Extractor from Milestone (Level 2 Meso Decomposition).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    ArchitecturalTaskLayer, FeaturePlan, FeatureStatus, FeatureTaskBlueprint, Milestone,
    MilestoneCapability, MilestoneStatus, PlannedFeature, SourceCitation, TechnicalConstraint,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::FeatureExtractorPort;

#[derive(Debug, Default, Clone, Copy)]
pub struct MilestoneFeatureExtractor;

impl MilestoneFeatureExtractor {
    pub fn slugify(input: &str) -> String {
        let mut slug = String::with_capacity(input.len());
        let mut prev_dash = false;
        for ch in input.chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
                prev_dash = false;
            } else if !prev_dash && !slug.is_empty() {
                slug.push('-');
                prev_dash = true;
            }
        }
        while slug.ends_with('-') {
            slug.pop();
        }
        if slug.is_empty() {
            "feature".to_string()
        } else {
            slug
        }
    }

    pub fn build_layered_blueprints(
        feature_slug: &str,
        caps: &[MilestoneCapability],
        cons: &[TechnicalConstraint],
    ) -> Vec<FeatureTaskBlueprint> {
        let cap_label = caps
            .first()
            .map(|c| c.description.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Fluxo Nominal");
        let short_cap: String = cap_label.chars().take(48).collect();
        let con_label = cons
            .first()
            .map(|c| c.title.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Adaptadores e Resiliencia");
        let short_con: String = con_label.chars().take(48).collect();
        let safe_slug = feature_slug.replace('-', "_");

        vec![
            FeatureTaskBlueprint {
                task_id: "TASK-001".to_string(),
                layer: ArchitecturalTaskLayer::SchemaAndContract,
                title: format!("Schema e Contrato ({short_cap})"),
                business_rule_id: "RN-001".to_string(),
                functional_req_id: "RF-001".to_string(),
                mermaid_node: "TASK_001".to_string(),
                target_test_file: format!("crates/trustkit-core/tests/{safe_slug}_schema_tests.rs"),
                evidence_artifact: format!("{safe_slug}_schema_verified"),
                depends_on: vec![],
            },
            FeatureTaskBlueprint {
                task_id: "TASK-002".to_string(),
                layer: ArchitecturalTaskLayer::CoreAndDomain,
                title: format!("Core e Dominio ({short_cap})"),
                business_rule_id: "RN-002".to_string(),
                functional_req_id: "RF-002".to_string(),
                mermaid_node: "TASK_002".to_string(),
                target_test_file: format!("crates/trustkit-core/tests/{safe_slug}_domain_tests.rs"),
                evidence_artifact: format!("{safe_slug}_domain_verified"),
                depends_on: vec!["TASK-001".to_string()],
            },
            FeatureTaskBlueprint {
                task_id: "TASK-003".to_string(),
                layer: ArchitecturalTaskLayer::ApiUiAndAdapters,
                title: format!("API, UI e Adaptadores ({short_con})"),
                business_rule_id: "RN-003".to_string(),
                functional_req_id: "RF-003".to_string(),
                mermaid_node: "TASK_003".to_string(),
                target_test_file: format!("crates/trustkit-cli/tests/{safe_slug}_adapter_tests.rs"),
                evidence_artifact: format!("{safe_slug}_adapter_verified"),
                depends_on: vec!["TASK-001".to_string()],
            },
            FeatureTaskBlueprint {
                task_id: "TASK-004".to_string(),
                layer: ArchitecturalTaskLayer::TelemetryAuditAndE2e,
                title: format!("Telemetria, Auditoria e E2E ({short_cap})"),
                business_rule_id: "RN-004".to_string(),
                functional_req_id: "RF-004".to_string(),
                mermaid_node: "TASK_004".to_string(),
                target_test_file: format!("crates/trustkit-cli/tests/{safe_slug}_e2e_tests.rs"),
                evidence_artifact: format!("{safe_slug}_e2e_verified"),
                depends_on: vec!["TASK-002".to_string(), "TASK-003".to_string()],
            },
        ]
    }

    pub fn parse_milestone_markdown(
        content: &str,
        file_path: &str,
        now_iso: &str,
    ) -> Result<Milestone, DomainViolation> {
        let mut milestone_id = "M01".to_string();
        let mut title = "Marco Planejado".to_string();
        let mut slug = "marco-planejado".to_string();
        let mut goal = "Entregar capacidades do marco".to_string();
        let mut capabilities = Vec::new();
        let mut constraints = Vec::new();
        let mut section = 0u8;

        for (idx, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(h1) = line.strip_prefix("# ") {
                if let Some((m_id, m_title)) = h1.split_once(':') {
                    milestone_id = m_id.trim().to_string();
                    title = m_title.trim().to_string();
                    slug = Self::slugify(&title);
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("> **Slug:**") {
                slug = rest.trim().to_string();
                continue;
            }
            if line.starts_with("## 1.") {
                section = 1;
                continue;
            } else if line.starts_with("## 2.") {
                section = 2;
                continue;
            } else if line.starts_with("## 3.") {
                section = 3;
                continue;
            } else if line.starts_with("## ") {
                section = 4;
                continue;
            }

            match section {
                1 if !line.starts_with('>') => {
                    goal = line.to_string();
                }
                2 if line.starts_with("- **") => {
                    let clean = line.trim_start_matches("- **");
                    if let Some((cap_id, rest)) = clean.split_once("**") {
                        let desc = rest.trim().trim_start_matches(':').trim().to_string();
                        capabilities.push(MilestoneCapability::new(
                            cap_id.trim(),
                            format!("Capacidade {}", cap_id.trim()),
                            desc,
                            SourceCitation::new(file_path, idx + 1, idx + 1, "Capacidades"),
                        ));
                    }
                }
                3 if line.starts_with("- **") => {
                    let clean = line.trim_start_matches("- **");
                    if let Some((tc_id, rest)) = clean.split_once("**") {
                        let desc = rest.trim().trim_start_matches(':').trim().to_string();
                        constraints.push(TechnicalConstraint::new(
                            tc_id.trim(),
                            format!("Restricao {}", tc_id.trim()),
                            desc,
                            SourceCitation::new(file_path, idx + 1, idx + 1, "Restricoes"),
                        ));
                    }
                }
                _ => {}
            }
        }

        if capabilities.is_empty() {
            capabilities.push(MilestoneCapability::new(
                "CAP-001",
                title.clone(),
                goal.clone(),
                SourceCitation::new(file_path, 1, 1, "Objetivo"),
            ));
        }

        Ok(Milestone {
            milestone_id,
            plan_id: "plan-prd-adr".to_string(),
            sequence_index: 1,
            title,
            slug,
            file_path: file_path.to_string(),
            goal,
            dependencies: vec![],
            capabilities,
            technical_constraints: constraints,
            status: MilestoneStatus::Planned,
            created_at: now_iso.to_string(),
            updated_at: now_iso.to_string(),
        })
    }
}

impl FeatureExtractorPort for MilestoneFeatureExtractor {
    fn extract_features_from_milestone(
        &self,
        milestone: &Milestone,
        output_dir: &str,
        now_iso: &str,
    ) -> Result<FeaturePlan, DomainViolation> {
        let caps = if milestone.capabilities.is_empty() {
            vec![MilestoneCapability::new(
                "CAP-001",
                milestone.title.clone(),
                milestone.goal.clone(),
                SourceCitation::new(&milestone.file_path, 1, 1, "Objetivo"),
            )]
        } else {
            milestone.capabilities.clone()
        };

        let chunk_size = if caps.len() <= 3 {
            1
        } else {
            caps.len().div_ceil(2)
        };

        let mut features = Vec::new();
        for (idx, chunk) in caps.chunks(chunk_size).enumerate() {
            let f_num = idx + 1;
            let feature_id = format!("{}-F{:02}", milestone.milestone_id, f_num);
            let role_prefix = match f_num {
                1 => "Caminho Feliz e Fluxo Nominal",
                2 => "Validacoes, Limites e Tratamento de Erros",
                3 => "Resiliencia, Alta Disponibilidade e Contingencia",
                _ => "Expansao Funcional e Integracao",
            };
            let primary_desc = chunk
                .first()
                .map(|c| c.description.as_str())
                .unwrap_or(&milestone.title);
            let short_desc: String = primary_desc.chars().take(45).collect();
            let title = format!("{role_prefix}: {short_desc}");
            let slug = Self::slugify(role_prefix);
            let dir_name = format!("{feature_id}-{slug}");
            let spec_dir = format!(
                "{}/{}/{}",
                output_dir.trim_end_matches('/'),
                milestone.milestone_id,
                dir_name
            );
            let spec_file_path = format!("{spec_dir}/spec.md");
            let dependencies = if idx == 0 {
                vec![]
            } else {
                vec![format!("{}-F{:02}", milestone.milestone_id, idx)]
            };

            let blueprints = Self::build_layered_blueprints(
                &format!("{}_{}", milestone.milestone_id.to_ascii_lowercase(), slug),
                chunk,
                &milestone.technical_constraints,
            );

            features.push(PlannedFeature {
                feature_id,
                milestone_id: milestone.milestone_id.clone(),
                sequence_index: f_num,
                title,
                slug,
                spec_dir,
                spec_file_path,
                goal: format!(
                    "Entregar {} para o marco {} ({})",
                    role_prefix, milestone.milestone_id, milestone.title
                ),
                dependencies,
                capabilities: chunk.to_vec(),
                technical_constraints: milestone.technical_constraints.clone(),
                task_blueprints: blueprints,
                status: FeatureStatus::Planned,
                linked_cycle_id: "UNASSIGNED".to_string(),
                created_at: now_iso.to_string(),
                updated_at: now_iso.to_string(),
            });
        }

        Ok(FeaturePlan {
            milestone_id: milestone.milestone_id.clone(),
            milestone_title: milestone.title.clone(),
            features,
            created_at: now_iso.to_string(),
        })
    }
}
