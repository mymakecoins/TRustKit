// Open Pull Request Use Case (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::ports::{
    CycleEvidenceRepositoryPort, ProjectBoardGatewayPort, PullRequestDetails, PullRequestOutcome,
};
use crate::special_cases::{ActiveCycleLookup, BoardColumn, IssueLinkage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPullRequestRequest {
    pub cycle_id: String,
    pub human_authorized: bool,
    pub base_branch: String,
    pub head_branch: String,
    pub title: String,
    pub custom_body: Option<String>,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPullRequestOutcome {
    pub pr_number: u64,
    pub pr_url: String,
    pub target_column: BoardColumn,
}

fn contains_closing_keyword(text: &str) -> bool {
    let lower = text.to_lowercase();
    let keywords = [
        "closes #",
        "close #",
        "fixes #",
        "fix #",
        "resolves #",
        "resolve #",
    ];
    keywords.iter().any(|kw| lower.contains(kw))
}

pub fn execute(
    req: OpenPullRequestRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    board_gateway: &dyn ProjectBoardGatewayPort,
) -> Result<OpenPullRequestOutcome, DomainViolation> {
    if !req.human_authorized {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "pull_request".to_string(),
            details: "Pull Request creation requires explicit human authorization (RN-012)"
                .to_string(),
        });
    }

    let cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "pull_request".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    let issue_number = match cycle.issue_linkage {
        IssueLinkage::LinkedIssue { issue_number, .. } => issue_number,
        IssueLinkage::UnlinkedLocalCycle { .. } => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "pull_request".to_string(),
                details: "Cannot open Pull Request for unlinked local cycle (RN-012 requires issue linkage)".to_string(),
            });
        }
    };

    let ref_tag = format!("Ref #{}", issue_number);
    let mut body = match req.custom_body {
        Some(custom) => {
            if !custom.contains(&ref_tag) {
                format!("{}\n\n{}", ref_tag, custom)
            } else {
                custom
            }
        }
        None => format!(
            "## MMC Gauntlet SDD Pull Request\n\n{}\n\n### Summary\n- Cycle: `{}`\n- Certified through Gates 0-6",
            ref_tag, cycle.cycle_id
        ),
    };

    if !body.contains("## Merge Danger") {
        let assessment =
            crate::certification_merge_pr_use_case::MergeDangerCalculator::evaluate(&[
                "crates/trustkit-core/src/certification_merge_pr_use_case.rs",
            ]);
        body = crate::certification_merge_pr_use_case::MergeDangerCalculator::inject_into_pr_body(
            &body,
            &assessment,
        );
    }

    // Invariant #12 / RN-012: Strictly forbid automatic issue closure keywords
    if contains_closing_keyword(&body) {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-012".to_string(),
            intercepted_command: "open_pull_request".to_string(),
            reason: "Automatic issue closing keywords (Closes/Fixes/Resolves) are strictly forbidden in Pull Requests.".to_string(),
            remediation: "Use 'Ref #<id>' instead of closing keywords.".to_string(),
        });
    }

    let details = PullRequestDetails {
        issue_number,
        title: req.title,
        body,
        base_branch: req.base_branch,
        head_branch: req.head_branch,
        reviewers: vec![],
        assignees: vec![],
    };

    let outcome: PullRequestOutcome = board_gateway.create_pull_request(&details)?;
    let _ = board_gateway.transition_issue_column(issue_number, BoardColumn::InReview);

    Ok(OpenPullRequestOutcome {
        pr_number: outcome.pr_number,
        pr_url: outcome.pr_url,
        target_column: BoardColumn::InReview,
    })
}
