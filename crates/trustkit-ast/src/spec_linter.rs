// Static AST and lexical linter for pre-lock specification documents (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecLintViolation {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub remediation: String,
}

impl SpecLintViolation {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"code\":\"{}\",\"message\":\"{}\",\"line\":{},\"remediation\":\"{}\"}}",
            escape_json(&self.code),
            escape_json(&self.message),
            self.line,
            escape_json(&self.remediation)
        )
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecLintVerdict {
    CleanPass,
    ViolationsFound(Vec<SpecLintViolation>),
}

impl SpecLintVerdict {
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::CleanPass)
    }

    pub fn violations(&self) -> &[SpecLintViolation] {
        match self {
            Self::CleanPass => &[],
            Self::ViolationsFound(v) => v.as_slice(),
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            Self::CleanPass => "{\"verdict\":\"PASS\",\"violations\":[]}".to_string(),
            Self::ViolationsFound(violations) => {
                let items: Vec<String> = violations.iter().map(|v| v.to_json()).collect();
                format!(
                    "{{\"verdict\":\"FAIL\",\"violations\":[{}]}}",
                    items.join(",")
                )
            }
        }
    }
}

pub struct SpecLinter;

impl SpecLinter {
    pub fn lint_markdown(content: &str) -> SpecLintVerdict {
        let mut violations = Vec::new();

        Self::check_title(content, &mut violations);
        Self::check_product_trio(content, &mut violations);
        Self::check_mermaid_flowchart(content, &mut violations);
        Self::check_gherkin_scenarios(content, &mut violations);
        Self::check_traceability_matrix(content, &mut violations);
        Self::check_invariants_and_emojis(content, &mut violations);

        if violations.is_empty() {
            SpecLintVerdict::CleanPass
        } else {
            SpecLintVerdict::ViolationsFound(violations)
        }
    }

    fn check_title(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_title = content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("# Especificacao:") || t.starts_with("# Especificação:")
        });
        if !has_title {
            violations.push(SpecLintViolation {
                code: "SPEC-001".to_string(),
                message: "Missing H1 title matching '# Especificacao: <nome>'".to_string(),
                line: 1,
                remediation: "Add '# Especificacao: <nome-da-feature>' at the top of the document"
                    .to_string(),
            });
        }
    }

    fn check_product_trio(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_section = content.contains("## 1. Product Trio");
        let has_pm = content.contains("- **PM**:");
        let has_tl = content.contains("- **Tech Lead**:");
        let has_qa = content.contains("- **Design / QA**:") || content.contains("- **QA**:");

        if !has_section || !has_pm || !has_tl || !has_qa {
            violations.push(SpecLintViolation {
                code: "SPEC-002".to_string(),
                message: "Missing or incomplete Product Trio section (requires PM, Tech Lead, Design/QA)".to_string(),
                line: 3,
                remediation: "Include '## 1. Product Trio' with bullet points for PM, Tech Lead and Design / QA".to_string(),
            });
        }
    }

    fn check_mermaid_flowchart(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_mermaid = content.contains("```mermaid");
        let has_flowchart_lr = content.contains("flowchart LR");

        if !has_mermaid || !has_flowchart_lr {
            violations.push(SpecLintViolation {
                code: "SPEC-003".to_string(),
                message: "Missing Mermaid flowchart LR diagram block".to_string(),
                line: 10,
                remediation: "Add ```mermaid block using 'flowchart LR' in section '## 2. Diagrama de Fluxo (Mermaid)'".to_string(),
            });
        }
    }

    fn check_gherkin_scenarios(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_gherkin_section = content.contains("## 3. Criterios de Aceitacao (Gherkin)")
            || content.contains("## 3. Critérios de Aceitação (Gherkin)");
        let lower = content.to_lowercase();
        let has_dado = lower.contains("- **dado**") || lower.contains("- dado");
        let has_quando = lower.contains("- **quando**") || lower.contains("- quando");
        let has_entao = lower.contains("- **entao**")
            || lower.contains("- **então**")
            || lower.contains("- entao");

        if !has_gherkin_section || !has_dado || !has_quando || !has_entao {
            violations.push(SpecLintViolation {
                code: "SPEC-004".to_string(),
                message: "Missing Gherkin acceptance criteria with Dado/Quando/Entao steps".to_string(),
                line: 20,
                remediation: "Include '## 3. Criterios de Aceitacao (Gherkin)' with at least one scenario containing Dado, Quando, Entao".to_string(),
            });
        }
    }

    fn check_traceability_matrix(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_matrix_sec = content.contains("## 4. Matriz de Rastreabilidade");
        let has_table =
            content.contains("| Regra Negocio") || content.contains("| Regra de Negocio");
        let has_rn_rf =
            content.contains("RN-") && content.contains("RF-") && content.contains("TASK-");

        if !has_matrix_sec || !has_table || !has_rn_rf {
            violations.push(SpecLintViolation {
                code: "SPEC-005".to_string(),
                message: "Missing or incomplete Traceability Matrix with RN, RF and TASK columns".to_string(),
                line: 30,
                remediation: "Include '## 4. Matriz de Rastreabilidade' with table linking RN-XXX, RF-XXX, and TASK-XXX".to_string(),
            });
        }
    }

    fn check_invariants_and_emojis(content: &str, violations: &mut Vec<SpecLintViolation>) {
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;

            let lower = line.to_lowercase();
            if lower.contains("co-authored-by:")
                || lower.contains("generated by:")
                || lower.contains("authored-by ai")
            {
                violations.push(SpecLintViolation {
                    code: "SPEC-006".to_string(),
                    message: format!("Forbidden AI trailer detected: '{}'", line.trim()),
                    line: line_num,
                    remediation: "Remove all AI co-authorship trailers".to_string(),
                });
            }

            for ch in line.chars() {
                if is_emoji_character(ch) {
                    violations.push(SpecLintViolation {
                        code: "SPEC-007".to_string(),
                        message: format!(
                            "Forbidden Unicode emoji detected: '{}' (U+{:04X})",
                            ch, ch as u32
                        ),
                        line: line_num,
                        remediation: "Remove all Unicode emojis from specification".to_string(),
                    });
                    break;
                }
            }
        }
    }
}

fn is_emoji_character(c: char) -> bool {
    matches!(c,
        '\u{1F600}'..='\u{1F64F}' |
        '\u{1F300}'..='\u{1F5FF}' |
        '\u{1F680}'..='\u{1F6FF}' |
        '\u{1F700}'..='\u{1F77F}' |
        '\u{1F780}'..='\u{1F7FF}' |
        '\u{1F800}'..='\u{1F8FF}' |
        '\u{1F900}'..='\u{1F9FF}' |
        '\u{1FA00}'..='\u{1FA6F}' |
        '\u{1FA70}'..='\u{1FAFF}' |
        '\u{2600}'..='\u{26FF}'   |
        '\u{2700}'..='\u{27BF}'
    )
}
