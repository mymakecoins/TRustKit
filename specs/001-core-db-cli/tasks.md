---
description: "Task list for Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence"
---

# Tasks: Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence

**Input**: Design documents from `/specs/001-core-db-cli/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/cli-contracts.md, quickstart.md

**Tests**: Included per TRustKit Constitution Principle III (NON-NEGOTIABLE Test-First TDD & Deterministic Equivalence). All tests MUST be written first and verified to fail (`RED`) before implementing production code (`GREEN`).

**Organization**: Tasks are grouped by user story in priority order (`P1`, `P1`, `P2`, `P3`) to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., `[US1]`, `[US2]`, `[US3]`, `[US4]`)
- Exact repository-relative file paths are included in every task description

## Path Conventions

- **Cargo Workspace Root**: `Cargo.toml`
- **Pure Domain Crate (`std` only)**: `crates/trustkit-core/src/`
- **SQLite WAL & BLAKE3 Adapter Crate**: `crates/trustkit-db/src/`
- **GitHub CLI & Git Inspector Adapter Crate**: `crates/trustkit-gh/src/`
- **Phase 2-4 Workspace Scaffolds**: `crates/trustkit-ast/src/`, `crates/trustkit-orch/src/`
- **CLI Binary & Composition Root Crate**: `crates/trustkit-cli/src/` and `crates/trustkit-cli/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Multi-crate Cargo Workspace initialization and compile-time hexagonal boundary scaffolding.

