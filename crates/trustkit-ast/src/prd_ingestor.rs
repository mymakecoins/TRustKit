// Upstream PRD/ADR specification parser.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use std::path::Path;
use trustkit_core::entities::{
    BusinessRule, FunctionalRequirement, GherkinScenario, SpecificationV2, TraceabilityRow,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::PrdSpecificationIngestorPort;
use trustkit_core::special_cases::SpecLockState;

#[derive(Debug, Default, Clone, Copy)]
pub struct PrdSpecificationIngestor;

impl PrdSpecificationIngestor {
    pub fn slugify(title: &str) -> String {
        let mut slug = String::with_capacity(title.len());
        let mut prev_dash = false;

        for ch in title.chars() {
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
            "spec".to_string()
        } else {
            slug
        }
    }

    pub fn render_spec_markdown(spec: &SpecificationV2, prd_title: &str) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Especificacao: {}\n\n", prd_title));
        md.push_str("## 1. Product Trio\n\n");
        md.push_str("- **PM**: Definicao de valor de negocio e regras.\n");
        md.push_str("- **Tech Lead**: Arquitetura, viabilidade e seguranca.\n");
        md.push_str("- **Design / QA**: Criterios de aceitacao e cenarios E2E.\n\n");

        md.push_str("## 2. Diagrama de Fluxo (Mermaid)\n\n");
        md.push_str("```mermaid\n");
        md.push_str(&spec.mermaid_content);
        md.push_str("\n```\n\n");

        md.push_str("## 3. Criterios de Aceitacao (Gherkin)\n\n");
        for scenario in &spec.gherkin_scenarios {
            md.push_str(&format!("### {}\n\n", scenario.name));
            for g in &scenario.given {
                md.push_str(&format!("- **Dado** {}\n", g));
            }
            for w in &scenario.when {
                md.push_str(&format!("- **Quando** {}\n", w));
            }
            for t in &scenario.then {
                md.push_str(&format!("- **Entao** {}\n", t));
            }
            md.push('\n');
        }

        md.push_str("## 4. Matriz de Rastreabilidade\n\n");
        md.push_str("| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
        for row in &spec.traceability_matrix {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                row.business_rule_id,
                row.functional_req_id,
                row.task_id,
                row.mermaid_node,
                row.playwright_spec_path,
                row.evidence_screenshot
            ));
        }

        md
    }
}

impl PrdSpecificationIngestorPort for PrdSpecificationIngestor {
    fn ingest_from_prd(
        &self,
        prd_path: &str,
        custom_slug: Option<&str>,
    ) -> Result<SpecificationV2, DomainViolation> {
        let content = fs::read_to_string(Path::new(prd_path)).map_err(|err| {
            DomainViolation::MissingBusinessRules {
                file_path: prd_path.to_string(),
                details: format!("Failed reading upstream PRD: {err}"),
            }
        })?;

        let mut h1_title = String::new();
        let mut business_rules = Vec::new();
        let mut functional_requirements = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if h1_title.is_empty() && trimmed.starts_with("# ") {
                h1_title = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }

            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let columns: Vec<&str> = trimmed
                    .split('|')
                    .map(str::trim)
                    .filter(|col| !col.is_empty())
                    .collect();

                if columns.len() >= 2 {
                    let first = columns[0];
                    let second = columns[1];
                    if first.starts_with("RN-") {
                        business_rules.push(BusinessRule {
                            rule_id: first.to_string(),
                            description: second.to_string(),
                        });
                    } else if first.starts_with("RF-") {
                        functional_requirements.push(FunctionalRequirement {
                            req_id: first.to_string(),
                            description: second.to_string(),
                        });
                    }
                }
            }
        }

        if business_rules.is_empty() {
            return Err(DomainViolation::MissingBusinessRules {
                file_path: prd_path.to_string(),
                details: "No business rules (RN-*) discovered in document tables.".to_string(),
            });
        }

        let slug = match custom_slug {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => {
                if h1_title.is_empty() {
                    "spec".to_string()
                } else {
                    Self::slugify(&h1_title)
                }
            }
        };

        let spec_file_path = format!("{slug}-spec.md");
        let mermaid_content = "flowchart LR\n    A[Entrada Requisitos] --> B[Validacao RN]\n    B --> C[Execucao TDD]\n    C --> D[Evidencia Playwright]".to_string();

        let mut gherkin_scenarios = Vec::new();
        for (idx, rf) in functional_requirements.iter().enumerate() {
            gherkin_scenarios.push(GherkinScenario {
                name: format!("Cenario {}: Atendimento de {}", idx + 1, rf.req_id),
                given: vec![format!("requisito {} habilitado no sistema", rf.req_id)],
                when: vec![rf.description.clone()],
                then: vec!["a operacao deve ser validada e persistida".to_string()],
            });
        }

        if gherkin_scenarios.is_empty() {
            gherkin_scenarios.push(GherkinScenario {
                name: "Cenario Padrao: Conformidade de Negocio".to_string(),
                given: vec!["ambiente inicializado".to_string()],
                when: vec!["regras de negocio sao processadas".to_string()],
                then: vec!["todas as restricoes sao atendidas".to_string()],
            });
        }

        let mut traceability_matrix = Vec::new();
        for (idx, rule) in business_rules.iter().enumerate() {
            let rf_id = functional_requirements
                .get(idx)
                .map(|rf| rf.req_id.clone())
                .unwrap_or_else(|| {
                    functional_requirements
                        .first()
                        .map(|rf| rf.req_id.clone())
                        .unwrap_or_else(|| "RF-001".to_string())
                });

            traceability_matrix.push(TraceabilityRow {
                business_rule_id: rule.rule_id.clone(),
                functional_req_id: rf_id,
                task_id: format!("TASK-{:03}", idx + 1),
                mermaid_node: format!("Node{}", idx + 1),
                playwright_spec_path: format!("tests/e2e/{slug}.spec.ts"),
                evidence_screenshot: format!("SC-{:03}.png", idx + 1),
            });
        }

        Ok(SpecificationV2 {
            cycle_id: String::new(),
            feature_slug: slug,
            spec_file_path,
            business_rules,
            functional_requirements,
            mermaid_content,
            gherkin_scenarios,
            traceability_matrix,
            lock_state: SpecLockState::DraftUnverified,
        })
    }
}
