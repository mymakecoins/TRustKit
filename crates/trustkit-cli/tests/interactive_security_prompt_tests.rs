use tempfile::tempdir;
use trustkit_cli::cli_contract::DoneArgs;
use trustkit_cli::commands::done_cmd;
use trustkit_cli::error_handler::CliBoundaryErrorHandler;
use trustkit_core::entities::{
    AppSecCategory, CriticKind, CycleStatus, EvidenceKind, ExecutionEvidence, ExecutionWave,
    GateStage, GateVerdict, MethodologyProfile, ReviewSeverity, SddCycle, SemanticReviewFinding,
    TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    CycleEvidenceRepositoryPort, InteractivePromptPort, SecurityDiffReviewerPort,
};
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

struct MockInteractivePrompt {
    confirm_security_review: bool,
    confirm_abort_on_vuln: bool,
}

impl InteractivePromptPort for MockInteractivePrompt {
    fn prompt_security_review_confirmation(&self) -> Result<bool, DomainViolation> {
        Ok(self.confirm_security_review)
    }

    fn prompt_abort_on_vulnerabilities(
        &self,
        _findings: &[SemanticReviewFinding],
    ) -> Result<bool, DomainViolation> {
        Ok(self.confirm_abort_on_vuln)
    }
}

struct MockSecurityReviewer {
    findings: Vec<SemanticReviewFinding>,
}

impl SecurityDiffReviewerPort for MockSecurityReviewer {
    fn review_branch_diff(
        &self,
        _target_branch: &str,
    ) -> Result<Vec<SemanticReviewFinding>, DomainViolation> {
        Ok(self.findings.clone())
    }
}

fn seed_passed_gauntlet_cycle(dir: &std::path::Path) -> std::path::PathBuf {
    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "cycle-tty-prompt".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "cycle-tty-prompt".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate5GauntletCleared,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.to_str().unwrap().to_string(),
        spec_file_path: dir.join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: dir.join("spec.md").to_str().unwrap().to_string(),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "feat/7-tty".to_string(),
        base_commit_sha: "1".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-30T10:00:00Z".to_string(),
        updated_at: "2026-10-30T10:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");

    let task = TaskNode::new(
        "TASK-005".to_string(),
        "TTY Prompt".to_string(),
        "CLI".to_string(),
        vec![],
        vec!["src/cmd.rs".to_string()],
        "tests/tty.rs".to_string(),
    );
    let tasks_slice = [task.clone()];
    repo.save_atomic_tasks(
        &cycle.cycle_id,
        &tasks_slice,
        &WaveSchedule {
            cycle_id: cycle.cycle_id.clone(),
            total_tasks: 1,
            waves: vec![ExecutionWave {
                wave_index: 1,
                tasks: vec![task],
            }],
        },
    )
    .expect("save task");

    repo.append_evidence(&ExecutionEvidence {
        evidence_id: 0,
        cycle_id: "cycle-tty-prompt".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: 0,
        duration_ms: 1,
        sanitized_stdout_tail: "PASS".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "a".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-30T10:01:00Z".to_string(),
    })
    .expect("append ast");

    for kind in [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ] {
        repo.append_evidence(&ExecutionEvidence {
            evidence_id: 0,
            cycle_id: "cycle-tty-prompt".to_string(),
            gate_stage: GateStage::Gate5GauntletCleared,
            gauntlet_attempt: 0,
            actor_role: format!("CRITIC:{}", kind.as_str()),
            evidence_kind: EvidenceKind::CriticVerdict,
            executed_command: format!("critic {}", kind.as_str()),
            exit_code: 0,
            duration_ms: 5,
            sanitized_stdout_tail: "PASS".to_string(),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "b".repeat(64),
            verdict: GateVerdict::Pass,
            recorded_at: "2026-10-30T10:02:00Z".to_string(),
        })
        .expect("append critic");
    }

    db_path
}

#[test]
fn test_interactive_tty_default_enter_skips_security_review_and_records_opt_out() {
    let dir = tempdir().expect("tempdir");
    let db_path = seed_passed_gauntlet_cycle(dir.path());

    let prompt = MockInteractivePrompt {
        confirm_security_review: false,
        confirm_abort_on_vuln: true,
    };
    let reviewer = MockSecurityReviewer { findings: vec![] };
    let args = DoneArgs::default();

    let res = done_cmd::execute_with_prompt(&args, false, &db_path, true, &prompt, &reviewer);
    assert!(res.is_ok());

    let report_content =
        std::fs::read_to_string(dir.path().join("gauntlet-report.md")).expect("read report");
    assert!(
        report_content
            .contains("- **Specialized Security Review:** SKIPPED (Opt-out pelo desenvolvedor)")
    );
}

#[test]
fn test_interactive_tty_confirmed_security_review_with_sql_injection_aborts_with_exit_code_5() {
    let dir = tempdir().expect("tempdir");
    let db_path = seed_passed_gauntlet_cycle(dir.path());

    let prompt = MockInteractivePrompt {
        confirm_security_review: true,
        confirm_abort_on_vuln: true,
    };
    let reviewer = MockSecurityReviewer {
        findings: vec![SemanticReviewFinding {
            file: "crates/trustkit-db/src/custom_query.rs".to_string(),
            line: 42,
            severity: ReviewSeverity::High,
            category_or_angle: AppSecCategory::InputValidationInjection
                .as_str()
                .to_string(),
            confidence: 9,
            summary: "SQL Injection".to_string(),
            scenario: "Parametro de busca concatenado sem prepared statement.".to_string(),
            recommendation: "Utilizar rusqlite::params!".to_string(),
        }],
    };
    let args = DoneArgs::default();

    let res = done_cmd::execute_with_prompt(&args, false, &db_path, true, &prompt, &reviewer);
    assert!(res.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(res, false, &mut stderr_output);
    assert_eq!(exit_code, 5);
}