- [X] T001 Configure Cargo Workspace root (`resolver = "2"`, release profile with `lto = "thin"` and `strip = true`, and workspace members) in `Cargo.toml`
- [X] T002 [P] Initialize zero-dependency pure domain crate manifest (Rust Edition 2024, empty `[dependencies]`) in `crates/trustkit-core/Cargo.toml` and module declarations in `crates/trustkit-core/src/lib.rs`
- [X] T003 [P] Initialize SQLite WAL and BLAKE3 adapter crate manifest (`rusqlite` with `bundled`, `blake3`, `rayon`) in `crates/trustkit-db/Cargo.toml` and module declarations in `crates/trustkit-db/src/lib.rs`
- [X] T004 [P] Initialize GitHub CLI and Git worktree adapter crate manifest (`serde`, `serde_json`) in `crates/trustkit-gh/Cargo.toml` and module declarations in `crates/trustkit-gh/src/lib.rs`
- [X] T005 [P] Initialize workspace member scaffolds for Phase 2-4 crates in `crates/trustkit-ast/Cargo.toml`, `crates/trustkit-ast/src/lib.rs`, `crates/trustkit-orch/Cargo.toml`, and `crates/trustkit-orch/src/lib.rs`
- [X] T006 [P] Initialize CLI binary crate manifest (`clap` derive, `serde`, `serde_json`, `tokio`, and `[dev-dependencies]` `assert_cmd`, `predicates`, `tempfile`) in `crates/trustkit-cli/Cargo.toml` and command module exports in `crates/trustkit-cli/src/commands/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core domain entities, Special Case algebraic types, 7-Gate Typestate FSM, Port traits, error hierarchy, CLI contracts, and Composition Root foundation that MUST be complete before user story implementation begins.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T007 [P] Implement Special Case algebraic types (`IssueLinkage` with `LinkedIssue` or `UnlinkedLocalCycle`, `SpecLockState` with `DraftUnverified` or `Frozen { spec_path, blake3_digest }`, `TddLifecycleState` with `AwaitingRedPhase`, `RedVerified`, `GreenVerified`, or `Committed`, `BoardSyncOutcome` with `Synchronized`, `SkippedUnlinkedCycle`, or `DeferredOffline`, `ContractUnlockState` with `Sealed` or `UnlockedByHuman { justification }`, `TraceVerificationState` with `PendingGate5` or `VerifiedPass`, `WorktreeAllocation` with `NotProvisioned`, `Provisioned { path }`, or `MergedAndPruned { sha }`, `ActiveCycleLookup`, and `IdentityVerification`) in `crates/trustkit-core/src/special_cases.rs`
- [X] T008 [P] Implement `DomainViolation` error enum and POSIX exit code categorization (`0`=`CleanPass`, `1`=`InfrastructureBoundaryFault`, `2`=`SpecOrTraceabilityIncomplete`, `3`=`TddLifecycleViolation`, `4`=`AstIntegrityOrInvariantViolation`, `5`=`CriticSwarmOrPlaywrightFailed`, `6`=`CircuitBreakerTripped`) in `crates/trustkit-core/src/errors.rs`
- [X] T009 Implement PORS domain entities (`SddCycle`, `ArtifactSeal`, `TraceabilityLink`, `AtomicTask`, `ExecutionEvidence`) enforcing verbatim data-model constraints (`cycle_id`: "Format: `issue-<id>-YYYYMMDDHHMM` or `no-issue-YYYYMMDDHHMM`. Non-empty."; `methodology_profile`: "`StandardGauntletV2` or `SpecKitIntegrated { feature_dir }`."; `current_gate`: "Current milestone (`Gate0Discovery` .. `Gate6Certification`)."; `lifecycle_status`: "`Active`, `Paused`, `CircuitBreakerTripped`, `Certified`, or `Merged`."; `gauntlet_attempt`: "Bounded integer `0..=5`. Trips Circuit Breaker at `5` (`RN-005`)."; `work_branch`: "Non-protected work branch following `feat/<id>-<slug>`, `fix/<id>-<slug>`, `chore/<id>-<slug>`, or active feature branch."; `base_commit_sha`: "40-char Git commit hash at cycle initialization."; `pre_existing_dirty_files`: "Relative paths of uncommitted files present prior to `init` (`[]` when clean)."; `created_at`/`updated_at`: "ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`)."; `seal_id`: "Unique sequential identifier (`0` before persistence)."; `artifact_category`: "`Spec`, `Constitution`, `PreExistingTest`, or `ForeignIssueEvidence`."; `blake3_digest`: "64-char hex BLAKE3 digest."; `ast_assertion_count`: "Count of AST assertion nodes (`expect`, `assert`, `assert_eq!`)."; `architectural_layer`: "Target layer (`core`, `db`, `ast`, `orch`, `gh`, `cli`)."; `wave_index`: "Topological wave index (`>= 1`)."; `depends_on`: "Upstream task IDs (`[]` if independent)."; `expected_files`: "Declared target source files (`[]` if none)."; `gate_stage`: "Gate in which the evidence was captured (`0..=6`)."; `actor_role`: "Actor identifier (`ORCHESTRATOR`, `BUILDER:TASK-001`, `AUDITOR:AST`, etc.)."; `evidence_kind`: "`CmdExec`, `AstAudit`, `BoardSync`, `SpecSeal`, or `CriticVerdict`."; `executed_command`: "Literal command executed or `'INTERNAL_AST_EVALUATION'`."; `exit_code`: "Process exit code (`0` for pass)."; `sanitized_stdout_tail`: "Secret-redacted stdout tail (`\"\"` when empty)."; `sanitized_stderr_tail`: "Secret-redacted stderr tail (`\"\"` when empty)."; `verdict`: "`Pass`, `Fail`, `Info`, or `Blocked`.") in `crates/trustkit-core/src/entities.rs`
- [X] T010 [P] Implement 7-Gate Typestate Machine (`Gate0Discovery` through `Gate6Certification` with `CircuitBreakerTripped` transition at `gauntlet_attempt == 5`) in `crates/trustkit-core/src/gates_fsm.rs`
- [X] T011 Implement domain Port traits (`CycleEvidenceRepositoryPort`, `ArtifactHasherPort`, `ProjectBoardGatewayPort`) in `crates/trustkit-core/src/ports.rs` and export use case modules in `crates/trustkit-core/src/use_cases/mod.rs`
- [X] T012 [P] Implement `Blake3ParallelSealerAdapter` (`ArtifactHasherPort` implementation computing 64-char hex BLAKE3 digests over files and byte slices using `blake3` and `rayon`) in `crates/trustkit-db/src/blake3_adapter.rs`
- [X] T013 [P] Implement Clap v4 CLI command and flag definitions (`init [--issue <NUMBER_OR_URL>] [--from-prd <PATH>] [--json]`, `status [--json]`, `hook pre-tool --command <SHELL_COMMAND_STRING> [--json]`) in `crates/trustkit-cli/src/cli_contract.rs`
- [X] T014 [P] Implement `CliBoundaryErrorHandler` translating `Result<DomainOutcome, DomainViolation>` into POSIX exit codes `0..=6` and structured `stderr` diagnostics in `crates/trustkit-cli/src/error_handler.rs`
- [X] T015 Implement Composition Root (`FastSyncBootstrap` without `tokio` for `hook pre-tool` and `status`, plus `FullConcurrentBootstrap` for `init`) in `crates/trustkit-cli/src/bootstrap.rs` and CLI entrypoint dispatch in `crates/trustkit-cli/src/main.rs`

