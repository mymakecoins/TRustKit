// Milestone Extractor for PRD and ADR files.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use trustkit_core::entities::{
    Milestone, MilestoneCapability, MilestonePlan, MilestoneStatus, SourceCitation,
    TechnicalConstraint,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::MilestoneExtractorPort;

#[derive(Debug, Default, Clone, Copy)]
pub struct PrdAdrMilestoneExtractor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMarkdownBlock {
    pub file_path: String,
    pub section_title: String,
    pub title: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub is_table_row: bool,
}

impl PrdAdrMilestoneExtractor {
    pub fn parse_blocks(content: &str, file_path: &str) -> Vec<ExtractedMarkdownBlock> {
        let mut blocks = Vec::new();
        let mut current_section = "Introducao".to_string();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('#') {
                current_section = trimmed.trim_start_matches('#').trim().to_string();
                blocks.push(ExtractedMarkdownBlock {
                    file_path: file_path.to_string(),
                    section_title: current_section.clone(),
                    title: current_section.clone(),
                    content: trimmed.to_string(),
                    start_line: line_num,
                    end_line: line_num,
                    is_table_row: false,
                });
                continue;
            }

            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let cols: Vec<&str> = trimmed
                    .split('|')
                    .map(str::trim)
                    .filter(|c| !c.is_empty())
                    .collect();
                if cols.len() >= 2 && !cols[0].starts_with("---") && !cols[0].starts_with(":---") {
                    let id = cols[0];
                    let desc = if cols.len() >= 3 {
                        format!("{}: {}", cols[1], cols[2])
                    } else {
                        cols[1].to_string()
                    };
                    blocks.push(ExtractedMarkdownBlock {
                        file_path: file_path.to_string(),
                        section_title: current_section.clone(),
                        title: id.to_string(),
                        content: desc,
                        start_line: line_num,
                        end_line: line_num,
                        is_table_row: true,
                    });
                }
                continue;
            }

            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let item = trimmed[2..].trim();
                blocks.push(ExtractedMarkdownBlock {
                    file_path: file_path.to_string(),
                    section_title: current_section.clone(),
                    title: format!("Item L{line_num}"),
                    content: item.to_string(),
                    start_line: line_num,
                    end_line: line_num,
                    is_table_row: false,
                });
                continue;
            }

            blocks.push(ExtractedMarkdownBlock {
                file_path: file_path.to_string(),
                section_title: current_section.clone(),
                title: format!("Texto L{line_num}"),
                content: trimmed.to_string(),
                start_line: line_num,
                end_line: line_num,
                is_table_row: false,
            });
        }

        blocks
    }

    pub fn extract_plan_from_content(
        prd_content: &str,
        adr_content: &str,
        prd_path: &str,
        adr_path: &str,
    ) -> Result<MilestonePlan, DomainViolation> {
        let prd_blocks = Self::parse_blocks(prd_content, prd_path);
        let adr_blocks = Self::parse_blocks(adr_content, adr_path);

        if prd_blocks.is_empty() {
            return Err(DomainViolation::MissingBusinessRules {
                file_path: prd_path.to_string(),
                details: "No readable content found in PRD document".to_string(),
            });
        }

        let mut capabilities = Vec::new();
        for b in &prd_blocks {
            if b.is_table_row
                && (b.title.starts_with("RN-")
                    || b.title.starts_with("RF-")
                    || b.title.starts_with("CAP-"))
            {
                capabilities.push(MilestoneCapability::new(
                    b.title.clone(),
                    format!("Capacidade {}", b.title),
                    b.content.clone(),
                    SourceCitation::new(
                        b.file_path.clone(),
                        b.start_line,
                        b.end_line,
                        b.section_title.clone(),
                    ),
                ));
            } else if (b.section_title.to_ascii_lowercase().contains("requisito")
                || b.section_title.to_ascii_lowercase().contains("visao")
                || b.section_title.to_ascii_lowercase().contains("proposito")
                || b.section_title.to_ascii_lowercase().contains("capacidade"))
                && !b.content.starts_with('#')
            {
                let cap_id = format!("CAP-{:03}", capabilities.len() + 1);
                capabilities.push(MilestoneCapability::new(
                    cap_id,
                    b.section_title.clone(),
                    b.content.clone(),
                    SourceCitation::new(
                        b.file_path.clone(),
                        b.start_line,
                        b.end_line,
                        b.section_title.clone(),
                    ),
                ));
            }
        }

        if capabilities.is_empty() {
            let first = &prd_blocks[0];
            capabilities.push(MilestoneCapability::new(
                "CAP-001",
                first.section_title.clone(),
                first.content.clone(),
                SourceCitation::new(
                    first.file_path.clone(),
                    first.start_line,
                    first.end_line,
                    first.section_title.clone(),
                ),
            ));
        }

        let mut constraints = Vec::new();
        for b in &adr_blocks {
            if (b.section_title.to_ascii_lowercase().contains("decisao")
                || b.section_title.to_ascii_lowercase().contains("arquitetura")
                || b.section_title
                    .to_ascii_lowercase()
                    .contains("justificativa")
                || b.section_title.to_ascii_lowercase().contains("problema"))
                && !b.content.starts_with('#')
            {
                let tc_id = format!("ADR-{:03}", constraints.len() + 1);
                constraints.push(TechnicalConstraint::new(
                    tc_id,
                    b.section_title.clone(),
                    b.content.clone(),
                    SourceCitation::new(
                        b.file_path.clone(),
                        b.start_line,
                        b.end_line,
                        b.section_title.clone(),
                    ),
                ));
            }
        }

        let chunk_size = 4;
        let mut milestones = Vec::new();
        let chunks: Vec<_> = capabilities.chunks(chunk_size).collect();

        for (idx, chunk) in chunks.iter().enumerate() {
            let m_num = idx + 1;
            let m_id = format!("M{m_num:02}");
            let deps = if idx == 0 {
                vec![]
            } else {
                vec![format!("M{:02}", idx)]
            };

            let title = match m_num {
                1 => "Fundacao e Modelos de Dominio".to_string(),
                2 => "Inteligencia Estrutural e Servicos".to_string(),
                3 => "Persistencia Transacional e Integracao".to_string(),
                _ => format!("Evolucao e Entregas Marco {m_num}"),
            };

            let slug = match m_num {
                1 => "fundacao-e-modelos".to_string(),
                2 => "inteligencia-e-servicos".to_string(),
                3 => "persistencia-e-integracao".to_string(),
                _ => format!("entrega-marco-{m_num}"),
            };

            let m_constraints = if idx < constraints.len() {
                constraints[idx..std::cmp::min(idx + 2, constraints.len())].to_vec()
            } else {
                vec![]
            };

            milestones.push(Milestone {
                milestone_id: m_id.clone(),
                plan_id: "plan-prd-adr".to_string(),
                sequence_index: m_num,
                title,
                slug: slug.clone(),
                file_path: format!("docs/milestones/{m_id}-{slug}.md"),
                goal: format!(
                    "Validar capacidades de negocio e restricoes tecnicas do marco {m_id}"
                ),
                dependencies: deps,
                capabilities: chunk.to_vec(),
                technical_constraints: m_constraints,
                status: MilestoneStatus::Planned,
                created_at: "2026-11-06T00:00:00Z".to_string(),
                updated_at: "2026-11-06T00:00:00Z".to_string(),
            });
        }

        Ok(MilestonePlan {
            plan_id: "plan-prd-adr".to_string(),
            prd_path: prd_path.to_string(),
            adr_path: adr_path.to_string(),
            milestones,
            created_at: "2026-11-06T00:00:00Z".to_string(),
        })
    }
}

impl MilestoneExtractorPort for PrdAdrMilestoneExtractor {
    fn extract_milestone_plan(
        &self,
        prd_path: &str,
        adr_path: &str,
    ) -> Result<MilestonePlan, DomainViolation> {
        let prd_content = fs::read_to_string(Path::new(prd_path)).map_err(|err| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "milestone_extractor".to_string(),
                details: format!("Failed reading PRD file '{prd_path}': {err}"),
            }
        })?;

        let adr_content = fs::read_to_string(Path::new(adr_path)).map_err(|err| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "milestone_extractor".to_string(),
                details: format!("Failed reading ADR file '{adr_path}': {err}"),
            }
        })?;

        Self::extract_plan_from_content(&prd_content, &adr_content, prd_path, adr_path)
    }
}
