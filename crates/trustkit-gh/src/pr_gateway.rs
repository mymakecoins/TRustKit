// PullRequestGateway implementing Pull Request creation and column transition.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::identity_guard::GitHubIdentityGuard;
use std::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{PullRequestDetails, PullRequestOutcome};
use trustkit_core::special_cases::{BoardColumn, IdentityVerification};

#[derive(Debug, Default, Clone)]
pub struct PullRequestGateway;

impl PullRequestGateway {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_invariants(details: &PullRequestDetails) -> Result<(), DomainViolation> {
        let combined = format!("{}\n{}", details.title, details.body).to_lowercase();
        let forbidden_keywords = [
            "closes #", "close #", "fixes #", "fix #", "resolves #", "resolve #",
        ];

        for kw in &forbidden_keywords {
            if combined.contains(kw) {
                return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                    invariant_id: "RN-012".to_string(),
                    intercepted_command: "gh pr create".to_string(),
                    reason: "Automatic issue closing keywords (Closes/Fixes/Resolves) are strictly forbidden in Pull Requests.".to_string(),
                    remediation: "Use 'Ref #<id>' instead of closing keywords.".to_string(),
                });
            }
        }

        let ref_token = format!("ref #{}", details.issue_number);
        if !combined.contains(&ref_token) {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-012".to_string(),
                intercepted_command: "gh pr create".to_string(),
                reason: format!("Pull Request must explicitly reference 'Ref #{}' (RN-012).", details.issue_number),
                remediation: format!("Include 'Ref #{}' in PR body or description.", details.issue_number),
            });
        }

        Ok(())
    }

    pub fn create_pr(
        &self,
        details: &PullRequestDetails,
    ) -> Result<PullRequestOutcome, DomainViolation> {
        Self::validate_invariants(details)?;

        // Verify account identity per RN-014
        match GitHubIdentityGuard::verify("marcos-rodoind")? {
            IdentityVerification::Rejected { detected_handle, expected_handle } => {
                return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                    invariant_id: "RN-014".to_string(),
                    intercepted_command: "gh pr create".to_string(),
                    reason: format!(
                        "Active GitHub account '{}' does not match required profile '{}'.",
                        detected_handle, expected_handle
                    ),
                    remediation: format!("Run 'gh auth switch -u {}' before opening PR.", expected_handle),
                });
            }
            IdentityVerification::GhUnavailable { details: _ } => {
                // If gh is unavailable, return offline fallback outcome or infrastructure fault
                return Ok(PullRequestOutcome {
                    pr_number: 0,
                    pr_url: format!("offline://deferred-pr/{}", details.issue_number),
                    target_column: BoardColumn::InReview,
                });
            }
            IdentityVerification::Verified { .. } => (),
        }

        let base_branch = if details.base_branch.is_empty() {
            "develop".to_string()
        } else {
            details.base_branch.clone()
        };

        let mut reviewers = details.reviewers.clone();
        if !reviewers.contains(&"thiagobotelhorodoind".to_string()) {
            reviewers.push("thiagobotelhorodoind".to_string());
        }

        let mut assignees = details.assignees.clone();
        if !assignees.contains(&"marcos-rodoind".to_string()) {
            assignees.push("marcos-rodoind".to_string());
        }

        let mut cmd = Command::new("gh");
        cmd.args([
            "pr",
            "create",
            "--base",
            &base_branch,
            "--head",
            &details.head_branch,
            "--title",
            &details.title,
            "--body",
            &details.body,
            "--reviewer",
            &reviewers.join(","),
            "--assignee",
            &assignees.join(","),
        ]);

        let output = cmd.output();
        match output {
            Ok(out) if out.status.success() => {
                let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let pr_number = url
                    .split('/')
                    .next_back()
                    .and_then(|seg| seg.parse::<u64>().ok())
                    .unwrap_or(details.issue_number);

                Ok(PullRequestOutcome {
                    pr_number,
                    pr_url: url,
                    target_column: BoardColumn::InReview,
                })
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "pull_request_gateway".to_string(),
                    details: format!("'gh pr create' failed: {}", stderr.trim()),
                })
            }
            Err(e) => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "pull_request_gateway".to_string(),
                details: format!("Failed to spawn 'gh': {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_invariants_rejects_closes_keyword() {
        let details = PullRequestDetails {
            issue_number: 45,
            title: "feat: #45 new feature".to_string(),
            body: "This PR Closes #45 immediately".to_string(),
            base_branch: "develop".to_string(),
            head_branch: "feat/45".to_string(),
            reviewers: vec!["thiagobotelhorodoind".to_string()],
            assignees: vec!["marcos-rodoind".to_string()],
        };

        let err = PullRequestGateway::validate_invariants(&details).unwrap_err();
        match err {
            DomainViolation::AstIntegrityOrInvariantViolation { invariant_id, .. } => {
                assert_eq!(invariant_id, "RN-012");
            }
            _ => panic!("Expected RN-012 violation"),
        }
    }

    #[test]
    fn test_validate_invariants_requires_ref_keyword() {
        let details = PullRequestDetails {
            issue_number: 45,
            title: "feat: #45 new feature".to_string(),
            body: "No reference tag here".to_string(),
            base_branch: "develop".to_string(),
            head_branch: "feat/45".to_string(),
            reviewers: vec!["thiagobotelhorodoind".to_string()],
            assignees: vec!["marcos-rodoind".to_string()],
        };

        let err = PullRequestGateway::validate_invariants(&details).unwrap_err();
        assert!(matches!(err, DomainViolation::AstIntegrityOrInvariantViolation { .. }));
    }

    #[test]
    fn test_validate_invariants_nominal_ref_success() {
        let details = PullRequestDetails {
            issue_number: 45,
            title: "feat: #45 new feature".to_string(),
            body: "Implementation completed. Ref #45".to_string(),
            base_branch: "develop".to_string(),
            head_branch: "feat/45".to_string(),
            reviewers: vec!["thiagobotelhorodoind".to_string()],
            assignees: vec!["marcos-rodoind".to_string()],
        };

        assert!(PullRequestGateway::validate_invariants(&details).is_ok());
    }
}
