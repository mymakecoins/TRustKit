# Phase 1: Interface & Boundary Contracts — `001-core-db-cli`

**Feature**: `001-core-db-cli`
**Date**: 2026-10-27
**Architectural Compliance**: Clean Code Ch. 7 (Boundary Error Translation, Zero NULL JSON), Ch. 8 (Ports & Adapters)

---

## 1. POSIX Exit Code Contract (`CliBoundaryErrorHandler`)

Every CLI invocation maps `Result<DomainOutcome, DomainViolation>` deterministically to the following POSIX exit codes:

| Exit Code | Domain Error Category | Deterministic Meaning for Agent / Harness |
| :---: | :--- | :--- |
| `0` | `CleanPass` / `Success` | Operation completed with 100% compliance. |
| `1` | `InfrastructureBoundaryFault` | Local I/O, Git, or SQLite boundary fault translated at adapter edge. |
| `2` | `SpecOrTraceabilityIncomplete` | Spec V2 missing Mermaid/Gherkin or open traceability matrix (`RN-001`). |
| `3` | `TddLifecycleViolation` | Attempt to complete task without prior `RED` phase failure evidence (`RN-002`). |
| `4` | `AstIntegrityOrInvariantViolation` | Layer 0 violation: test dilution, emoji, AI trailer, forbidden command (`RN-004`, `RN-006`, `RN-007`, `RN-013`). |
| `5` | `CriticSwarmOrPlaywrightFailed` | Layer 1 Gauntlet failed (Lint/Build, E2E, or Critic Swarm); eligible for retry `< 5`. |
| `6` | `CircuitBreakerTripped` | 5th consecutive Gauntlet failure reached (`RN-005`). Autonomous loop locked; requires human intervention. |

---

## 2. CLI Subcommand Contracts (Phase 1 Scope)

### 2.1. `trustkit init`
Initializes or resumes an SDD cycle at Gate 0.

**Invocation:**
```bash
trustkit init [--issue <NUMBER_OR_URL>] [--from-prd <PATH>] [--json]
```

**Arguments & Behavior:**
- `--issue <NUMBER_OR_URL>`: Optional GitHub issue number or URL. When omitted on an unlinked branch, initializes an `UNLINKED_LOCAL_CYCLE` (`no-issue-YYYYMMDDHHMM`) without blocking `stdin`.
- `--from-prd <PATH>`: Optional source PRD/spec path recorded in `sdd_cycles.spec_file_path` (defaults to `<spec_dir>/spec.md` when omitted).
- `--json`: Emits deterministic, null-free JSON output (`TRUSTKIT_AGENT_MODE=1`).

**JSON Response Contract (`--json`, Exit Code `0` — Linked Issue):**
```json
{
  "verdict": "PASS",
  "gate": "GATE_0_DISCOVERY",
  "cycle_id": "issue-45-202610271415",
  "methodology_profile": "SPECKIT_INTEGRATED",
  "spec_dir": "docs/specs/issue-45-202610271415",
  "spec_file_path": "docs/specs/issue-45-202610271415/spec.md",
  "work_branch": "feat/45-core-db-cli",
  "issue_linkage": {
    "kind": "LINKED_ISSUE",
    "issue_number": 45,
    "issue_url": "https://github.com/TibaldiHolding/TRustKit/issues/45",
    "board_column": "In progress",
    "sync_outcome": "SYNCHRONIZED"
  },
  "pre_existing_dirty_files": []
}
```

**JSON Response Contract (`--json`, Exit Code `0` — Unlinked Local Cycle Special Case):**
```json
{
  "verdict": "PASS",
  "gate": "GATE_0_DISCOVERY",
  "cycle_id": "no-issue-202610271415",
  "methodology_profile": "SPECKIT_INTEGRATED",
  "spec_dir": "docs/specs/no-issue-202610271415",
  "spec_file_path": "docs/specs/no-issue-202610271415/spec.md",
  "work_branch": "001-core-db-cli",
  "issue_linkage": {
    "kind": "UNLINKED_LOCAL_CYCLE",
    "directory_slug": "no-issue-202610271415",
    "sync_outcome": "SKIPPED_UNLINKED_CYCLE"
  },
  "pre_existing_dirty_files": [
    "README.md"
  ]
}
```

