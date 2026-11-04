// Structural specification validator (Mermaid, Gherkin, Traceability Matrix).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::collections::HashSet;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{SpecValidationReport, SpecValidatorPort};

fn sanitize_markdown_token(token: &str) -> &str {
    token.trim_matches(|c: char| c == '*' || c == '`' || c == '_' || c.is_whitespace())
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SpecValidator;

impl SpecValidator {
    pub fn inspect_specification(&self, spec_content: &str) -> SpecValidationReport {
        let mermaid_valid = Self::validate_mermaid(spec_content);
        let gherkin_valid = Self::validate_gherkin(spec_content);
        let (orphan_rules, total_rules, total_requirements) =
            Self::validate_traceability_matrix(spec_content);

        SpecValidationReport {
            mermaid_valid,
            gherkin_valid,
            orphan_rules,
            total_rules,
            total_requirements,
        }
    }

    fn validate_mermaid(content: &str) -> bool {
        let mut in_mermaid = false;
        let mut mermaid_block = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```mermaid") {
                in_mermaid = true;
                continue;
            }
            if in_mermaid {
                if trimmed.starts_with("```") {
                    break;
                }
                mermaid_block.push_str(trimmed);
                mermaid_block.push('\n');
            }
        }

        let trimmed_block = mermaid_block.trim();
        if trimmed_block.is_empty() {
            return false;
        }

        let valid_type = trimmed_block.starts_with("flowchart LR")
            || trimmed_block.starts_with("flowchart TD")
            || trimmed_block.starts_with("graph LR")
            || trimmed_block.starts_with("graph TD");

        if !valid_type {
            return false;
        }

        let has_connection = trimmed_block.contains("-->")
            || trimmed_block.contains("---")
            || trimmed_block.contains("-.->")
            || trimmed_block.contains("==>");

        if !has_connection {
            return false;
        }

        let forbidden_tags = ["<script", "<iframe", "<div", "<span", "<b>", "<a", "style="];
        for tag in forbidden_tags {
            if trimmed_block.to_lowercase().contains(tag) {
                return false;
            }
        }

        true
    }

    fn validate_gherkin(content: &str) -> bool {
        let mut scenarios: Vec<Vec<String>> = Vec::new();
        let mut current_scenario: Option<Vec<String>> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("<!--") {
                continue;
            }

            if trimmed.starts_with('#') {
                if trimmed.starts_with("### ")
                    || trimmed.to_lowercase().contains("cenario")
                    || trimmed.to_lowercase().contains("scenario")
                {
                    if let Some(sc) = current_scenario.take() {
                        scenarios.push(sc);
                    }
                    current_scenario = Some(Vec::new());
                    continue;
                } else if let Some(sc) = current_scenario.take() {
                    scenarios.push(sc);
                }
            }

            if let Some(ref mut sc) = current_scenario {
                let clean_line = trimmed
                    .trim_start_matches(|c: char| {
                        c == '-' || c == '*' || c == '`' || c.is_whitespace()
                    })
                    .trim_start();
                let lower = clean_line.to_lowercase();

                if lower.starts_with("dado") || lower.starts_with("given") {
                    sc.push("GIVEN".to_string());
                } else if lower.starts_with("quando") || lower.starts_with("when") {
                    sc.push("WHEN".to_string());
                } else if lower.starts_with("entao")
                    || lower.starts_with("então")
                    || lower.starts_with("then")
                {
                    sc.push("THEN".to_string());
                } else {
                    if lower.contains("dado") || lower.contains("given") {
                        sc.push("GIVEN".to_string());
                    }
                    if lower.contains("quando") || lower.contains("when") {
                        sc.push("WHEN".to_string());
                    }
                    if lower.contains("entao") || lower.contains("então") || lower.contains("then")
                    {
                        sc.push("THEN".to_string());
                    }
                }
            }
        }

        if let Some(sc) = current_scenario {
            scenarios.push(sc);
        }

        if scenarios.is_empty() {
            return false;
        }

        for sc in scenarios {
            let has_given = sc.contains(&"GIVEN".to_string());
            let has_when = sc.contains(&"WHEN".to_string());
            let has_then = sc.contains(&"THEN".to_string());
            if !has_given || !has_when || !has_then {
                return false;
            }
        }

        true
    }

    fn validate_traceability_matrix(content: &str) -> (Vec<String>, usize, usize) {
        let mut declared_rules = Vec::new();
        let mut declared_reqs = Vec::new();
        let mut matrix_rows = Vec::new();
        let mut in_matrix_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.contains("Matriz de Rastreabilidade") {
                in_matrix_section = true;
                continue;
            }

            if trimmed.starts_with('#') && in_matrix_section && !trimmed.contains("Matriz") {
                in_matrix_section = false;
            }

            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let cols: Vec<&str> = trimmed
                    .split('|')
                    .map(str::trim)
                    .filter(|c| !c.is_empty())
                    .collect();

                if cols.is_empty() || cols[0].starts_with(":---") {
                    continue;
                }

                if in_matrix_section {
                    let clean_col0 = sanitize_markdown_token(cols[0]);
                    if clean_col0 != "Regra Negocio" && clean_col0.starts_with("RN-") {
                        matrix_rows.push(cols);
                    }
                } else {
                    for col in &cols {
                        let clean = sanitize_markdown_token(col);
                        if clean.starts_with("RN-") && !declared_rules.contains(&clean.to_string())
                        {
                            declared_rules.push(clean.to_string());
                        }
                        if clean.starts_with("RF-") && !declared_reqs.contains(&clean.to_string()) {
                            declared_reqs.push(clean.to_string());
                        }
                    }
                }
            }
        }

        let mut covered_rules = HashSet::new();
        let mut orphan_rules = Vec::new();

        for row in matrix_rows {
            let rule_id = sanitize_markdown_token(row[0]).to_string();
            if !declared_rules.contains(&rule_id) {
                declared_rules.push(rule_id.clone());
            }

            let rf = sanitize_markdown_token(row.get(1).copied().unwrap_or(""));
            let task = sanitize_markdown_token(row.get(2).copied().unwrap_or(""));
            let _mermaid = sanitize_markdown_token(row.get(3).copied().unwrap_or(""));
            let test = sanitize_markdown_token(row.get(4).copied().unwrap_or(""));
            let screenshot = sanitize_markdown_token(row.get(5).copied().unwrap_or(""));

            if !rf.is_empty() && rf.starts_with("RF-") && !declared_reqs.contains(&rf.to_string()) {
                declared_reqs.push(rf.to_string());
            }

            let is_incomplete = task.is_empty()
                || task == "-"
                || task.to_uppercase() == "N/A"
                || screenshot.is_empty()
                || screenshot == "-"
                || screenshot.to_uppercase() == "N/A"
                || test.is_empty()
                || test == "-";

            if is_incomplete {
                if !orphan_rules.contains(&rule_id) {
                    orphan_rules.push(rule_id.clone());
                }
            } else {
                covered_rules.insert(rule_id);
            }
        }

        for declared in &declared_rules {
            if !covered_rules.contains(declared) && !orphan_rules.contains(declared) {
                orphan_rules.push(declared.clone());
            }
        }

        let total_rules = declared_rules.len();
        let total_requirements = declared_reqs.len();

        (orphan_rules, total_rules, total_requirements)
    }
}

impl SpecValidatorPort for SpecValidator {
    fn validate_specification(
        &self,
        spec_content: &str,
    ) -> Result<SpecValidationReport, DomainViolation> {
        let report = self.inspect_specification(spec_content);

        if !report.mermaid_valid {
            return Err(DomainViolation::TraceabilityGap {
                orphan_rule: "MERMAID_SYNTAX".to_string(),
                message: "Bloco Mermaid invalido, ausente ou contem tags HTML nao suportadas (esperado flowchart LR/TD)".to_string(),
            });
        }

        if !report.gherkin_valid {
            return Err(DomainViolation::TraceabilityGap {
                orphan_rule: "GHERKIN_SYNTAX".to_string(),
                message:
                    "Cenarios Gherkin ausentes ou incompletos (devem conter Dado, Quando e Entao)"
                        .to_string(),
            });
        }

        if let Some(orphan) = report.orphan_rules.first() {
            return Err(DomainViolation::TraceabilityGap {
                orphan_rule: orphan.clone(),
                message: format!(
                    "Regra de negocio {orphan} nao possui TASK ou Screenshot correspondente na Matriz de Rastreabilidade"
                ),
            });
        }

        Ok(report)
    }
}
