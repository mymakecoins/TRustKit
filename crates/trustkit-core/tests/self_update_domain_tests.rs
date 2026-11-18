// Tests for TASK-001: self-update domain contracts and NextStepResolver::for_update.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    NextStepPriority, NextStepResolver, SelfUpdateReport, SelfUpdateRequest, SelfUpdateStatus,
    UpdateInstallStrategy,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::SelfUpdatePort;

struct StubSelfUpdatePort;

impl SelfUpdatePort for StubSelfUpdatePort {
    fn execute_update(&self, req: &SelfUpdateRequest) -> Result<SelfUpdateReport, DomainViolation> {
        Ok(SelfUpdateReport {
            status: if req.check_only {
                SelfUpdateStatus::UpdateAvailable
            } else if req.force {
                SelfUpdateStatus::ForcedReinstalled
            } else {
                SelfUpdateStatus::Updated
            },
            strategy: UpdateInstallStrategy::SourceRepository,
            local_commit: "abc1234".to_string(),
            remote_commit: "def5678".to_string(),
            target_branch: req.target_branch.clone(),
            binaries_updated: vec!["trustkit".to_string(), "tk".to_string()],
            summary: "Self-update check or execution completed".to_string(),
        })
    }
}

#[test]
fn test_self_update_status_and_strategy_labels() {
    assert_eq!(SelfUpdateStatus::UpToDate.as_str(), "UP_TO_DATE");
    assert!(!SelfUpdateStatus::UpToDate.is_update_available());

    assert_eq!(
        SelfUpdateStatus::UpdateAvailable.as_str(),
        "UPDATE_AVAILABLE"
    );
    assert!(SelfUpdateStatus::UpdateAvailable.is_update_available());

    assert_eq!(SelfUpdateStatus::Updated.as_str(), "UPDATED");
    assert_eq!(
        SelfUpdateStatus::ForcedReinstalled.as_str(),
        "FORCED_REINSTALLED"
    );

    assert_eq!(
        UpdateInstallStrategy::SourceRepository.as_str(),
        "SOURCE_REPOSITORY"
    );
    assert_eq!(
        UpdateInstallStrategy::CargoGitInstall.as_str(),
        "CARGO_GIT_INSTALL"
    );
}

#[test]
fn test_self_update_request_defaults_and_port_contract() {
    let default_req = SelfUpdateRequest::new(true, false, "");
    assert_eq!(default_req.target_branch, "main");

    let port = StubSelfUpdatePort;
    let check_report = port.execute_update(&default_req).expect("check succeeds");
    assert!(check_report.status.is_update_available());
    assert_eq!(check_report.binaries_updated.len(), 2);

    let force_req = SelfUpdateRequest::new(false, true, "main");
    let force_report = port.execute_update(&force_req).expect("force succeeds");
    assert_eq!(force_report.status, SelfUpdateStatus::ForcedReinstalled);
}

#[test]
fn test_next_step_resolver_for_update_guidance() {
    let pending_steps = NextStepResolver::for_update(true, true);
    assert_eq!(pending_steps.len(), 2);
    assert_eq!(pending_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(pending_steps[0].target_gate, "MAINTENANCE");
    assert_eq!(pending_steps[0].command, "trustkit update");
    assert_eq!(pending_steps[1].priority, NextStepPriority::Secondary);
    assert_eq!(pending_steps[1].target_gate, "DIAGNOSTIC");
    assert_eq!(pending_steps[1].command, "trustkit doctor");

    let post_steps = NextStepResolver::for_update(false, false);
    assert_eq!(post_steps.len(), 2);
    assert_eq!(post_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(post_steps[0].target_gate, "DIAGNOSTIC");
    assert_eq!(post_steps[0].command, "trustkit doctor");
    assert_eq!(post_steps[1].priority, NextStepPriority::Secondary);
    assert_eq!(post_steps[1].target_gate, "GRAPH");
    assert_eq!(post_steps[1].command, "trustkit graph build");
}