**Checkpoint**: Foundation ready - user story implementation can now proceed.

---

## Phase 3: User Story 1 - Deterministic Cycle Initialization & Board Synchronization (Priority: P1) - MVP

**Goal**: Provide a deterministic `trustkit init` command that inspects repository state, verifies operator identity (`RN-014`), preserves pre-existing dirty files without running `git stash`, creates a cycle directory (`docs/specs/issue-<id>-YYYYMMDDHHMM/` or `docs/specs/no-issue-YYYYMMDDHHMM/`), synchronizes the GitHub Project board to `"In progress"`, and persists the Gate 0 session in under 250ms.

**Independent Test**: Run `trustkit init --issue 45 --json` and `trustkit init --json` in an isolated temporary Git repository with pre-existing modified files; verify that modified files remain untouched, the cycle directory is created without blocking `stdin` in `--json` mode, the session is persisted, and the command exits with code `0` in `< 250ms`.

### Tests for User Story 1 (MANDATORY TDD - write first, ensure RED before GREEN)

- [X] T016 [P] [US1] Write failing in-memory unit tests for `initialize_cycle` use case (linked issue initialization, unlinked `no-issue-YYYYMMDDHHMM` fallback without blocking `stdin` in `--json` mode, `--from-prd` path resolution, pre-existing dirty files recorded without stash, invalid operator identity rejection, and offline board transition queueing) in `crates/trustkit-core/src/use_cases/initialize_cycle.rs`
- [X] T017 [P] [US1] Write failing E2E tests for `trustkit init` (linked cycle, unlinked local cycle in `--json` mode, dirty file preservation, detached HEAD / non-Git repository failure with exit code `1`) in `crates/trustkit-cli/tests/e2e_init_and_status.rs`

### Implementation for User Story 1

- [X] T018 [P] [US1] Implement `initialize_cycle` use case orchestrating Gate 0 `SddCycle` construction, optional `--from-prd <PATH>` resolution into `spec_file_path` (defaulting to `<spec_dir>/spec.md`), dirty file baseline capture, `ProjectBoardGatewayPort` identity check, board synchronization, and initial `ExecutionEvidence` recording in `crates/trustkit-core/src/use_cases/initialize_cycle.rs`
- [X] T019 [P] [US1] Implement local Git worktree inspector (active branch resolution supporting `feat|fix|chore/<id>-<slug>` and `<operator>/<id>/<slug>`, 40-char `HEAD` commit SHA, dirty working tree file discovery without `git stash`, and detached HEAD / missing repository validation) in `crates/trustkit-gh/src/git_inspector.rs`
- [X] T020 [P] [US1] Implement `GitHubIdentityGuard` verifying active `gh auth status` operator identity (`RN-014`) in `crates/trustkit-gh/src/identity_guard.rs`
- [X] T021 [US1] Implement `ProjectBoardSynchronizer` (`ProjectBoardGatewayPort` implementation) executing 9-column GitHub Projects v2 transitions (`"In progress"`) with `BoardSyncOutcome::DeferredOffline` fallback on network failure in `crates/trustkit-gh/src/board_synchronizer.rs`
- [X] T022 [US1] Implement `trustkit init` command handler (creating `docs/specs/issue-<id>-YYYYMMDDHHMM/` or `docs/specs/no-issue-YYYYMMDDHHMM/` directory, resolving optional `--from-prd <PATH>` into `spec_file_path` or defaulting to `<spec_dir>/spec.md`, invoking `initialize_cycle` without blocking `stdin` in `--json` mode, and wiring route in `bootstrap.rs`) in `crates/trustkit-cli/src/commands/init_cmd.rs`

