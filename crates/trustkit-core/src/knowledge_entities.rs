// Project Knowledge Memory Domain Entities & Special Cases (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
pub use crate::glossary_entities::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDocument {
    pub slug: String,
    pub title: String,
    pub tags: Vec<String>,
    pub body_markdown: String,
    pub updated_at: String,
    pub blake3_digest: String,
}

impl KnowledgeDocument {
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        tags: Vec<String>,
        body_markdown: impl Into<String>,
        updated_at: impl Into<String>,
        blake3_digest: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            tags,
            body_markdown: body_markdown.into(),
            updated_at: updated_at.into(),
            blake3_digest: blake3_digest.into(),
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn validate_slug(raw_slug: &str) -> Result<String, DomainViolation> {
        let trimmed = raw_slug.trim().trim_end_matches(".md");
        if trimmed.is_empty()
            || trimmed.starts_with('/')
            || trimmed.starts_with('\\')
            || trimmed.contains("..")
            || trimmed.contains('\\')
        {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "knowledge_memory".to_string(),
                details: format!("Invalid or unsafe knowledge document slug: '{raw_slug}'"),
            });
        }
        Ok(trimmed.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    pub source_slug: String,
    pub target_slug: String,
}

impl WikiLink {
    pub fn new(source_slug: impl Into<String>, target_slug: impl Into<String>) -> Self {
        Self {
            source_slug: source_slug.into(),
            target_slug: target_slug.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSearchQuery {
    pub query_text: String,
    pub limit: usize,
    pub required_tags: Vec<String>,
}

impl KnowledgeSearchQuery {
    pub fn new(query_text: impl Into<String>, limit: usize) -> Self {
        Self {
            query_text: query_text.into(),
            limit: if limit == 0 { 10 } else { limit },
            required_tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.required_tags = tags;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnowledgeSearchHit {
    pub slug: String,
    pub title: String,
    pub bm25_score: f64,
    pub snippet: String,
    pub adjacent_links: Vec<String>,
}

impl KnowledgeSearchHit {
    pub fn to_json(&self) -> String {
        let links_json: Vec<String> = self
            .adjacent_links
            .iter()
            .map(|l| format!("\"{}\"", escape_json(l)))
            .collect();
        format!(
            "{{\"slug\":\"{}\",\"title\":\"{}\",\"bm25_score\":{:.4},\"snippet\":\"{}\",\"adjacent_links\":[{}]}}",
            escape_json(&self.slug),
            escape_json(&self.title),
            self.bm25_score,
            escape_json(&self.snippet),
            links_json.join(",")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeLookupOutcome {
    Found(KnowledgeDocument),
    DocumentNotFound,
}

impl KnowledgeLookupOutcome {
    pub const fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }

    pub const fn as_document(&self) -> Option<&KnowledgeDocument> {
        match self {
            Self::Found(doc) => Some(doc),
            Self::DocumentNotFound => None,
        }
    }

    pub const fn variant_label(&self) -> &'static str {
        match self {
            Self::Found(_) => "FOUND",
            Self::DocumentNotFound => "DOCUMENT_NOT_FOUND",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnowledgeSearchOutcome {
    Results(Vec<KnowledgeSearchHit>),
    EmptyKnowledgeList,
}

impl KnowledgeSearchOutcome {
    pub fn from_hits(hits: Vec<KnowledgeSearchHit>) -> Self {
        if hits.is_empty() {
            Self::EmptyKnowledgeList
        } else {
            Self::Results(hits)
        }
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::EmptyKnowledgeList)
    }

    pub fn as_slice(&self) -> &[KnowledgeSearchHit] {
        match self {
            Self::Results(hits) => hits.as_slice(),
            Self::EmptyKnowledgeList => &[],
        }
    }

    pub const fn variant_label(&self) -> &'static str {
        match self {
            Self::Results(_) => "RESULTS",
            Self::EmptyKnowledgeList => "EMPTY_KNOWLEDGE_LIST",
        }
    }

    pub fn as_json_array(&self) -> String {
        match self {
            Self::EmptyKnowledgeList => "[]".to_string(),
            Self::Results(hits) => {
                let items: Vec<String> = hits.iter().map(KnowledgeSearchHit::to_json).collect();
                format!("[{}]", items.join(","))
            }
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"outcome\":\"{}\",\"hits\":{}}}",
            self.variant_label(),
            self.as_json_array()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeWriteOutcome {
    Success {
        slug: String,
        file_path: String,
        blake3_digest: String,
        indexed_links_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeRebuildSummary {
    pub notes_restored: usize,
    pub links_restored: usize,
    pub duration_ms: u64,
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[path = "knowledge_entities_tests.rs"]
mod tests;
