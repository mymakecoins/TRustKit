<!--
Sync Impact Report
- Version change: Unversioned Scaffold -> 1.0.0
- Modified principles:
  * [PRINCIPLE_1_NAME] -> I. Strict Hexagonal Isolation (Domain Purity - Clean Code Ch. 8)
  * [PRINCIPLE_2_NAME] -> II. Absolute Prohibition of NULL & Special Case Objects (Clean Code Ch. 7)
  * [PRINCIPLE_3_NAME] -> III. Test-First, Anti-Cheat & Deterministic Equivalence (NON-NEGOTIABLE)
  * [PRINCIPLE_4_NAME] -> IV. Resilient Error Handling & Zero-Panic Policy (Clean Code Ch. 7)
  * [PRINCIPLE_5_NAME] -> V. Separation of Construction and Execution (Clean Code Ch. 11)
- Added principles:
  * VI. Ubiquitous Language & Focused Modular Clean Code (Clean Code Ch. 2 & 12)
  * VII. Cryptographic Immutability & Event Sourcing ("The Event Log Proves")
- Added sections:
  * Architectural Constraints, Technology Stack & Performance Standards (Section 2)
  * Development Workflow, Quality Gates & Gauntlet Compliance (Section 3)
- Removed sections: None (all template scaffold sections formalized)
- Follow-up TODOs: None (all placeholders resolved deterministically)
-->

# TRustKit Constitution

## Core Principles

### I. Strict Hexagonal Isolation (Domain Purity - Clean Code Ch. 8)
The `trustkit-core` crate MUST NOT contain any external third-party dependencies in its `Cargo.toml`; it depends solely on the Rust standard library (`std`). All external libraries (including `rusqlite`, `tree-sitter`, `petgraph`, `blake3`, `gix`, `git2`, `tokio`, `clap`, and `serde_json`) are strictly quarantined to boundary infrastructure crates (`trustkit-db`, `trustkit-ast`, `trustkit-orch`, `trustkit-gh`, and `trustkit-cli`). The domain MUST access external capabilities exclusively through abstract Traits (Ports) defined in `trustkit_core::ports`. All domain unit tests MUST execute entirely in-memory with zero disk, network, or database I/O in under 1ms using in-memory fake adapters.

### II. Absolute Prohibition of NULL & Special Case Objects (Clean Code Ch. 7)
Database schemas and domain representations MUST NEVER utilize nullable representations where domain intent can be captured via algebraic types. In the SQLite persistence layer (`trustkit-db`), 100% of state columns MUST be marked `NOT NULL` and default to explicit Special Case values (such as `'UNLINKED_LOCAL_CYCLE'`, `'DRAFT_UNVERIFIED'`, `'NOT_PROVISIONED'`, `'[]'`). Across all CLI outputs in `--json` mode and MCP tool payloads, `: null` is strictly forbidden; missing or unlinked relationships MUST be emitted as typed Special Case objects or empty arrays `[]` to minimize agent token consumption and eliminate null-dereference defects.

### III. Test-First, Anti-Cheat & Deterministic Equivalence (NON-NEGOTIABLE)
Strict Test-Driven Development (`RED -> GREEN -> REFACTOR`) is mandatory across all crates. Tests MUST be authored and proven to fail before production implementation begins. Dilution or weakening of existing assertions to force passing builds is strictly prohibited. Because TRustKit is a native CLI binary without browser UI, Gauntlet verification MUST satisfy the following deterministic test equivalences:
- **Lint & Static Analysis:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` with exit code 0 (zero compiler or linter warnings permitted).
- **Workspace Build:** `cargo build --workspace --release` with exit code 0, alongside automated `cargo tree -p trustkit-core` verification proving zero external domain dependencies.
- **Unit & Integration Suite:** `cargo test --workspace --lib --bins` verifying domain state transitions and port contracts.
- **End-to-End Suite:** `cargo test --test 'e2e_*'` executing the compiled `trustkit` binary against isolated ephemeral Git repositories via `assert_cmd`, `predicates`, and `tempfile`.

### IV. Resilient Error Handling & Zero-Panic Policy (Clean Code Ch. 7)
The use of `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` is strictly forbidden in production code across all crates (only permitted within `#[cfg(test)]` blocks). External adapter failures MUST be intercepted at the infrastructure boundary and mapped into explicit domain error variants (`trustkit_core::errors::DomainViolation`). Only `trustkit-cli::error_handler::CliBoundaryErrorHandler` is authorized to translate domain errors into POSIX-compliant exit codes (`0` through `6`) and emit structured error payloads to `stderr`.

### V. Separation of Construction and Execution (Clean Code Ch. 11)
Domain entities and use case services MUST NOT instantiate concrete adapters, read environment variables, or rely on mutable global singletons (`lazy_static!` or infrastructure `OnceLock`). Object graph assembly and dependency injection are strictly isolated within the Composition Root (`trustkit-cli::bootstrap`). High-frequency synchronous execution paths (such as `trustkit hook pre-tool`) MUST execute without initializing the `tokio` async runtime, guaranteeing a sub-3ms cold-start latency (`p99 < 3ms`).

### VI. Ubiquitous Language & Focused Modular Clean Code (Clean Code Ch. 2 & 12)
Code identifiers MUST reflect the ubiquitous language of Spec-Driven Development and the Gauntlet workflow (`SddCycle`, `ArtifactSeal`, `TraceabilityLink`, `ExecutionEvidence`, `GateFsm`, `WavePlanner`). Generic names and anti-pattern suffixes (`Manager`, `Processor`, `Data`, `Info`, `Helper`, `Util`) are strictly prohibited. Every Rust source file MUST have a single cohesive responsibility and MUST NOT exceed 250 lines of code. Functions MUST do one thing and do it well.

