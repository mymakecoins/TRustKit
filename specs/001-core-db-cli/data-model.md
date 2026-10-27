# Phase 1: Data Model & Domain Entities — `001-core-db-cli`

**Feature**: `001-core-db-cli`
**Date**: 2026-10-27
**Architectural Compliance**: Clean Code Ch. 2 (Ubiquitous Language), Ch. 7 (Zero NULL / Special Cases), Ch. 8 (Ports & Adapters), Ch. 11 (Pure PORS)

---

## 1. Pure Domain Entities (`trustkit-core::entities`)

All domain entities are implemented as Plain Old Rust Structs (PORS) using only `std` types, with zero `Option<T>` for domain state where a Special Case Enum captures business semantics.

### 1.1. `SddCycle`
Represents an isolated Spec-Driven Development execution cycle.

| Field | Domain Type | Description & Invariants |
| :--- | :--- | :--- |
| `cycle_id` | `CycleId` (`String`) | Format: `issue-<id>-YYYYMMDDHHMM` or `no-issue-YYYYMMDDHHMM`. Non-empty. |
| `issue_linkage` | `IssueLinkage` | Special Case Enum: `LinkedIssue` or `UnlinkedLocalCycle`. |
| `methodology_profile` | `MethodologyProfile` | `StandardGauntletV2` or `SpecKitIntegrated { feature_dir }`. |
| `current_gate` | `GateStage` | Current milestone (`Gate0Discovery` .. `Gate6Certification`). |
| `lifecycle_status` | `CycleStatus` | `Active`, `Paused`, `CircuitBreakerTripped`, `Certified`, or `Merged`. |
| `gauntlet_attempt` | `u8` | Bounded integer `0..=5`. Trips Circuit Breaker at `5` (`RN-005`). |
| `spec_seal_state` | `SpecLockState` | `DraftUnverified` or `Frozen { spec_path, blake3_digest }`. |
| `work_branch` | `String` | Non-protected work branch following `feat/<id>-<slug>`, `fix/<id>-<slug>`, `chore/<id>-<slug>`, `marcos/<id>/<slug>`, or active feature branch. |
| `base_commit_sha` | `String` | 40-char Git commit hash at cycle initialization. |
| `pre_existing_dirty_files` | `Vec<String>` | Relative paths of uncommitted files present prior to `init` (`[]` when clean). |
| `created_at` | `String` | ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`). |
| `updated_at` | `String` | ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`). |

### 1.2. `ArtifactSeal`
Represents a cryptographic BLAKE3 signature and structural assertion count captured at Gate 2.

| Field | Domain Type | Description & Invariants |
| :--- | :--- | :--- |
| `seal_id` | `u64` | Unique sequential identifier (`0` before persistence). |
| `cycle_id` | `CycleId` | Foreign reference to owning `SddCycle`. |
| `file_path` | `String` | Repository-relative file path. |
| `artifact_category` | `ArtifactCategory` | `Spec`, `Constitution`, `PreExistingTest`, or `ForeignIssueEvidence`. |
| `blake3_digest` | `String` | 64-char hex BLAKE3 digest. |
| `ast_assertion_count` | `u32` | Count of AST assertion nodes (`expect`, `assert`, `assert_eq!`). |
| `contract_unlock_state` | `ContractUnlockState` | `Sealed` or `UnlockedByHuman { justification }`. |

### 1.3. `TraceabilityLink`
Represents a verified end-to-end traceability tuple ($\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$).

| Field | Domain Type | Description & Invariants |
| :--- | :--- | :--- |
| `trace_id` | `u64` | Unique sequential identifier. |
| `cycle_id` | `CycleId` | Owning cycle identifier. |
| `business_rule_id` | `String` | Business rule identifier (e.g., `RN-001`). |
| `functional_req_id` | `String` | Functional requirement identifier (e.g., `RF-001`). |
| `task_id` | `String` | Atomic task identifier (e.g., `TASK-001`). |
| `mermaid_journey_node` | `String` | Corresponding Mermaid flowchart node identifier. |
| `playwright_spec_path` | `String` | Relative path to E2E verification test. |
| `evidence_screenshot` | `String` | Expected evidence filename. |
| `verification_state` | `TraceVerificationState` | `PendingGate5` or `VerifiedPass`. |

### 1.4. `AtomicTask`
Represents an atomic unit of TDD implementation assigned to an execution wave.

