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
        Self::lint_markdown_with_glossary(
            content,
            &trustkit_core::entities::DomainGlossary::default(),
        )
    }

    pub fn lint_markdown_with_glossary(
        content: &str,
        glossary: &trustkit_core::entities::DomainGlossary,
    ) -> SpecLintVerdict {
        let mut violations = Vec::new();

        Self::check_title(content, &mut violations);
        Self::check_product_trio(content, &mut violations);
        Self::check_mermaid_flowchart(content, &mut violations);
        Self::check_gherkin_scenarios(content, &mut violations);
        Self::check_traceability_matrix(content, &mut violations);
        Self::check_invariants_and_emojis(content, &mut violations);
        Self::check_glossary_conflicts(content, glossary, &mut violations);

        if violations.is_empty() {
            SpecLintVerdict::CleanPass
        } else {
            SpecLintVerdict::ViolationsFound(violations)
        }
    }

    fn add_violation(
        v: &mut Vec<SpecLintViolation>,
        code: &str,
        msg: impl Into<String>,
        line: usize,
        rem: &str,
    ) {
        v.push(SpecLintViolation {
            code: code.to_string(),
            message: msg.into(),
            line,
            remediation: rem.to_string(),
        });
    }

    fn check_title(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_title = content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("# Especificacao:") || t.starts_with("# Especificação:")
        });
        if !has_title {
            Self::add_violation(
                violations,
                "SPEC-001",
                "Missing H1 title matching '# Especificacao: <nome>'",
                1,
                "Add '# Especificacao: <nome-da-feature>' at top",
            );
        }
    }

    fn check_product_trio(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_section = content.contains("## 1. Product Trio");
        let has_pm = content.contains("- **PM**:");
        let has_tl = content.contains("- **Tech Lead**:");
        let has_qa = content.contains("- **Design / QA**:") || content.contains("- **QA**:");

        if !has_section || !has_pm || !has_tl || !has_qa {
            Self::add_violation(
                violations,
                "SPEC-002",
                "Missing or incomplete Product Trio section (requires PM, Tech Lead, Design/QA)",
                3,
                "Include '## 1. Product Trio' with bullet points",
            );
        }
    }

    fn check_mermaid_flowchart(content: &str, violations: &mut Vec<SpecLintViolation>) {
        if !content.contains("```mermaid") || !content.contains("flowchart LR") {
            Self::add_violation(
                violations,
                "SPEC-003",
                "Missing Mermaid flowchart LR diagram block",
                10,
                "Add ```mermaid block using 'flowchart LR'",
            );
        }
    }

    fn check_gherkin_scenarios(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_sec = content.contains("## 3. Criterios de Aceitacao (Gherkin)")
            || content.contains("## 3. Critérios de Aceitação (Gherkin)");
        let lower = content.to_lowercase();
        let has_dado = lower.contains("- **dado**") || lower.contains("- dado");
        let has_quando = lower.contains("- **quando**") || lower.contains("- quando");
        let has_entao = lower.contains("- **entao**")
            || lower.contains("- **então**")
            || lower.contains("- entao");

        if !has_sec || !has_dado || !has_quando || !has_entao {
            Self::add_violation(
                violations,
                "SPEC-004",
                "Missing Gherkin acceptance criteria with Dado/Quando/Entao steps",
                20,
                "Include '## 3. Criterios de Aceitacao (Gherkin)'",
            );
        }
    }

    fn check_traceability_matrix(content: &str, violations: &mut Vec<SpecLintViolation>) {
        let has_matrix_sec = content.contains("## 4. Matriz de Rastreabilidade");
        let has_table =
            content.contains("| Regra Negocio") || content.contains("| Regra de Negocio");
        let has_rn_rf =
            content.contains("RN-") && content.contains("RF-") && content.contains("TASK-");

        if !has_matrix_sec || !has_table || !has_rn_rf {
            Self::add_violation(
                violations,
                "SPEC-005",
                "Missing or incomplete Traceability Matrix with RN, RF and TASK columns",
                30,
                "Include '## 4. Matriz de Rastreabilidade'",
            );
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
                Self::add_violation(
                    violations,
                    "SPEC-006",
                    format!("Forbidden AI trailer detected: '{}'", line.trim()),
                    line_num,
                    "Remove all AI co-authorship trailers",
                );
            }

            for ch in line.chars() {
                if is_emoji_character(ch) {
                    Self::add_violation(
                        violations,
                        "SPEC-007",
                        format!(
                            "Forbidden Unicode emoji detected: '{}' (U+{:04X})",
                            ch, ch as u32
                        ),
                        line_num,
                        "Remove all Unicode emojis from specification",
                    );
                    break;
                }
            }
        }
    }

    fn check_glossary_conflicts(
        content: &str,
        glossary: &trustkit_core::entities::DomainGlossary,
        violations: &mut Vec<SpecLintViolation>,
    ) {
        if glossary.entries.is_empty() {
            return;
        }
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let lower = line.to_lowercase();
            for entry in &glossary.entries {
                for rejected in &entry.rejected_synonyms {
                    let needle = rejected.trim().to_lowercase();
                    if !needle.is_empty() && lower.contains(&needle) {
                        Self::add_violation(
                            violations,
                            "SPEC-008",
                            format!(
                                "Termo rejeitado '{}' detectado. Utilize o termo canonico '{}'",
                                rejected, entry.term
                            ),
                            line_num,
                            &format!("Substitua '{}' por '{}'", rejected, entry.term),
                        );
                    }
                }
            }
        }
    }
}

fn is_emoji_character(c: char) -> bool {
    matches!(
        c,
        '\u{1F600}'..='\u{1F64F}'
            | '\u{1F300}'..='\u{1F5FF}'
            | '\u{1F680}'..='\u{1F6FF}'
            | '\u{1F700}'..='\u{1F77F}'
            | '\u{1F780}'..='\u{1F7FF}'
            | '\u{1F800}'..='\u{1F8FF}'
            | '\u{1F900}'..='\u{1F9FF}'
            | '\u{1FA00}'..='\u{1FA6F}'
            | '\u{1FA70}'..='\u{1FAFF}'
            | '\u{2600}'..='\u{26FF}'
            | '\u{2700}'..='\u{27BF}'
    )
}
