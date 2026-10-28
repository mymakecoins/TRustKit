// initialize_spec_from_prd use case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::entities::{BusinessRule, FunctionalRequirement, SpecificationV2};
use crate::errors::DomainViolation;
use crate::ports::PrdSpecificationIngestorPort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeSpecFromPrdRequest {
    pub prd_path: String,
    pub custom_name: Option<String>,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeSpecFromPrdOutcome {
    pub topic_slug: String,
    pub generated_spec_file: String,
    pub rules_extracted_count: usize,
    pub requirements_extracted_count: usize,
    pub business_rules: Vec<BusinessRule>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub specification: SpecificationV2,
}

pub fn execute(
    req: InitializeSpecFromPrdRequest,
    ingestor: &dyn PrdSpecificationIngestorPort,
) -> Result<InitializeSpecFromPrdOutcome, DomainViolation> {
    let mut spec = ingestor.ingest_from_prd(&req.prd_path, req.custom_name.as_deref())?;

    let generated_spec_file = match &req.output_dir {
        Some(dir) if !dir.trim().is_empty() => {
            format!(
                "{}/{}-spec.md",
                dir.trim_end_matches('/'),
                spec.feature_slug
            )
        }
        _ => format!("specs/{}/{}-spec.md", spec.feature_slug, spec.feature_slug),
    };

    spec.spec_file_path = generated_spec_file.clone();

    let rules_count = spec.business_rules.len();
    let reqs_count = spec.functional_requirements.len();

    Ok(InitializeSpecFromPrdOutcome {
        topic_slug: spec.feature_slug.clone(),
        generated_spec_file,
        rules_extracted_count: rules_count,
        requirements_extracted_count: reqs_count,
        business_rules: spec.business_rules.clone(),
        functional_requirements: spec.functional_requirements.clone(),
        specification: spec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::special_cases::SpecLockState;

    struct FakeIngestor {
        fail_missing: bool,
    }

    impl PrdSpecificationIngestorPort for FakeIngestor {
        fn ingest_from_prd(
            &self,
            prd_path: &str,
            custom_slug: Option<&str>,
        ) -> Result<SpecificationV2, DomainViolation> {
            if self.fail_missing {
                return Err(DomainViolation::MissingBusinessRules {
                    file_path: prd_path.to_string(),
                    details: "No rules in fake".to_string(),
                });
            }

            let slug = custom_slug.unwrap_or("fake-feature").to_string();
            Ok(SpecificationV2 {
                cycle_id: "test-cycle".to_string(),
                feature_slug: slug.clone(),
                spec_file_path: format!("{slug}-spec.md"),
                business_rules: vec![BusinessRule {
                    rule_id: "RN-001".to_string(),
                    description: "Fake Rule".to_string(),
                }],
                functional_requirements: vec![FunctionalRequirement {
                    req_id: "RF-001".to_string(),
                    description: "Fake Req".to_string(),
                }],
                mermaid_content: "flowchart LR; A-->B;".to_string(),
                gherkin_scenarios: vec![],
                traceability_matrix: vec![],
                lock_state: SpecLockState::DraftUnverified,
            })
        }
    }

    #[test]
    fn test_execute_success_with_output_dir() {
        let fake = FakeIngestor {
            fail_missing: false,
        };
        let req = InitializeSpecFromPrdRequest {
            prd_path: "docs/PRD.md".to_string(),
            custom_name: Some("auth".to_string()),
            output_dir: Some("specs/001-auth/".to_string()),
        };

        let outcome = execute(req, &fake).expect("use case should succeed");
        assert_eq!(outcome.topic_slug, "auth");
        assert_eq!(outcome.generated_spec_file, "specs/001-auth/auth-spec.md");
        assert_eq!(outcome.rules_extracted_count, 1);
        assert_eq!(outcome.requirements_extracted_count, 1);
        assert_eq!(
            outcome.specification.spec_file_path,
            "specs/001-auth/auth-spec.md"
        );
    }

    #[test]
    fn test_execute_propagates_missing_rules_error() {
        let fake = FakeIngestor { fail_missing: true };
        let req = InitializeSpecFromPrdRequest {
            prd_path: "docs/Empty.md".to_string(),
            custom_name: None,
            output_dir: None,
        };

        let err = execute(req, &fake).expect_err("should return error");
        match err {
            DomainViolation::MissingBusinessRules { file_path, .. } => {
                assert_eq!(file_path, "docs/Empty.md");
            }
            other => panic!("expected MissingBusinessRules, got {other:?}"),
        }
    }
}