---

### 2.2. `trustkit status`
Queries the active cycle state and health metrics in `< 3ms`.

**Invocation:**
```bash
trustkit status [--json]
```

**JSON Response Contract (`--json`, Active Cycle):**
```json
{
  "verdict": "PASS",
  "cycle_state": {
    "kind": "ACTIVE_CYCLE",
    "cycle_id": "issue-45-202610271415",
    "current_gate": 0,
    "gate_label": "GATE_0_DISCOVERY",
    "lifecycle_status": "ACTIVE",
    "gauntlet_attempt": 0,
    "spec_lock_state": {
      "kind": "DRAFT_UNVERIFIED"
    },
    "work_branch": "001-core-db-cli",
    "pre_existing_dirty_files": []
  }
}
```

**JSON Response Contract (`--json`, No Active Cycle Special Case — Exit Code `0`):**
```json
{
  "verdict": "PASS",
  "cycle_state": {
    "kind": "NO_ACTIVE_CYCLE",
    "remediation": "Run 'trustkit init [--issue <ID>]' to initialize a new SDD cycle."
  }
}
```

**Human TTY Output Contract (Default):**
```text
[PASS] [GATE-0] Active Cycle Status
------------------------------------------------------------
Cycle ID          : issue-45-202610271415
Current Gate      : 0 (GATE_0_DISCOVERY)
Lifecycle Status  : ACTIVE
Gauntlet Attempts : 0 / 5
Spec Lock State   : DRAFT_UNVERIFIED
Work Branch       : 001-core-db-cli
Dirty Baseline    : 0 files
```

---

### 2.3. `trustkit hook pre-tool`
Intercepts and validates shell commands before execution on the Fast Synchronous Bootstrap path (`< 3ms`, no `tokio`).

**Invocation:**
```bash
trustkit hook pre-tool --command "<SHELL_COMMAND_STRING>" [--json]
```

**Blocked Command Patterns (`RN-006`, `RN-012`, `RN-013`):**
- `git stash` (any subcommand other than `list`/`show`)
- `git add .` or `git add -A` or `git add --all`
- `git reset --hard` or `git clean -fd`
- `git commit` containing `Co-Authored-By:` or Unicode Emoji blocks
- `git push` or direct `git commit` onto `main` or `staging`
- `gh pr create` invoked outside explicit `trustkit pr` workflow

**JSON Response Contract (`--json`, Exit Code `4` — Blocked):**
```json
{
  "verdict": "BLOCKED",
  "exit_code": 4,
  "violation": {
    "kind": "FORBIDDEN_COMMAND_INTERCEPTED",
    "invariant_id": "RN-006",
    "intercepted_command": "git add .",
    "reason": "Sweeping stage operations ('git add .' / 'git add -A') are prohibited in shared trees.",
    "remediation": "Stage only explicit task files: git add <file1> <file2>"
  }
}
```

**JSON Response Contract (`--json`, Exit Code `0` — Approved):**
```json
{
  "verdict": "PASS",
  "exit_code": 0,
  "evaluated_command": "cargo test -p trustkit-core"
}
```

---

## 3. Domain Port Traits Contract (`trustkit-core::ports`)

```rust
pub trait CycleEvidenceRepositoryPort {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation>;
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation>;
    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation>;
    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation>;
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation>;
    fn list_evidences_for_cycle(&self, cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation>;
}

pub trait ArtifactHasherPort {
    fn compute_file_blake3(&self, path: &str) -> Result<String, DomainViolation>;
    fn compute_bytes_blake3(&self, bytes: &[u8]) -> String;
}

pub trait ProjectBoardGatewayPort {
    fn verify_active_identity(&self, expected_handle: &str) -> Result<IdentityVerification, DomainViolation>;
    fn transition_issue_column(&self, issue_number: u64, target: BoardColumn) -> Result<BoardSyncOutcome, DomainViolation>;
}
```
