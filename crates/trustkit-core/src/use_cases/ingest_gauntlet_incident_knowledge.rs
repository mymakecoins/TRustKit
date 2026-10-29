// Gate 5 (Gauntlet & Doctor) automatic incident knowledge ingestion use case (TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::knowledge_entities::{KnowledgeDocument, KnowledgeWriteOutcome};
use crate::ports::KnowledgeRepositoryPort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestGauntletIncidentKnowledgeRequest {
    pub cycle_id: String,
    pub attempt: u8,
    pub circuit_breaker_tripped: bool,
    pub violated_invariants: Vec<String>,
    pub raw_diagnostic_output: String,
    pub remediation_hint: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestGauntletIncidentKnowledgeOutcome {
    pub cycle_id: String,
    pub attempt: u8,
    pub slug: String,
    pub file_path: String,
    pub spec_wikilink: String,
    pub sanitized_diagnostic: String,
    pub document_blake3_digest: String,
    pub indexed_links_count: usize,
    pub write_outcome: KnowledgeWriteOutcome,
}

pub fn execute(
    req: IngestGauntletIncidentKnowledgeRequest,
    knowledge_repo: &dyn KnowledgeRepositoryPort,
) -> Result<IngestGauntletIncidentKnowledgeOutcome, DomainViolation> {
    let cycle_id = req.cycle_id.trim().to_string();
    if cycle_id.is_empty() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "ingest_gauntlet_incident_knowledge".to_string(),
            details: "cycle_id cannot be empty when recording Gate 5 incident".to_string(),
        });
    }

    let attempt = req.attempt.max(1);
    let raw_slug = format!("incidents/incident-{cycle_id}-attempt-{attempt}");
    let slug = KnowledgeDocument::validate_slug(&raw_slug)?;
    let spec_wikilink = format!("specs/spec-{cycle_id}");

    let tags = vec![
        "incident".to_string(),
        "gauntlet".to_string(),
        "post-mortem".to_string(),
    ];

    let sanitized_diagnostic = sanitize_incident_output(&req.raw_diagnostic_output);
    let sanitized_remediation = sanitize_incident_output(&req.remediation_hint);

    let status_label = if req.circuit_breaker_tripped {
        "CIRCUIT_BREAKER_TRIPPED"
    } else {
        "GAUNTLET_REJECTED"
    };

    let invariants_line = if req.violated_invariants.is_empty() {
        "Nenhuma invariante isolada especificada".to_string()
    } else {
        req.violated_invariants.join(", ")
    };

    let title = format!("Incidente Gate 5 ({cycle_id}) - Tentativa {attempt}");
    let body_markdown = format!(
        "## Diagnostico de Incidente (Gate 5 - Gauntlet & Doctor)\n\n\
         - **Ciclo SDD:** `{cycle_id}` ([[{spec_wikilink}]])\n\
         - **Tentativa:** `{attempt}`\n\
         - **Status do Disjuntor:** `{status_label}`\n\
         - **Invariantes Violadas:** {invariants_line}\n\
         - **Registrado em:** `{}`\n\n\
         ## Saida Diagnosticada (Sanitizada)\n\n\
         ```text\n\
         {}\n\
         ```\n\n\
         ## Diretriz de Remediacao\n\n\
         {}",
        req.now_iso,
        sanitized_diagnostic.trim(),
        sanitized_remediation.trim()
    );

    let doc = KnowledgeDocument::new(
        slug.clone(),
        title,
        tags,
        body_markdown,
        req.now_iso,
        String::new(),
    );

    let write_outcome = knowledge_repo.write_document(&doc)?;
    let (file_path, document_blake3_digest, indexed_links_count) = match &write_outcome {
        KnowledgeWriteOutcome::Success {
            file_path,
            blake3_digest,
            indexed_links_count,
            ..
        } => (
            file_path.clone(),
            blake3_digest.clone(),
            *indexed_links_count,
        ),
    };

    Ok(IngestGauntletIncidentKnowledgeOutcome {
        cycle_id,
        attempt,
        slug,
        file_path,
        spec_wikilink,
        sanitized_diagnostic,
        document_blake3_digest,
        indexed_links_count,
        write_outcome,
    })
}

pub fn sanitize_incident_output(raw: &str) -> String {
    let mut lines_out = Vec::new();
    for line in raw.lines() {
        let mut tokens_out = Vec::new();
        let mut redact_next = false;
        for token in line.split_whitespace() {
            if redact_next {
                tokens_out.push("[REDACTED]".to_string());
                redact_next = false;
                continue;
            }
            if token.eq_ignore_ascii_case("Bearer") || token.eq_ignore_ascii_case("Basic") {
                tokens_out.push(token.to_string());
                redact_next = true;
                continue;
            }
            tokens_out.push(sanitize_single_token(token));
        }
        lines_out.push(tokens_out.join(" "));
    }
    lines_out.join("\n")
}

fn sanitize_single_token(token: &str) -> String {
    // Redact URL embedded credentials: scheme://user:pass@host
    if let Some(scheme_idx) = token.find("://") {
        let after_scheme = &token[scheme_idx + 3..];
        if let Some(at_idx) = after_scheme.find('@')
            && after_scheme[..at_idx].contains(':')
        {
            let prefix = &token[..scheme_idx + 3];
            let host_part = &after_scheme[at_idx + 1..];
            return format!("{prefix}[REDACTED]@{host_part}");
        }
    }

    // Redact KEY=VALUE or KEY:VALUE assignments for sensitive keys
    for sep in ['=', ':'] {
        if let Some((k, v)) = token.split_once(sep)
            && !v.is_empty()
            && (is_sensitive_key(k) || is_secret_token_value(v))
        {
            return format!("{k}{sep}[REDACTED]");
        }
    }

    if is_secret_token_value(token) {
        return "[REDACTED]".to_string();
    }

    token.to_string()
}

fn is_sensitive_key(key: &str) -> bool {
    let clean = key
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .to_ascii_lowercase();
    [
        "token",
        "secret",
        "password",
        "passwd",
        "pwd",
        "api_key",
        "apikey",
        "private_key",
        "access_key",
        "client_secret",
        "authorization",
    ]
    .iter()
    .any(|kw| clean.contains(kw))
}

fn is_secret_token_value(val: &str) -> bool {
    let trimmed = val.trim_matches(['"', '\'', ',', ';', ')', '(', '[', ']']);
    let prefixes = [
        "ghp_",
        "gho_",
        "ghs_",
        "ghu_",
        "github_pat_",
        "sk-",
        "sk-ant-",
        "sk-proj-",
        "AKIA",
        "AIza",
        "eyJ",
    ];
    prefixes
        .iter()
        .any(|p| trimmed.starts_with(p) && trimmed.len() >= p.len() + 6)
}
