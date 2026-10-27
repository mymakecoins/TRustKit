# Implementation Plan: Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence

**Branch**: `001-core-db-cli` | **Date**: 2026-10-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-core-db-cli/spec.md` and architecture decision record `/docs/ADR_001_TRustKit_Architecture.md`

## Summary

Implement the Phase 1 foundation of **TRustKit** (`trustkit` static native binary in Rust 2024 Edition) following a strict Hexagonal Architecture (Ports & Adapters). This phase establishes:
1. **`trustkit-core`**: Zero-dependency pure domain crate (`std` only) containing Plain Old Rust Structs (PORS), Special Case algebraic enums (zero `NULL`), the 7-Gate Typestate FSM, the 16 Hard Invariants policy, rich `DomainViolation` error hierarchy, and Port traits.
2. **`trustkit-db`**: Embedded SQLite 3 persistence adapter (`rusqlite` bundled) operating in `WAL` mode with `100% NOT NULL` columns, paired with parallel `BLAKE3` cryptographic hashing (`Blake3ParallelSealerAdapter`).
3. **`trustkit-gh`**: GitHub CLI (`gh`) adapter implementing operator identity verification (`marcos-rodoind` — `RN-014`) and 9-column GitHub Projects v2 synchronization (`RN-009`) with offline queue resilience.
4. **`trustkit-cli`**: Single Composition Root (`bootstrap.rs`) with dual execution routes (Fast Synchronous path `< 3ms` without `tokio` for `hook pre-tool` and `status`, plus concurrent path for `init`), central `CliBoundaryErrorHandler` (mapping domain errors to POSIX exit codes `0..=6`), and dual-audience `Presenter` (null-free `--json` vs. emoji-free ASCII tables).

## Technical Context

**Language/Version**: Rust `1.82+` (Edition 2024)

**Primary Dependencies**:
- `trustkit-core`: **None** (`std` only — enforced via `cargo tree -p trustkit-core`)
- `trustkit-db`: `rusqlite` (`0.32+`, `bundled`), `blake3` (`1.5+`), `rayon` (`1.10+`)
- `trustkit-gh`: `serde` (`1.0+`), `serde_json` (`1.0+`)
- `trustkit-cli`: `clap` (`4.5+`, derive), `serde`, `serde_json`, `tokio` (`1.40+`, restricted to concurrent commands)
- Dev/Test (`[dev-dependencies]`): `assert_cmd`, `predicates`, `tempfile`

**Storage**: Embedded SQLite 3 at `.trustkit/state.db` configured with `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`, `PRAGMA busy_timeout = 5000;`, and `NOT NULL` across 100% of columns.

**Testing**: `cargo test --workspace` (in-memory F.I.R.S.T. unit tests in `< 50ms` for `trustkit-core`, temporary directory SQLite integration tests for `trustkit-db`, and `assert_cmd` E2E binary tests in `tests/e2e_*`).

**Target Platform**: Linux (`x86_64-unknown-linux-gnu` / `musl`) and macOS (`aarch64-apple-darwin`) native CLI binary.

**Project Type**: Multi-crate Cargo Workspace producing a single static CLI binary (`trustkit`).

**Performance Goals**:
- Cold-start & execution of `trustkit hook pre-tool` and `trustkit status`: `< 3ms` (p99).
- Complete execution of `trustkit init` (local Git discovery + SQLite write + concurrent board check): `< 250ms`.
- In-memory unit test suite execution (`trustkit-core`): `< 50ms`.

**Constraints**:
- Zero external crates in `trustkit-core`.
- Zero `NULL` columns in SQLite schema and zero `: null` fields in `--json` output payloads.
- Zero `.unwrap()`, `.expect()`, `panic!()`, or `todo!()` in production code.
- Maximum 250 lines of code per `.rs` source file; zero generic suffixes (`Manager`, `Processor`, `Helper`, `Util`, `Data`, `Info`).
- Absolute prohibition of Unicode emojis and AI attribution trailers (`Co-Authored-By:`).

**Scale/Scope**: 4 active crates in Phase 1 (`trustkit-core`, `trustkit-db`, `trustkit-gh`, `trustkit-cli`) plus workspace scaffolding for Phase 2–4 crates (`trustkit-ast`, `trustkit-orch`), 5 relational tables, and 3 primary CLI subcommands (`init`, `status`, `hook pre-tool`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked and verified after Phase 1 design.*

| Constitutional Principle | Pre-Design Check | Post-Design Check | Verification Mechanism |
| :--- | :---: | :---: | :--- |
| **I. Strict Hexagonal Isolation (`trustkit-core` std-only)** | PASS | PASS | `trustkit-core/Cargo.toml` has empty `[dependencies]`; audited via `cargo tree -p trustkit-core`. |
| **II. Absolute Prohibition of NULL & Special Cases** | PASS | PASS | All 5 SQLite tables in `data-model.md` enforce `NOT NULL` with explicit Special Case defaults; JSON contracts in `contracts/cli-contracts.md` contain zero `: null`. |
| **III. Test-First, Anti-Cheat & Deterministic Equivalence** | PASS | PASS | TDD cycle enforced; `cargo clippy -- -D warnings`, `cargo test --workspace`, and `assert_cmd` E2E suites planned. |
| **IV. Resilient Error Handling & Zero-Panic Policy** | PASS | PASS | `DomainViolation` enum in `trustkit-core::errors`; adapter boundary translation; `CliBoundaryErrorHandler` maps to POSIX codes `0..=6`. |
| **V. Separation of Construction and Execution** | PASS | PASS | Dependency wiring isolated in `trustkit-cli::bootstrap`; synchronous fast path bypasses `tokio` for `< 3ms` hook execution. |
| **VI. Ubiquitous Language & Modular Clean Code (< 250 LOC)** | PASS | PASS | Domain terms follow ADR-001 glossary (`SddCycle`, `ArtifactSeal`, `ExecutionEvidence`); modules split into focused `< 250` LOC files. |
| **VII. Cryptographic Immutability & Zero Emojis / Zero AI Trailers** | PASS | PASS | `BLAKE3` sealing via `ArtifactHasherPort`; `hook pre-tool` blocks emojis and `Co-Authored-By`; outputs use ASCII `[PASS]`/`[FAIL]`. |

## Project Structure

### Documentation (this feature)

```text
specs/001-core-db-cli/
├── plan.md              # This implementation plan
├── research.md          # Phase 0 architectural & technical decisions
├── data-model.md        # Phase 1 domain entities, Special Cases, FSM & SQLite DDL
├── quickstart.md        # Phase 1 build, test, and E2E validation scenarios
├── contracts/
│   └── cli-contracts.md # Phase 1 CLI subcommands, exit codes & JSON schemas
└── checklists/
    └── requirements.md  # Specification quality validation checklist
