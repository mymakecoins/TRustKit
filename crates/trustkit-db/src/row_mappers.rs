// Bidirectional row mappers converting between SQLite NOT NULL sentinel values and trustkit-core types.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use rusqlite::Row;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, AtomicTask, CycleStatus, EvidenceKind, ExecutionEvidence,
    GateStage, GateVerdict, MethodologyProfile, SddCycle,
};
use trustkit_core::special_cases::{
    BoardColumn, ContractUnlockState, IssueLinkage, SpecLockState, TddLifecycleState,
    WorktreeAllocation,
};

pub fn parse_board_column(col: &str) -> BoardColumn {
    match col {
        "Backlog" => BoardColumn::Backlog,
        "Todo" => BoardColumn::Todo,
        "Planning" => BoardColumn::Planning,
        "In progress" => BoardColumn::InProgress,
        "In review" => BoardColumn::InReview,
        "Ready for test" => BoardColumn::ReadyForTest,
        "Testing" => BoardColumn::Testing,
        "Done" => BoardColumn::Done,
        _ => BoardColumn::Blocked,
    }
}

pub fn string_list_to_json(items: &[String]) -> String {
    let escaped: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
        .collect();
    format!("[{}]", escaped.join(","))
}

pub fn json_to_string_list(json: &str) -> Vec<String> {
    json.trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn map_row_to_cycle(row: &Row) -> rusqlite::Result<SddCycle> {
    let cycle_id: String = row.get(0)?;
    let linkage_kind: String = row.get(1)?;
    let issue_number: i64 = row.get(2)?;
    let issue_url: String = row.get(3)?;
    let project_board_id: String = row.get(4)?;
    let project_item_id: String = row.get(5)?;
    let board_column: String = row.get(6)?;
    let methodology: String = row.get(7)?;
    let current_gate_raw: i64 = row.get(8)?;
    let lifecycle_raw: String = row.get(9)?;
    let gauntlet_attempt: i64 = row.get(10)?;
    let spec_dir: String = row.get(11)?;
    let spec_file_path: String = row.get(12)?;
    let spec_lock_raw: String = row.get(13)?;
    let spec_blake3_digest: String = row.get(14)?;
    let work_branch: String = row.get(15)?;
    let base_commit_sha: String = row.get(16)?;
    let dirty_json: String = row.get(17)?;
    let created_at: String = row.get(18)?;
    let updated_at: String = row.get(19)?;

    let issue_linkage = if linkage_kind == "LINKED_ISSUE" {
        IssueLinkage::LinkedIssue {
            issue_number: issue_number as u64,
            issue_url,
            project_board_id,
            project_item_id,
            board_column: parse_board_column(&board_column),
        }
    } else {
        IssueLinkage::UnlinkedLocalCycle {
            directory_slug: cycle_id.clone(),
        }
    };

    let spec_seal_state = if spec_lock_raw == "FROZEN" {
        SpecLockState::Frozen {
            spec_path: spec_file_path.clone(),
            blake3_digest: spec_blake3_digest,
        }
    } else {
        SpecLockState::DraftUnverified
    };

    let methodology_profile = if methodology == "STRUCTURED_SDD" {
        MethodologyProfile::StructuredSdd {
            feature_dir: spec_dir.clone(),
        }
    } else {
        MethodologyProfile::StandardGauntletV2
    };

    let current_gate =
        GateStage::from_u8(current_gate_raw as u8).unwrap_or(GateStage::Gate0Discovery);
    let lifecycle_status = match lifecycle_raw.as_str() {
        "ACTIVE" => CycleStatus::Active,
        "PAUSED" => CycleStatus::Paused,
        "CIRCUIT_BREAKER_TRIPPED" => CycleStatus::CircuitBreakerTripped,
        "CERTIFIED" => CycleStatus::Certified,
        _ => CycleStatus::Merged,
    };

    Ok(SddCycle {
        cycle_id,
        issue_linkage,
        methodology_profile,
        current_gate,
        lifecycle_status,
        gauntlet_attempt: gauntlet_attempt as u8,
        spec_dir,
        spec_file_path,
        spec_seal_state,
        work_branch,
        base_commit_sha,
        pre_existing_dirty_files: json_to_string_list(&dirty_json),
        created_at,
        updated_at,
    })
}

pub fn map_row_to_artifact_seal(row: &Row) -> rusqlite::Result<ArtifactSeal> {
    let seal_id: i64 = row.get(0)?;
    let cycle_id: String = row.get(1)?;
    let file_path: String = row.get(2)?;
    let cat_raw: String = row.get(3)?;
    let blake3_digest: String = row.get(4)?;
    let ast_assertion_count: i64 = row.get(5)?;
    let unlock_state_raw: String = row.get(6)?;
    let unlock_justification: String = row.get(7)?;

    let artifact_category = match cat_raw.as_str() {
        "SPEC" => ArtifactCategory::Spec,
        "CONSTITUTION" => ArtifactCategory::Constitution,
        "PRE_EXISTING_TEST" => ArtifactCategory::PreExistingTest,
        _ => ArtifactCategory::ForeignIssueEvidence,
    };

    let contract_unlock_state = if unlock_state_raw == "UNLOCKED_BY_HUMAN" {
        ContractUnlockState::UnlockedByHuman {
            justification: unlock_justification,
        }
    } else {
        ContractUnlockState::Sealed
    };

    Ok(ArtifactSeal {
        seal_id: seal_id as u64,
        cycle_id,
        file_path,
        artifact_category,
        blake3_digest,
        ast_assertion_count: ast_assertion_count as u32,
        contract_unlock_state,
    })
}

pub fn map_row_to_atomic_task(row: &Row) -> rusqlite::Result<AtomicTask> {
    let cycle_id: String = row.get(0)?;
    let task_id: String = row.get(1)?;
    let title: String = row.get(2)?;
    let architectural_layer: String = row.get(3)?;
    let wave_index: i64 = row.get(4)?;
    let depends_on_json: String = row.get(5)?;
    let expected_files_json: String = row.get(6)?;
    let target_test_file: String = row.get(7)?;
    let tdd_raw: String = row.get(8)?;
    let red_evidence_id: i64 = row.get(9)?;
    let green_evidence_id: i64 = row.get(10)?;
    let worktree_raw: String = row.get(11)?;
    let commit_sha: String = row.get(12)?;

    let tdd_lifecycle_state = match tdd_raw.as_str() {
        "RED_VERIFIED" => TddLifecycleState::RedVerified {
            failing_evidence_id: red_evidence_id as u64,
        },
        "GREEN_VERIFIED" => TddLifecycleState::GreenVerified {
            red_evidence_id: red_evidence_id as u64,
            green_evidence_id: green_evidence_id as u64,
        },
        "COMMITTED" => TddLifecycleState::Committed {
            commit_sha: commit_sha.clone(),
        },
        _ => TddLifecycleState::AwaitingRedPhase,
    };

    let worktree_allocation = match worktree_raw.as_str() {
        s if s.starts_with("PROVISIONED:") => {
            let payload = s.trim_start_matches("PROVISIONED:");
            let parts: Vec<&str> = payload.split(':').collect();
            let (path, branch, base) = match parts.as_slice() {
                [p, b, base] => (p.to_string(), b.to_string(), base.to_string()),
                [p, b] => (p.to_string(), b.to_string(), String::new()),
                _ => (payload.to_string(), String::new(), String::new()),
            };
            WorktreeAllocation::Provisioned {
                worktree_path: path,
                branch_name: branch,
                base_branch: base,
            }
        }
        s if s.starts_with("MERGED:") => WorktreeAllocation::MergedAndPruned {
            commit_sha: s.trim_start_matches("MERGED:").to_string(),
        },
        _ => WorktreeAllocation::NotProvisioned,
    };

    Ok(AtomicTask {
        cycle_id,
        task_id,
        title,
        architectural_layer,
        wave_index: wave_index as u32,
        depends_on: json_to_string_list(&depends_on_json),
        expected_files: json_to_string_list(&expected_files_json),
        target_test_file,
        tdd_lifecycle_state,
        red_evidence_id: red_evidence_id as u64,
        green_evidence_id: green_evidence_id as u64,
        worktree_allocation,
        commit_sha,
    })
}

pub fn map_row_to_evidence(row: &Row) -> rusqlite::Result<ExecutionEvidence> {
    let evidence_id: i64 = row.get(0)?;
    let cycle_id: String = row.get(1)?;
    let gate_raw: i64 = row.get(2)?;
    let gauntlet_attempt: i64 = row.get(3)?;
    let actor_role: String = row.get(4)?;
    let kind_raw: String = row.get(5)?;
    let executed_command: String = row.get(6)?;
    let exit_code: i64 = row.get(7)?;
    let duration_ms: i64 = row.get(8)?;
    let sanitized_stdout_tail: String = row.get(9)?;
    let sanitized_stderr_tail: String = row.get(10)?;
    let output_blake3_digest: String = row.get(11)?;
    let verdict_raw: String = row.get(12)?;
    let recorded_at: String = row.get(13)?;

    let gate_stage = GateStage::from_u8(gate_raw as u8).unwrap_or(GateStage::Gate0Discovery);
    let evidence_kind = match kind_raw.as_str() {
        "CMD_EXEC" => EvidenceKind::CmdExec,
        "AST_AUDIT" => EvidenceKind::AstAudit,
        "BOARD_SYNC" => EvidenceKind::BoardSync,
        "SPEC_SEAL" => EvidenceKind::SpecSeal,
        "CODE_REVIEW" => EvidenceKind::CodeReview,
        "SECURITY_REVIEW" => EvidenceKind::SecurityReview,
        _ => EvidenceKind::CriticVerdict,
    };
    let verdict = match verdict_raw.as_str() {
        "PASS" => GateVerdict::Pass,
        "FAIL" => GateVerdict::Fail,
        "INFO" => GateVerdict::Info,
        _ => GateVerdict::Blocked,
    };

    Ok(ExecutionEvidence {
        evidence_id: evidence_id as u64,
        cycle_id,
        gate_stage,
        gauntlet_attempt: gauntlet_attempt as u8,
        actor_role,
        evidence_kind,
        executed_command,
        exit_code: exit_code as i32,
        duration_ms: duration_ms as u64,
        sanitized_stdout_tail,
        sanitized_stderr_tail,
        output_blake3_digest,
        verdict,
        recorded_at,
    })
}