| Field | Domain Type | Description & Invariants |
| :--- | :--- | :--- |
| `cycle_id` | `CycleId` | Owning cycle identifier. |
| `task_id` | `String` | Task identifier (e.g., `TASK-001`). |
| `title` | `String` | Concise imperative title. |
| `architectural_layer` | `String` | Target layer (`core`, `db`, `ast`, `orch`, `gh`, `cli`). |
| `wave_index` | `u32` | Topological wave index (`>= 1`). |
| `depends_on` | `Vec<String>` | Upstream task IDs (`[]` if independent). |
| `expected_files` | `Vec<String>` | Declared target source files (`[]` if none). |
| `target_test_file` | `String` | Mandatory test file verifying this task. |
| `tdd_lifecycle_state` | `TddLifecycleState` | `AwaitingRedPhase`, `RedVerified`, `GreenVerified`, or `Committed`. |
| `worktree_allocation` | `WorktreeAllocation` | `NotProvisioned`, `Provisioned { path }`, or `MergedAndPruned { sha }`. |

### 1.5. `ExecutionEvidence`
Represents an immutable audit log entry ("The Event Log Proves" — `RN-002`).

| Field | Domain Type | Description & Invariants |
| :--- | :--- | :--- |
| `evidence_id` | `u64` | Unique sequential identifier. |
| `cycle_id` | `CycleId` | Owning cycle identifier. |
| `gate_stage` | `GateStage` | Gate in which the evidence was captured (`0..=6`). |
| `gauntlet_attempt` | `u8` | Gauntlet iteration counter (`0..=5`). |
| `actor_role` | `String` | Actor identifier (`ORCHESTRATOR`, `BUILDER:TASK-001`, `AUDITOR:AST`, etc.). |
| `evidence_kind` | `EvidenceKind` | `CmdExec`, `AstAudit`, `BoardSync`, `SpecSeal`, or `CriticVerdict`. |
| `executed_command` | `String` | Literal command executed or `'INTERNAL_AST_EVALUATION'`. |
| `exit_code` | `i32` | Process exit code (`0` for pass). |
| `duration_ms` | `u64` | Execution duration in milliseconds. |
| `sanitized_stdout_tail` | `String` | Secret-redacted stdout tail (`""` when empty). |
| `sanitized_stderr_tail` | `String` | Secret-redacted stderr tail (`""` when empty). |
| `output_blake3_digest` | `String` | BLAKE3 digest of the combined command output. |
| `verdict` | `GateVerdict` | `Pass`, `Fail`, `Info`, or `Blocked`. |
| `recorded_at` | `String` | ISO-8601 UTC timestamp. |

---

## 2. Special Case Algebraic Types (`trustkit-core::special_cases`)

```rust
pub enum IssueLinkage {
    LinkedIssue {
        issue_number: u64,
        issue_url: String,
        project_board_id: String,
        project_item_id: String,
        board_column: BoardColumn,
    },
    UnlinkedLocalCycle {
        directory_slug: String,
    },
}

pub enum SpecLockState {
    DraftUnverified,
    Frozen {
        spec_path: String,
        blake3_digest: String,
    },
}

pub enum TddLifecycleState {
    AwaitingRedPhase,
    RedVerified { failing_evidence_id: u64 },
    GreenVerified { red_evidence_id: u64, green_evidence_id: u64 },
    Committed { commit_sha: String },
}

pub enum BoardSyncOutcome {
    Synchronized { column: BoardColumn },
    SkippedUnlinkedCycle,
    DeferredOffline { queued_transition: BoardColumn, retry_after_ms: u64 },
}
```

---

## 3. State Transitions & Typestate Machine (`trustkit-core::gates_fsm`)

```text
[Gate0Discovery]
       |  (trustkit init -> records SddCycle & PreExistingDirtyFiles)
       v
[Gate1SpecDrafting]
       |  (trustkit spec lock -> verifies Mermaid/Gherkin/Matrix & computes BLAKE3)
       v
[Gate2SpecSealed]
       |  (trustkit wave plan -> validates DAG & groups ExecutionWaves)
       v
[Gate3WavePlanned]
       |  (trustkit wave run -> enforces RED -> GREEN per AtomicTask)
       v
[Gate4TasksImplemented]
       |  (trustkit gauntlet run -> Layer 0 AST + Layer 1 Critics + E2E)
       +-----> [FAIL < 5x] ----> Returns to Gate4TasksImplemented
       +-----> [FAIL == 5x] ---> [CircuitBreakerTripped] (Halts autonomous loop)
       |
       v  (100% PASS)
[Gate5GauntletCleared]
       |  (trustkit done -> renders gauntlet-report.md & posts issue comment)
       v
[Gate6Certified]
```

---

## 4. Relational Schema (`trustkit-db` — SQLite WAL, 100% `NOT NULL`)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