### VII. Cryptographic Immutability & Event Sourcing ("The Event Log Proves")
All specification documents and test suite baselines MUST be cryptographically sealed with BLAKE3 hashes at Gate 2. Audit reports (`gauntlet-report.md`) and issue closure comments MUST NOT be synthesized creatively by an LLM; they MUST be rendered deterministically from the immutable SQLite WAL event store (`.trustkit/state.db`). All source code, commit messages, pull requests, and documentation MUST remain completely free of emojis, Unicode icons, and artificial AI attribution trailers (`Co-Authored-By: Claude...`).

## Architectural Constraints, Technology Stack & Performance Standards

### 1. Technology Stack
- **Language & Toolchain:** Rust (`1.82+`, Edition 2024), standard library conventions, formatted with `rustfmt`.
- **Workspace Architecture:** Cargo multi-crate workspace comprising:
  * `trustkit-core`: Pure domain logic, PORS entities, Typestate FSM, Ports, and zero dependencies.
  * `trustkit-db`: SQLite repository adapter (`rusqlite` bundled) and cryptographic sealing (`blake3`, `rayon`).
  * `trustkit-ast`: Syntactic inspection via `tree-sitter` and DAG wave topology via `petgraph`.
  * `trustkit-orch`: Concurrency orchestrator, Git worktree provisioning, and process supervisor.
  * `trustkit-gh`: GitHub CLI GraphQL adapter and Project Board synchronizer.
  * `trustkit-cli`: CLI parser (`clap`), Composition Root (`bootstrap`), and POSIX error boundary.
- **Persistence Engine:** Embedded SQLite 3 in WAL mode (`PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`, `PRAGMA busy_timeout = 5000;`).
- **Isolation Mechanism:** Ephemeral Git Worktrees with Copy-on-Write (`reflink` / `ioctl_ficlone` where supported) for parallel wave execution.

### 2. Performance & Boundary Budgets
- **Hook Latency:** `trustkit hook pre-tool` and `trustkit hook post-tool` MUST complete in less than 3ms (`p99 < 3.0ms`).
- **AST Verification Latency:** Complete syntax tree audit of modified files MUST complete in less than 5ms.
- **Memory Footprint:** Resident Set Size (RSS) during synchronous inspection MUST remain below 15MB.
- **Binary Footprint:** Release binary compiled with `lto = "thin"` and `strip = true` targeting minimal distribution size.

## Development Workflow, Quality Gates & Gauntlet Compliance

### 1. The 7-Gate Lifecycle
Every feature, bugfix, or refactoring MUST navigate the 7 Gauntlet Gates:
- **Gate 0 (Discovery):** Workspace and tooling sanity check, developer identity verification.
- **Gate 1 (Context Slicing):** Ingestion of requirements and AST-level context pruning.
- **Gate 2 (Spec & Test Sealing):** Authoring of specification, Gherkin scenarios, and cryptographic BLAKE3 sealing.
- **Gate 3 (Atomization & DAG Wave Planning):** Decomposition into atomic tasks and topological wave grouping via `petgraph`.
- **Gate 4 (Parallel Wave Implementation):** TDD execution (`RED -> GREEN -> REFACTOR`) across isolated Git worktrees.
- **Gate 5 (Adversarial Gauntlet):** Automated anti-cheat verification, lexical invariant scan, and test integrity check.
- **Gate 6 (Certification & Board Transition):** Deterministic audit report generation, issue closure comment, and pull request dispatch.

### 2. Pull Request & Branch Standards
- Branch naming MUST adhere to:
  * Feature branches: `feat/<issue-id>-<slug>`
  * Bugfix branches: `fix/<issue-id>-<slug>`
  * Documentation/Refactoring branches: `chore/<issue-id>-<slug>`
- Conventional Commits format is strictly enforced: `type(scope): imperative description`. Commits MUST NOT include AI co-authorship tags or Unicode emojis.
- Every Pull Request MUST link to an active issue, present the deterministic `gauntlet-report.md`, and confirm clean passes across all test equivalence gates.

## Governance

This Constitution represents the supreme architectural and operational contract of the TRustKit repository. It supersedes informal conventions, oral agreements, and prompt instructions.

### 1. Amendment Procedure
- Any modification, addition, or revocation of principles within this Constitution requires an explicit proposal documented in an Architecture Decision Record (ADR) or specification document.
- Amendments MUST be validated through a formal Spec Kit amendment cycle (`/speckit-constitution`).
- Every amendment MUST increment the document version in accordance with the Semantic Versioning policy below and document rationale in the temporary Sync Impact Report.

### 2. Versioning Policy
- **MAJOR (X.0.0):** Incompatible structural changes, removals, or fundamental redefinitions of core principles or governance policies.
- **MINOR (1.X.0):** Addition of new principles, expansion of architectural constraints, or introduction of new compliance gates.
- **PATCH (1.0.X):** Clarifications, wording corrections, formatting adjustments, or non-semantic refinements.

### 3. Compliance & Enforcement
- Automated verification via `/speckit-analyze` and `trustkit doctor` MUST validate project artifacts against the rules established herein.
- Violations detected during code review or Gauntlet evaluation MUST be treated as blocking defects requiring resolution prior to merge.

**Version**: 1.0.0 | **Ratified**: 2026-10-27 | **Last Amended**: 2026-10-27
