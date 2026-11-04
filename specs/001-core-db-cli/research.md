# Phase 0: Architectural & Technical Research — `001-core-db-cli`

**Feature**: `001-core-db-cli` (Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence)
**Date**: 2026-10-27
**Source Reference**: `docs/ADR_001_TRustKit_Architecture.md`, `docs/PRD_TRustKit.md`

---

## 1. Runtime & Compilation Strategy for Sub-3ms Hooks

- **Decision**: Compile `trustkit` as a static native binary using Rust (`1.82+`, Edition 2024) with `lto = "thin"` and `strip = true`, implementing a dual-route Composition Root (`trustkit-cli::bootstrap`) that bypasses async runtime initialization for synchronous commands (`hook`, `status`).
- **Rationale**: Pre-tool hooks (`PreToolUse`) are invoked dozens of times per coding session. Skipping `tokio` runtime thread-pool allocation on the synchronous path guarantees `< 1ms` bootstrap and `< 3ms` total execution time (p99).
- **Alternatives Considered**:
  - *Python (`asyncio` + `pydantic`) or Node.js/TypeScript*: Rejected due to `300ms–2000ms` interpreter cold-start penalty per subprocess invocation.
  - *Always-on background daemon*: Rejected to honor Kent Beck's 4th rule of Simple Design (minimize moving parts, avoid orphan daemon lifecycle issues).

---

## 2. Strict Hexagonal Architecture & Zero-Dependency Core

- **Decision**: Isolate 100% of domain entities (`SddCycle`, `ArtifactSeal`, `TraceabilityLink`, `AtomicTask`, `ExecutionEvidence`), Special Case types, Typestate FSM (`Gate0Discovery..Gate6Certification`), and Port traits inside `trustkit-core` with **zero external crates** in `Cargo.toml` (only Rust `std`).
- **Rationale**: Enforces Clean Code Chapter 8 (Boundaries) and Chapter 11 (Separation of Construction and Execution). Guarantees that domain logic is immune to third-party API churn and allows the entire domain unit test suite to execute in-memory in `< 1ms` using lightweight Fake Adapters.
- **Alternatives Considered**:
  - *Direct `rusqlite` and `serde` imports inside domain structs*: Rejected because it couples domain entities to persistence/serialization macros and slows down compilation and isolated unit testing.

---

## 3. Transactional Event Store with SQLite WAL & Zero-NULL Schema

- **Decision**: Use embedded SQLite 3 (`rusqlite` with `bundled` feature) configured with `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`, and `PRAGMA busy_timeout = 5000;`. Enforce `NOT NULL` across 100% of table columns using explicit Special Case default values (`'UNLINKED_LOCAL_CYCLE'`, `'DRAFT_UNVERIFIED'`, `'NOT_PROVISIONED'`, `'[]'`).
- **Rationale**: Write-Ahead Logging (WAL) supports concurrent reads and atomic serialized writes across parallel agent processes without `SQLITE_BUSY` failures. Eliminating `NULL` at the schema level prevents null-dereference defects and aligns directly with Rust algebraic types.
- **Alternatives Considered**:
  - *Plain JSON files in `.trustkit/`*: Rejected due to race conditions under parallel wave execution and lack of ACID relational integrity.
  - *PostgreSQL / External Database*: Rejected because a developer CLI tool must operate zero-config in any local Git checkout.

---

## 4. Cryptographic Artifact Sealing

- **Decision**: Use `blake3` (assisted by `rayon` for multi-file hashing) inside `trustkit-db::Blake3ParallelSealerAdapter` implementing `ArtifactHasherPort`.
- **Rationale**: `BLAKE3` provides cryptographic tamper-resistance at multi-gigabyte-per-second throughput, enabling instant verification of specification files and test suites in `< 2ms`.
- **Alternatives Considered**:
  - *SHA-256 / MD5*: MD5 is cryptographically broken; standard SHA-256 is slower than BLAKE3 on modern SIMD architectures.

---

## 5. Dual-Audience Output Protocol (Null-Free JSON vs. Emoji-Free ASCII)

- **Decision**: Implement a dedicated `Presenter` in `trustkit-cli` supporting two deterministic rendering modes:
  1. **Agent Mode (`--json` or `TRUSTKIT_AGENT_MODE=1`)**: Emits compact, single-line or structured JSON with zero ANSI codes and zero `: null` fields (using Special Case objects and empty arrays `[]`).
  2. **Human Mode (Default TTY)**: Emits aligned ASCII tables and formal prefixes (`[PASS]`, `[FAIL]`, `[GATE-0]..[GATE-6]`) with strict prohibition of Unicode emojis.
- **Rationale**: Optimizes LLM context window efficiency while maintaining crisp terminal legibility for human engineers and complying with Hard Invariant `RN-007` (Zero Emojis).
- **Alternatives Considered**:
  - *Rich TUI (`ratatui`)*: Rejected as out of scope and detrimental to fast CLI piping and agent interaction.

---

## 6. GitHub Projects v2 & Identity Verification

- **Decision**: Encapsulate GitHub interactions behind `ProjectBoardGatewayPort` implemented by `trustkit-gh::GitHubCliGraphQlAdapter`, invoking the authenticated host `gh` CLI (`gh auth status`, `gh api graphql`). Support offline resilience via `BoardSyncOutcome::DeferredOffline`.
- **Rationale**: Reuses the developer's existing `gh` keyring authentication without managing raw PAT tokens, enforces `RN-014` (active account identity check), and avoids blocking local development if GitHub's API is unreachable.
- **Alternatives Considered**:
  - *Direct HTTP OAuth client in Rust*: Rejected to avoid duplicating credential storage already managed securely by `gh auth`.