**Checkpoint**: User Story 1 domain, adapters, and command handler are complete and independently verifiable.

---

## Phase 4: User Story 2 - Sub-Millisecond Pre-Tool Hook Interception (Priority: P1)

**Goal**: Provide a synchronous `trustkit hook pre-tool --command "<CMD>"` inspector that evaluates shell commands on the fast bootstrap path (no `tokio`) in `< 3ms` (p99), blocking destructive or non-compliant commands (`RN-006`, `RN-007`, `RN-012`, `RN-013`) with exit code `4`.

**Independent Test**: Invoke `trustkit hook pre-tool --command "git stash" --json` and `trustkit hook pre-tool --command "cargo test" --json`, verifying exit code `4` (`BLOCKED`) for forbidden patterns and exit code `0` (`PASS`) for safe commands in under 3 milliseconds.

### Tests for User Story 2 (MANDATORY TDD - write first, ensure RED before GREEN)

- [X] T023 [P] [US2] Write failing in-memory unit tests for pre-tool command policy evaluation (`git stash` except `list`/`show`, `git add .`, `git add -A`, `git add --all`, `git reset --hard`, `git clean -fd`, direct commit/push to `main`/`staging`/`develop`, autonomous `gh pr create`, `Co-Authored-By:` trailers, Unicode emojis, and safe scoped commands) in `crates/trustkit-core/src/invariants_policy.rs`
- [X] T024 [P] [US2] Write failing E2E binary tests for `trustkit hook pre-tool` verifying exit code `4`, null-free JSON violation payload, exit code `0` approval payload, and `< 3ms` execution budget in `crates/trustkit-cli/tests/e2e_hook_interception.rs`

### Implementation for User Story 2

- [X] T025 [US2] Implement pure Hard Invariants policy evaluators (`RN-001`..`RN-016` command pattern matching, protected branch guard, AI trailer detector, and Unicode emoji range scanner) in `crates/trustkit-core/src/invariants_policy.rs`
- [X] T026 [US2] Implement `inspect_pre_tool_command` synchronous use case returning `CleanPass` or `DomainViolation::AstIntegrityOrInvariantViolation` with `invariant_id`, `intercepted_command`, `reason`, and `remediation` in `crates/trustkit-core/src/use_cases/inspect_pre_tool_command.rs`
- [X] T027 [US2] Implement `trustkit hook pre-tool` command handler executing directly on the synchronous `FastSyncBootstrap` path without async runtime allocation in `crates/trustkit-cli/src/commands/hook_cmd.rs`

**Checkpoint**: User Story 2 pre-tool hook interception is complete and verifiable in under 3ms.

---

## Phase 5: User Story 3 - Resilient Local Event Persistence & Dual-Audience Output (Priority: P2)

**Goal**: Persist all cycle state and execution evidence in an embedded SQLite 3 WAL database (`.trustkit/state.db`) enforcing `100% NOT NULL` columns and explicit Special Case defaults, paired with a dual-audience `Presenter` emitting null-free JSON (`--json`) for agents and emoji-free ASCII tables for humans.

**Independent Test**: Run concurrent writes against `SqliteWalRepositoryAdapter` in a temporary directory and render outputs through `Presenter` in both `--json` and human TTY modes, verifying zero `SQLITE_BUSY` errors, zero `: null` fields in JSON, and zero Unicode emojis in terminal output.

### Tests for User Story 3 (MANDATORY TDD - write first, ensure RED before GREEN)

