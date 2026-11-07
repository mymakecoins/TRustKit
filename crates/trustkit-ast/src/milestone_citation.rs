// Milestone Citation Resolver and Validator (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use trustkit_core::entities::{Milestone, SourceCitation};
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCitation {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationValidationResult {
    pub citation: SourceCitation,
    pub is_valid: bool,
    pub total_file_lines: usize,
    pub extracted_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedCitationBlock {
    pub markdown: String,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneCitationReport {
    pub total_citations: usize,
    pub valid_citations: usize,
    pub invalid_citations: Vec<String>,
    pub all_valid: bool,
}

pub struct CitationValidator;

impl CitationValidator {
    pub fn parse_citation_string(input: &str) -> Result<ParsedCitation, DomainViolation> {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split('#').collect();
        if parts.len() != 2 {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_parser".to_string(),
                details: format!("Invalid citation format: expected path#Lxx[-Lyy], got '{input}'"),
            });
        }

        let file_path = parts[0].to_string();
        let fragment = parts[1];

        if !fragment.starts_with('L') {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_parser".to_string(),
                details: format!("Invalid line indicator: '{fragment}'"),
            });
        }

        let line_spec = &fragment[1..];
        let (start, end) = if line_spec.contains("-L") {
            let line_parts: Vec<&str> = line_spec.split("-L").collect();
            if line_parts.len() != 2 {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "citation_parser".to_string(),
                    details: format!("Malformed range in citation: '{fragment}'"),
                });
            }
            let s: usize = line_parts[0].parse().map_err(|_| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "citation_parser".to_string(),
                    details: format!("Invalid start line: '{}'", line_parts[0]),
                }
            })?;
            let e: usize = line_parts[1].parse().map_err(|_| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "citation_parser".to_string(),
                    details: format!("Invalid end line: '{}'", line_parts[1]),
                }
            })?;
            (s, e)
        } else {
            let s: usize =
                line_spec
                    .parse()
                    .map_err(|_| DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "citation_parser".to_string(),
                        details: format!("Invalid single line: '{line_spec}'"),
                    })?;
            (s, s)
        };

        if start == 0 || start > end {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_parser".to_string(),
                details: format!("Inconsistent line range: {start}..{end}"),
            });
        }

        Ok(ParsedCitation {
            file_path,
            start_line: start,
            end_line: end,
        })
    }

    pub fn validate_citation(
        citation: &SourceCitation,
        base_dir: &Path,
    ) -> Result<CitationValidationResult, DomainViolation> {
        if citation.start_line == 0 || citation.start_line > citation.end_line {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_validator".to_string(),
                details: format!(
                    "Invalid line range in citation: {}-{}",
                    citation.start_line, citation.end_line
                ),
            });
        }

        let full_path = if Path::new(&citation.file_path).is_absolute() {
            Path::new(&citation.file_path).to_path_buf()
        } else {
            base_dir.join(&citation.file_path)
        };

        if !full_path.exists() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_validator".to_string(),
                details: format!("Cited file not found on disk: {}", full_path.display()),
            });
        }

        let content = fs::read_to_string(&full_path).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_validator".to_string(),
                details: format!("Failed to read file {}: {e}", full_path.display()),
            }
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let total_file_lines = lines.len();

        if citation.end_line > total_file_lines {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "citation_validator".to_string(),
                details: format!(
                    "Citation end_line {} exceeds total file lines ({}) in {}",
                    citation.end_line,
                    total_file_lines,
                    full_path.display()
                ),
            });
        }

        let mut extracted = Vec::with_capacity(citation.end_line - citation.start_line + 1);
        for line in lines
            .iter()
            .take(citation.end_line)
            .skip(citation.start_line - 1)
        {
            extracted.push((*line).to_string());
        }

        Ok(CitationValidationResult {
            citation: citation.clone(),
            is_valid: true,
            total_file_lines,
            extracted_lines: extracted,
        })
    }

    pub fn format_citation_block(
        citation: &SourceCitation,
        base_dir: &Path,
    ) -> Result<FormattedCitationBlock, DomainViolation> {
        let validation = Self::validate_citation(citation, base_dir)?;
        let snippet = validation.extracted_lines.join("\n");
        let formatted_link = citation.format_citation();

        let markdown = format!(
            "> **Fonte:** `{formatted_link}` ({})\n>\n> ```markdown\n> {}\n> ```",
            citation.section_title,
            validation.extracted_lines.join("\n> ")
        );

        Ok(FormattedCitationBlock { markdown, snippet })
    }

    pub fn validate_milestone(
        milestone: &Milestone,
        base_dir: &Path,
    ) -> Result<MilestoneCitationReport, DomainViolation> {
        let mut total = 0;
        let mut valid = 0;
        let mut invalid = Vec::new();

        for cap in &milestone.capabilities {
            total += 1;
            match Self::validate_citation(&cap.citation, base_dir) {
                Ok(_) => valid += 1,
                Err(err) => invalid.push(format!("{}: {:?}", cap.capability_id, err)),
            }
        }

        for con in &milestone.technical_constraints {
            total += 1;
            match Self::validate_citation(&con.citation, base_dir) {
                Ok(_) => valid += 1,
                Err(err) => invalid.push(format!("{}: {:?}", con.constraint_id, err)),
            }
        }

        Ok(MilestoneCitationReport {
            total_citations: total,
            valid_citations: valid,
            all_valid: invalid.is_empty(),
            invalid_citations: invalid,
        })
    }
}
