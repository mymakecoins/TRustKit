// Domain Glossary & ADR Triage Entities (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryEntry {
    pub term: String,
    pub definition: String,
    pub rejected_synonyms: Vec<String>,
}

impl GlossaryEntry {
    pub fn new(
        term: impl Into<String>,
        definition: impl Into<String>,
        rejected_synonyms: Vec<String>,
    ) -> Result<Self, DomainViolation> {
        let entry = Self {
            term: term.into().trim().to_string(),
            definition: definition.into().trim().to_string(),
            rejected_synonyms: rejected_synonyms
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        };
        entry.validate()?;
        Ok(entry)
    }

    pub fn validate(&self) -> Result<(), DomainViolation> {
        if self.term.is_empty() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "glossary".to_string(),
                details: "Glossary entry canonical term cannot be empty".to_string(),
            });
        }
        if self.definition.is_empty() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "glossary".to_string(),
                details: "Glossary entry definition cannot be empty".to_string(),
            });
        }
        // Prohibition of code snippets and implementation file paths in ubiquitous definitions
        if self.definition.contains("```")
            || self.definition.contains("src/")
            || self.definition.contains(".rs")
            || self.definition.contains(".ts")
        {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "glossary".to_string(),
                details:
                    "Definition cannot contain implementation details, code snippets, or file paths"
                        .to_string(),
            });
        }
        Ok(())
    }

    pub fn matches_canonical(&self, candidate: &str) -> bool {
        self.term.eq_ignore_ascii_case(candidate.trim())
    }

    pub fn matches_rejected(&self, candidate: &str) -> bool {
        let needle = candidate.trim().to_lowercase();
        self.rejected_synonyms
            .iter()
            .any(|syn| syn.to_lowercase() == needle)
    }

    pub fn to_markdown(&self) -> String {
        let syns = if self.rejected_synonyms.is_empty() {
            String::new()
        } else {
            format!(" (rejeitados: {})", self.rejected_synonyms.join(", "))
        };
        format!("- **{}**{}: {}", self.term, syns, self.definition)
    }

    pub fn to_json(&self) -> String {
        let syns: Vec<String> = self
            .rejected_synonyms
            .iter()
            .map(|s| format!("\"{}\"", escape_json(s)))
            .collect();
        format!(
            "{{\"term\":\"{}\",\"definition\":\"{}\",\"rejected_synonyms\":[{}]}}",
            escape_json(&self.term),
            escape_json(&self.definition),
            syns.join(",")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DomainGlossary {
    pub entries: Vec<GlossaryEntry>,
}

impl DomainGlossary {
    pub fn new(entries: Vec<GlossaryEntry>) -> Result<Self, DomainViolation> {
        for entry in &entries {
            entry.validate()?;
        }
        Ok(Self { entries })
    }

    pub fn add_entry(&mut self, entry: GlossaryEntry) -> Result<(), DomainViolation> {
        entry.validate()?;
        self.entries.retain(|e| !e.matches_canonical(&entry.term));
        self.entries.push(entry);
        Ok(())
    }

    pub fn find_entry(&self, term: &str) -> Option<&GlossaryEntry> {
        self.entries.iter().find(|e| e.matches_canonical(term))
    }

    pub fn find_canonical_for_rejected(&self, candidate: &str) -> Option<(&GlossaryEntry, &str)> {
        for entry in &self.entries {
            if let Some(matched) = entry
                .rejected_synonyms
                .iter()
                .find(|s| s.eq_ignore_ascii_case(candidate.trim()))
            {
                return Some((entry, matched.as_str()));
            }
        }
        None
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::from("# Glossario de Linguagem Ubiqua\n\n");
        for entry in &self.entries {
            out.push_str(&entry.to_markdown());
            out.push('\n');
        }
        out
    }

    pub fn to_json(&self) -> String {
        let items: Vec<String> = self.entries.iter().map(GlossaryEntry::to_json).collect();
        format!("{{\"entries\":[{}]}}", items.join(","))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrTriageVerdict {
    ApproveAdr,
    SkipAdr,
}

impl AdrTriageVerdict {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ApproveAdr => "ApproveAdr",
            Self::SkipAdr => "SkipAdr",
        }
    }

    pub const fn is_approved(&self) -> bool {
        matches!(self, Self::ApproveAdr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrTriageAssessment {
    pub proposed_title: String,
    pub context_summary: String,
    pub is_hard_to_reverse: bool,
    pub is_surprising_without_context: bool,
    pub has_genuine_alternatives_tradeoff: bool,
    pub verdict: AdrTriageVerdict,
    pub rationale: String,
}

impl AdrTriageAssessment {
    pub fn evaluate(
        proposed_title: impl Into<String>,
        context_summary: impl Into<String>,
        is_hard_to_reverse: bool,
        is_surprising_without_context: bool,
        has_genuine_alternatives_tradeoff: bool,
    ) -> Self {
        let is_approved = is_hard_to_reverse
            && is_surprising_without_context
            && has_genuine_alternatives_tradeoff;
        let verdict = if is_approved {
            AdrTriageVerdict::ApproveAdr
        } else {
            AdrTriageVerdict::SkipAdr
        };
        let rationale = if is_approved {
            "Approved: All 3 mandatory criteria met (hard to reverse, surprising, genuine tradeoff).".to_string()
        } else {
            "Skipped: Proposta reprovada no filtro de 3 criterios. Registre a decisao diretamente no codigo ou na especificacao.".to_string()
        };
        Self {
            proposed_title: proposed_title.into(),
            context_summary: context_summary.into(),
            is_hard_to_reverse,
            is_surprising_without_context,
            has_genuine_alternatives_tradeoff,
            verdict,
            rationale,
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"proposed_title\":\"{}\",\"context_summary\":\"{}\",\"is_hard_to_reverse\":{},\"is_surprising_without_context\":{},\"has_genuine_alternatives_tradeoff\":{},\"verdict\":\"{}\",\"rationale\":\"{}\"}}",
            escape_json(&self.proposed_title),
            escape_json(&self.context_summary),
            self.is_hard_to_reverse,
            self.is_surprising_without_context,
            self.has_genuine_alternatives_tradeoff,
            self.verdict.as_str(),
            escape_json(&self.rationale)
        )
    }
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
