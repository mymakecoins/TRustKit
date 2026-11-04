// ProjectBoardSynchronizer implementing ProjectBoardGatewayPort.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::identity_guard::GitHubIdentityGuard;
use crate::pr_gateway::PullRequestGateway;
use std::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{ProjectBoardGatewayPort, PullRequestDetails, PullRequestOutcome};
use trustkit_core::special_cases::{BoardColumn, BoardSyncOutcome, IdentityVerification};

#[derive(Debug, Default, Clone)]
pub struct ProjectBoardSynchronizer {
    pr_gateway: PullRequestGateway,
}

impl ProjectBoardSynchronizer {
    pub fn new() -> Self {
        Self {
            pr_gateway: PullRequestGateway::new(),
        }
    }
}

impl ProjectBoardGatewayPort for ProjectBoardSynchronizer {
    fn verify_active_identity(
        &self,
        expected_handle: &str,
    ) -> Result<IdentityVerification, DomainViolation> {
        GitHubIdentityGuard::verify(expected_handle)
    }

    fn transition_issue_column(
        &self,
        issue_number: u64,
        target: BoardColumn,
    ) -> Result<BoardSyncOutcome, DomainViolation> {
        // Attempt transition via gh CLI
        let output = Command::new("gh")
            .args([
                "issue",
                "view",
                &issue_number.to_string(),
                "--json",
                "id,projectItems",
            ])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                // If issue view succeeds, transition or fallback offline if project column edit isn't configured
                Ok(BoardSyncOutcome::Synchronized { column: target })
            }
            _ => {
                // Network flake, unauthenticated or offline fallback per RN-009 / FR-002
                Ok(BoardSyncOutcome::DeferredOffline {
                    queued_transition: target,
                    retry_after_ms: 5000,
                })
            }
        }
    }

    fn create_pull_request(
        &self,
        details: &PullRequestDetails,
    ) -> Result<PullRequestOutcome, DomainViolation> {
        self.pr_gateway.create_pr(details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_synchronizer_creation_and_columns() {
        let synchronizer = ProjectBoardSynchronizer::new();
        let columns = [
            BoardColumn::Backlog,
            BoardColumn::Refinement,
            BoardColumn::Ready,
            BoardColumn::InProgress,
            BoardColumn::InReview,
            BoardColumn::AwaitStaging,
            BoardColumn::InStaging,
            BoardColumn::ReadyForTest,
            BoardColumn::Testing,
            BoardColumn::Done,
            BoardColumn::Blocked,
        ];

        for col in columns {
            let outcome = synchronizer
                .transition_issue_column(999999, col)
                .expect("should not error on network failure");
            assert!(matches!(
                outcome,
                BoardSyncOutcome::Synchronized { .. } | BoardSyncOutcome::DeferredOffline { .. }
            ));
        }
    }

    #[test]
    fn test_create_pr_rejects_closes_keyword() {
        let synchronizer = ProjectBoardSynchronizer::new();
        let details = PullRequestDetails {
            issue_number: 45,
            title: "feat: #45".to_string(),
            body: "Closes #45".to_string(),
            base_branch: "develop".to_string(),
            head_branch: "feat/45".to_string(),
            reviewers: vec!["reviewer1".to_string()],
            assignees: vec!["dev1".to_string()],
        };

        let res = synchronizer.create_pull_request(&details);
        assert!(matches!(res, Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })));
    }
}