```

### Source Code (repository root)

```text
Cargo.toml                                  # Cargo Workspace root (resolver = "2", lto = "thin", strip = true)
crates/
├── trustkit-core/                          # Pure Domain (ZERO external dependencies, 100% PORS)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entities.rs                     # SddCycle, ArtifactSeal, TraceabilityLink, AtomicTask, ExecutionEvidence
│       ├── special_cases.rs                # IssueLinkage, SpecLockState, TddLifecycleState, BoardSyncOutcome
│       ├── ports.rs                        # CycleEvidenceRepositoryPort, ArtifactHasherPort, ProjectBoardGatewayPort
│       ├── errors.rs                       # DomainViolation & exit code categorization
│       ├── gates_fsm.rs                    # Typestate Machine (Gate0Discovery..Gate6Certification)
│       ├── invariants_policy.rs            # Pure evaluators for Hard Invariants (RN-001..RN-016, command guard)
│       └── use_cases/
│           ├── mod.rs
│           ├── initialize_cycle.rs         # Gate 0 initialization & pre-existing dirty file preservation
│           ├── query_cycle_status.rs       # Status inspection use case
│           └── inspect_pre_tool_command.rs # Synchronous pre-tool hook policy evaluation
│
├── trustkit-db/                            # SQLite WAL Persistence & BLAKE3 Hashing Adapters
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── migrations.rs                   # WAL pragmas & 5-table zero-NULL schema DDL
│       ├── sqlite_adapter.rs               # SqliteWalRepositoryAdapter implementing CycleEvidenceRepositoryPort
│       ├── row_mappers.rs                  # Bidirectional mapping between SQLite rows and Special Case Enums
│       └── blake3_adapter.rs               # Blake3ParallelSealerAdapter implementing ArtifactHasherPort
│
├── trustkit-gh/                            # GitHub CLI & Projects v2 Adapters
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── identity_guard.rs               # GitHubIdentityGuard (RN-014 active account verification)
│       ├── board_synchronizer.rs           # ProjectBoardSynchronizer (9 official columns & offline fallback)
│       └── git_inspector.rs                # Local Git worktree & pre-existing dirty file discovery
│
├── trustkit-ast/                           # Workspace member scaffold for Phase 2/3 (tree-sitter & petgraph)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── trustkit-orch/                          # Workspace member scaffold for Phase 4 (worktrees & swarm)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
└── trustkit-cli/                           # CLI Entrypoint, Composition Root & Dual Presenter
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs                         # Entrypoint invoking bootstrap and CliBoundaryErrorHandler
    │   ├── bootstrap.rs                    # Composition Root (Fast Sync Bootstrap vs Full Concurrent Bootstrap)
    │   ├── cli_contract.rs                 # Clap v4 command definitions (init, status, hook pre-tool, --json)
    │   ├── error_handler.rs                # CliBoundaryErrorHandler (DomainViolation -> ExitCode 0..6)
    │   ├── presenter.rs                    # Null-free JSON serializer & Emoji-free ASCII table renderer
    │   └── commands/
    │       ├── mod.rs
    │       ├── init_cmd.rs                 # Handler for `trustkit init`
    │       ├── status_cmd.rs               # Handler for `trustkit status`
    │       └── hook_cmd.rs                 # Handler for `trustkit hook pre-tool`
    └── tests/
        ├── e2e_hook_interception.rs        # E2E tests for pre-tool command blocking (< 3ms)
        └── e2e_init_and_status.rs          # E2E tests for linked/unlinked init, dirty files & null-free JSON
```

**Structure Decision**: Multi-crate Cargo Workspace under `crates/` matching Section 6.4 of `ADR-001`. This physical crate separation enforces architectural boundaries at compile time: `trustkit-core` cannot accidentally import `rusqlite` or `clap` because those crates do not exist in its `Cargo.toml`. Every source file is scoped to a single cohesive responsibility well below the 250-line limit.

## Complexity Tracking

> **No Constitutional violations exist. All 7 Core Principles are satisfied with zero exceptions.**