- [X] T028 [P] [US3] Write failing integration tests for SQLite WAL schema initialization, `100% NOT NULL` constraints, concurrent evidence writes, and Special Case round-trip mapping in `crates/trustkit-db/src/sqlite_adapter.rs`
- [X] T029 [P] [US3] Write failing unit tests for `Presenter` verifying zero `: null` substrings across all JSON responses and zero Unicode emojis in human ASCII table mode in `crates/trustkit-cli/src/presenter.rs`

### Implementation for User Story 3

- [X] T030 [P] [US3] Implement SQLite WAL pragma initialization (`PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;`) and 5-table `100% NOT NULL` DDL schema (`sdd_cycles` with `CHECK (issue_linkage_kind IN ('LINKED_ISSUE', 'UNLINKED_LOCAL_CYCLE'))`, `DEFAULT 'UNLINKED_LOCAL_CYCLE'`, `CHECK (methodology_profile IN ('STANDARD_GAUNTLET_V2', 'SPECKIT_INTEGRATED'))`, `CHECK (current_gate BETWEEN 0 AND 6)`, `CHECK (lifecycle_status IN ('ACTIVE', 'PAUSED', 'CIRCUIT_BREAKER_TRIPPED', 'CERTIFIED', 'MERGED'))`, `CHECK (gauntlet_attempt BETWEEN 0 AND 5)`, `CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN'))`, `spec_blake3_digest TEXT NOT NULL DEFAULT 'UNSEALED_DRAFT'`, `pre_existing_dirty_json TEXT NOT NULL DEFAULT '[]'`; `artifact_seals` with `CHECK (artifact_category IN ('SPEC', 'CONSTITUTION', 'PRE_EXISTING_TEST', 'FOREIGN_ISSUE_EVIDENCE'))`, `CHECK (contract_unlock_state IN ('SEALED', 'UNLOCKED_BY_HUMAN'))`, `unlock_justification TEXT NOT NULL DEFAULT 'NONE'`, `UNIQUE(cycle_id, file_path)`; `traceability_links` with `CHECK (verification_state IN ('PENDING_GATE5', 'VERIFIED_PASS'))`; `atomic_tasks` with `CHECK (wave_index >= 1)`, `depends_on_json TEXT NOT NULL DEFAULT '[]'`, `expected_files_json TEXT NOT NULL DEFAULT '[]'`, `CHECK (tdd_lifecycle_state IN ('AWAITING_RED_PHASE', 'RED_VERIFIED', 'GREEN_VERIFIED', 'COMMITTED'))`, `worktree_allocation TEXT NOT NULL DEFAULT 'NOT_PROVISIONED'`, `commit_sha TEXT NOT NULL DEFAULT 'UNCOMMITTED'`, `PRIMARY KEY (cycle_id, task_id)`; `execution_evidences` with `executed_command TEXT NOT NULL DEFAULT 'INTERNAL_AST_EVALUATION'`, `sanitized_stdout_tail TEXT NOT NULL DEFAULT ''`, `sanitized_stderr_tail TEXT NOT NULL DEFAULT ''`, `CHECK (verdict IN ('PASS', 'FAIL', 'INFO', 'BLOCKED'))`; plus indexes `idx_evidences_cycle_gate` and `idx_traceability_cycle`) in `crates/trustkit-db/src/migrations.rs`
- [X] T031 [P] [US3] Implement bidirectional row mappers converting between SQLite `NOT NULL` sentinel values and `trustkit-core` entities/Special Case enums in `crates/trustkit-db/src/row_mappers.rs`
- [X] T032 [US3] Implement `SqliteWalRepositoryAdapter` (`CycleEvidenceRepositoryPort` implementation) with transactional cycle persistence, artifact seal storage, immutable evidence appending, and boundary error mapping in `crates/trustkit-db/src/sqlite_adapter.rs`
- [X] T033 [P] [US3] Implement dual-audience `Presenter` supporting Agent JSON mode (`--json` or `TRUSTKIT_AGENT_MODE=1` with zero `: null` occurrences) and Human ASCII mode (`[PASS]`, `[FAIL]`, `[GATE-0]`..`[GATE-6]` prefixes and zero emojis) in `crates/trustkit-cli/src/presenter.rs`