CREATE TABLE IF NOT EXISTS sdd_cycles (
    cycle_id                TEXT PRIMARY KEY NOT NULL,
    issue_linkage_kind      TEXT NOT NULL CHECK (issue_linkage_kind IN ('LINKED_ISSUE', 'UNLINKED_LOCAL_CYCLE')),
    issue_number            INTEGER NOT NULL DEFAULT 0,
    issue_url               TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_board_id        TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_item_id         TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    board_column            TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    methodology_profile     TEXT NOT NULL CHECK (methodology_profile IN ('STANDARD_GAUNTLET_V2', 'SPECKIT_INTEGRATED')),
    current_gate            INTEGER NOT NULL DEFAULT 0 CHECK (current_gate BETWEEN 0 AND 6),
    lifecycle_status        TEXT NOT NULL CHECK (lifecycle_status IN ('ACTIVE', 'PAUSED', 'CIRCUIT_BREAKER_TRIPPED', 'CERTIFIED', 'MERGED')),
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0 CHECK (gauntlet_attempt BETWEEN 0 AND 5),
    spec_dir                TEXT NOT NULL,
    spec_file_path          TEXT NOT NULL,
    spec_lock_state         TEXT NOT NULL DEFAULT 'DRAFT_UNVERIFIED' CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN')),
    spec_blake3_digest      TEXT NOT NULL DEFAULT 'UNSEALED_DRAFT',
    work_branch             TEXT NOT NULL,
    base_commit_sha         TEXT NOT NULL,
    pre_existing_dirty_json TEXT NOT NULL DEFAULT '[]',
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS artifact_seals (
    seal_id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    file_path               TEXT NOT NULL,
    artifact_category       TEXT NOT NULL CHECK (artifact_category IN ('SPEC', 'CONSTITUTION', 'PRE_EXISTING_TEST', 'FOREIGN_ISSUE_EVIDENCE')),
    blake3_digest           TEXT NOT NULL,
    ast_assertion_count     INTEGER NOT NULL DEFAULT 0,
    contract_unlock_state   TEXT NOT NULL DEFAULT 'SEALED' CHECK (contract_unlock_state IN ('SEALED', 'UNLOCKED_BY_HUMAN')),
    unlock_justification    TEXT NOT NULL DEFAULT 'NONE',
    UNIQUE(cycle_id, file_path)
);

CREATE TABLE IF NOT EXISTS traceability_links (
    trace_id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    business_rule_id        TEXT NOT NULL,
    functional_req_id       TEXT NOT NULL,
    task_id                 TEXT NOT NULL,
    mermaid_journey_node    TEXT NOT NULL,
    playwright_spec_path    TEXT NOT NULL,
    evidence_screenshot     TEXT NOT NULL,
    verification_state      TEXT NOT NULL DEFAULT 'PENDING_GATE5' CHECK (verification_state IN ('PENDING_GATE5', 'VERIFIED_PASS'))
);

CREATE TABLE IF NOT EXISTS atomic_tasks (
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    task_id                 TEXT NOT NULL,
    title                   TEXT NOT NULL,
    architectural_layer     TEXT NOT NULL,
    wave_index              INTEGER NOT NULL CHECK (wave_index >= 1),
    depends_on_json         TEXT NOT NULL DEFAULT '[]',
    expected_files_json     TEXT NOT NULL DEFAULT '[]',
    target_test_file        TEXT NOT NULL,
    tdd_lifecycle_state     TEXT NOT NULL DEFAULT 'AWAITING_RED_PHASE' CHECK (tdd_lifecycle_state IN ('AWAITING_RED_PHASE', 'RED_VERIFIED', 'GREEN_VERIFIED', 'COMMITTED')),
    red_evidence_id         INTEGER NOT NULL DEFAULT 0,
    green_evidence_id       INTEGER NOT NULL DEFAULT 0,
    worktree_allocation     TEXT NOT NULL DEFAULT 'NOT_PROVISIONED',
    commit_sha              TEXT NOT NULL DEFAULT 'UNCOMMITTED',
    PRIMARY KEY (cycle_id, task_id)
);

CREATE TABLE IF NOT EXISTS execution_evidences (
    evidence_id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    gate_stage              INTEGER NOT NULL,
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0,
    actor_role              TEXT NOT NULL,
    evidence_kind           TEXT NOT NULL,
    executed_command        TEXT NOT NULL DEFAULT 'INTERNAL_AST_EVALUATION',
    exit_code               INTEGER NOT NULL DEFAULT 0,
    duration_ms             INTEGER NOT NULL,
    sanitized_stdout_tail   TEXT NOT NULL DEFAULT '',
    sanitized_stderr_tail   TEXT NOT NULL DEFAULT '',
    output_blake3_digest    TEXT NOT NULL,
    verdict                 TEXT NOT NULL CHECK (verdict IN ('PASS', 'FAIL', 'INFO', 'BLOCKED')),
    recorded_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_evidences_cycle_gate ON execution_evidences(cycle_id, gate_stage, verdict);
CREATE INDEX IF NOT EXISTS idx_traceability_cycle ON traceability_links(cycle_id, business_rule_id);
```
