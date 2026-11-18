// Tests for TASK-002: SelfUpdateOrchestrator and UpdateCommandRunnerPort.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use trustkit_core::entities::{SelfUpdateRequest, SelfUpdateStatus, UpdateInstallStrategy};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::SelfUpdatePort;
use trustkit_orch::{SelfUpdateOrchestrator, UpdateCommandRunnerPort};

struct MockUpdateRunner {
    source_repo: bool,
    clean_tree: bool,
    local_sha: String,
    remote_sha: String,
    fail_build: bool,
    build_invoked: AtomicBool,
    cargo_install_invoked: AtomicBool,
}

impl UpdateCommandRunnerPort for MockUpdateRunner {
    fn is_source_repository(&self, _dir: &Path) -> bool {
        self.source_repo
    }

    fn is_working_tree_clean(&self, _dir: &Path) -> Result<bool, DomainViolation> {
        Ok(self.clean_tree)
    }

    fn resolve_local_commit(&self, _dir: &Path) -> Result<String, DomainViolation> {
        Ok(self.local_sha.clone())
    }

    fn resolve_remote_main_commit(
        &self,
        _dir: &Path,
        _branch: &str,
    ) -> Result<String, DomainViolation> {
        Ok(self.remote_sha.clone())
    }

    fn pull_and_build_release(
        &self,
        _dir: &Path,
        _branch: &str,
    ) -> Result<Vec<String>, DomainViolation> {
        self.build_invoked.store(true, Ordering::SeqCst);
        if self.fail_build {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "self_update".to_string(),
                details: "Simulated release compilation error".to_string(),
            });
        }
        Ok(vec!["trustkit".to_string(), "tk".to_string()])
    }

    fn cargo_install_from_git(
        &self,
        _repo_url: &str,
        _branch: &str,
    ) -> Result<Vec<String>, DomainViolation> {
        self.cargo_install_invoked.store(true, Ordering::SeqCst);
        if self.fail_build {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "self_update".to_string(),
                details: "Simulated cargo install failure".to_string(),
            });
        }
        Ok(vec!["trustkit".to_string(), "tk".to_string()])
    }
}

#[test]
fn test_check_only_detects_remote_update_without_building() {
    let runner = MockUpdateRunner {
        source_repo: true,
        clean_tree: true,
        local_sha: "aaaa111".to_string(),
        remote_sha: "bbbb222".to_string(),
        fail_build: false,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch = SelfUpdateOrchestrator::new(".", runner);
    let report = orch
        .execute_update(&SelfUpdateRequest::new(true, false, "main"))
        .expect("check should pass");

    assert_eq!(report.status, SelfUpdateStatus::UpdateAvailable);
    assert_eq!(report.strategy, UpdateInstallStrategy::SourceRepository);
    assert!(report.binaries_updated.is_empty());
}

#[test]
fn test_source_repo_updates_and_force_reinstalls() {
    let runner = MockUpdateRunner {
        source_repo: true,
        clean_tree: true,
        local_sha: "aaaa111".to_string(),
        remote_sha: "bbbb222".to_string(),
        fail_build: false,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch = SelfUpdateOrchestrator::new(".", runner);
    let report = orch
        .execute_update(&SelfUpdateRequest::new(false, false, "main"))
        .expect("update should pass");
    assert_eq!(report.status, SelfUpdateStatus::Updated);
    assert_eq!(report.binaries_updated, vec!["trustkit", "tk"]);

    let same_runner = MockUpdateRunner {
        source_repo: true,
        clean_tree: true,
        local_sha: "bbbb222".to_string(),
        remote_sha: "bbbb222".to_string(),
        fail_build: false,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch_force = SelfUpdateOrchestrator::new(".", same_runner);
    let forced_report = orch_force
        .execute_update(&SelfUpdateRequest::new(false, true, "main"))
        .expect("force reinstall should pass");
    assert_eq!(forced_report.status, SelfUpdateStatus::ForcedReinstalled);
}

#[test]
fn test_dirty_tree_and_compile_failure_trigger_circuit_breaker() {
    let dirty_runner = MockUpdateRunner {
        source_repo: true,
        clean_tree: false,
        local_sha: "aaaa111".to_string(),
        remote_sha: "bbbb222".to_string(),
        fail_build: false,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch_dirty = SelfUpdateOrchestrator::new(".", dirty_runner);
    let err_dirty = orch_dirty
        .execute_update(&SelfUpdateRequest::new(false, false, "main"))
        .expect_err("dirty tree must abort");
    assert!(matches!(
        err_dirty,
        DomainViolation::InfrastructureBoundaryFault { .. }
    ));

    let failing_runner = MockUpdateRunner {
        source_repo: true,
        clean_tree: true,
        local_sha: "aaaa111".to_string(),
        remote_sha: "bbbb222".to_string(),
        fail_build: true,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch_fail = SelfUpdateOrchestrator::new(".", failing_runner);
    let err_fail = orch_fail
        .execute_update(&SelfUpdateRequest::new(false, false, "main"))
        .expect_err("build fault must abort");
    assert!(matches!(
        err_fail,
        DomainViolation::InfrastructureBoundaryFault { .. }
    ));
}

#[test]
fn test_global_install_uses_cargo_git_strategy() {
    let global_runner = MockUpdateRunner {
        source_repo: false,
        clean_tree: true,
        local_sha: "v0.1.0".to_string(),
        remote_sha: "cccc333".to_string(),
        fail_build: false,
        build_invoked: AtomicBool::new(false),
        cargo_install_invoked: AtomicBool::new(false),
    };
    let orch = SelfUpdateOrchestrator::new(".", global_runner);
    let report = orch
        .execute_update(&SelfUpdateRequest::new(false, false, "main"))
        .expect("cargo git install should succeed");
    assert_eq!(report.strategy, UpdateInstallStrategy::CargoGitInstall);
    assert_eq!(report.status, SelfUpdateStatus::Updated);
}
