// Handler for `trustkit spec init`.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::SpecInitArgs;
use crate::error_handler::DomainOutcome;
use serde::Serialize;
use std::fs;
use std::path::Path;
use trustkit_ast::PrdSpecificationIngestor;
use trustkit_core::errors::DomainViolation;
use trustkit_core::use_cases::initialize_spec_from_prd::{
    self, InitializeSpecFromPrdOutcome, InitializeSpecFromPrdRequest,
};

#[derive(Debug, Serialize)]
pub struct JsonBusinessRule {
    pub rule_id: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct JsonFunctionalRequirement {
    pub req_id: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct JsonSpecInitResponse {
    pub status: String,
    pub topic_slug: String,
    pub generated_spec_file: String,
    pub rules_extracted_count: usize,
    pub requirements_extracted_count: usize,
    pub business_rules: Vec<JsonBusinessRule>,
    pub functional_requirements: Vec<JsonFunctionalRequirement>,
}

pub fn execute(args: &SpecInitArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let ingestor = PrdSpecificationIngestor;

    let req = InitializeSpecFromPrdRequest {
        prd_path: args.from_prd.clone(),
        custom_name: args.name.clone(),
        output_dir: args.output_dir.clone(),
    };

    let outcome = initialize_spec_from_prd::execute(req, &ingestor)?;

    // Persist generated spec file to disk
    let spec_path = Path::new(&outcome.generated_spec_file);
    if let Some(parent) = spec_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|err| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "filesystem".to_string(),
            details: format!(
                "Failed creating spec directory '{}': {err}",
                parent.display()
            ),
        })?;
    }

    let markdown_content =
        PrdSpecificationIngestor::render_spec_markdown(&outcome.specification, &outcome.topic_slug);

    fs::write(spec_path, markdown_content).map_err(|err| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "filesystem".to_string(),
            details: format!(
                "Failed writing specification file '{}': {err}",
                spec_path.display()
            ),
        }
    })?;

    let rendered = render_output(&outcome, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

fn render_output(outcome: &InitializeSpecFromPrdOutcome, json_mode: bool) -> String {
    if json_mode {
        let payload = JsonSpecInitResponse {
            status: "SPEC_INITIALIZED".to_string(),
            topic_slug: outcome.topic_slug.clone(),
            generated_spec_file: outcome.generated_spec_file.clone(),
            rules_extracted_count: outcome.rules_extracted_count,
            requirements_extracted_count: outcome.requirements_extracted_count,
            business_rules: outcome
                .business_rules
                .iter()
                .map(|r| JsonBusinessRule {
                    rule_id: r.rule_id.clone(),
                    description: r.description.clone(),
                })
                .collect(),
            functional_requirements: outcome
                .functional_requirements
                .iter()
                .map(|r| JsonFunctionalRequirement {
                    req_id: r.req_id.clone(),
                    description: r.description.clone(),
                })
                .collect(),
        };

        serde_json::to_string_pretty(&payload)
            .unwrap_or_else(|_| "{\"status\":\"SPEC_INITIALIZED\"}".to_string())
    } else {
        let mut out = String::new();
        out.push_str("[PASS] Specification V2 initialized\n");
        out.push_str("------------------------------------------------------------\n");
        out.push_str(&format!(
            "Topic Slug             : {}\n",
            outcome.topic_slug
        ));
        out.push_str(&format!(
            "Generated Spec File    : {}\n",
            outcome.generated_spec_file
        ));
        out.push_str(&format!(
            "Rules Extracted        : {}\n",
            outcome.rules_extracted_count
        ));
        out.push_str(&format!(
            "Requirements Extracted : {}",
            outcome.requirements_extracted_count
        ));
        out
    }
}
