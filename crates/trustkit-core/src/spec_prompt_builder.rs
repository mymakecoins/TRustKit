// Spec Prompt Builder (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::glossary_entities::DomainGlossary;

pub struct SpecPromptBuilder;

impl SpecPromptBuilder {
    pub fn format_glossary_guidance(glossary: &DomainGlossary) -> String {
        if glossary.entries.is_empty() {
            return String::new();
        }
        let mut out = String::from("=== Vocabulario Ubiquo Obrigatorio (Glossario) ===\n");
        out.push_str("Adote exclusivamente os termos canonicos. Termos rejeitados sao estritamente proibidos:\n");
        for entry in &glossary.entries {
            let syn_note = if entry.rejected_synonyms.is_empty() {
                String::new()
            } else {
                format!(
                    " | Termos Rejeitados / Proibidos: {}",
                    entry.rejected_synonyms.join(", ")
                )
            };
            out.push_str(&format!(
                "- Termo Canonico: \"{}\" | Definicao: \"{}\"{}\n",
                entry.term, entry.definition, syn_note
            ));
        }
        out
    }
}