**Checkpoint**: SQLite WAL persistence and dual-audience presentation are complete and independently verifiable.

---

## Phase 6: User Story 4 - Cycle Inspection & Health Diagnostic Querying (Priority: P3)

**Goal**: Provide a fast `trustkit status` command (`< 3ms` p99) that queries and displays active cycle health, gate milestone, branch compliance, and tracked pre-existing dirty files, or gracefully reports `NO_ACTIVE_CYCLE` with exit code `0` when no cycle is initialized.

**Independent Test**: Run `trustkit status` and `trustkit status --json` in both an uninitialized repository and an active Gate 0 repository, verifying accurate reporting of cycle ID, gate stage, branch name, and dirty baseline in `< 3ms` with exit code `0`.

### Tests for User Story 4 (MANDATORY TDD - write first, ensure RED before GREEN)

- [X] T034 [P] [US4] Write failing in-memory unit tests for `query_cycle_status` use case covering `ActiveCycleLookup::Found(SddCycle)` and `ActiveCycleLookup::NoActiveCycle` Special Case in `crates/trustkit-core/src/use_cases/query_cycle_status.rs`
- [X] T035 [P] [US4] Extend E2E test suite with `trustkit status` scenarios (active Gate 0 cycle status in `--json` and ASCII modes, and uninitialized repository graceful exit code `0`) in `crates/trustkit-cli/tests/e2e_init_and_status.rs`

### Implementation for User Story 4

- [X] T036 [US4] Implement `query_cycle_status` use case inspecting active cycle state via `CycleEvidenceRepositoryPort` in `crates/trustkit-core/src/use_cases/query_cycle_status.rs`
- [X] T037 [US4] Implement `trustkit status` command handler invoking `query_cycle_status` on `FastSyncBootstrap` and formatting output via `Presenter` in `crates/trustkit-cli/src/commands/status_cmd.rs`

**Checkpoint**: All 4 user stories are wired through the Composition Root and independently testable end-to-end.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Architectural governance enforcement, line-count modularity audit, zero-panic audit, and release binary performance verification.

- [X] T038 [P] Audit all `.rs` source files across `crates/` to verify `< 250` LOC per file, zero `.unwrap()`/`.expect()`/`panic!()`/`todo!()` outside `#[cfg(test)]`, zero Unicode emojis, and zero prohibited class suffixes (`Manager`, `Processor`, `Helper`, `Util`, `Data`, `Info`) starting from `crates/trustkit-core/src/lib.rs`
- [X] T039 [P] Verify hexagonal dependency isolation and strict linting by running `cargo tree -p trustkit-core` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` against `Cargo.toml`
- [X] T040 Execute full release build (`cargo build --workspace --release`) and validate all E2E and performance scenarios (`< 3ms` hook/status cold start, `< 250ms` init, zero `: null` JSON output) per `specs/001-core-db-cli/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1 completion - BLOCKS all user stories. Includes Composition Root (`T015`) so command handlers can wire routes incrementally.
- **Phase 3 (US1 - Cycle Initialization, P1)**: Depends on Phase 2 completion for domain/adapter unit tests; E2E binary tests (`T017`) run against the SQLite adapter and `Presenter` from Phase 5 (`US3`).
- **Phase 4 (US2 - Pre-Tool Hook Interception, P1)**: Depends on Phase 2 completion; completely self-contained and can run in parallel with US1, US3, and US4.
- **Phase 5 (US3 - SQLite WAL Persistence & Dual Presenter, P2)**: Depends on Phase 2 completion; provides concrete `SqliteWalRepositoryAdapter` (`T032`) and `Presenter` (`T033`).
- **Phase 6 (US4 - Cycle Status Inspection, P3)**: Depends on Phase 2 and Phase 5 (`SqliteWalRepositoryAdapter` & `Presenter`).
- **Phase 7 (Polish)**: Depends on all user stories (Phases 3-6) being complete.
