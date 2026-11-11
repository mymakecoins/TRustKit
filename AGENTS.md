# AGENTS.md - TRustKit Agent Guidance and Governance

## 1. Project Overview

TRustKit is a deterministic engine written in Rust for Spec-Driven Development (SDD) governance, Git worktrees isolation, Gauntlet Loop V2 multi-agent consensus, and structural AST code intelligence.

The architecture follows a Hybrid Rust Core + Thin Skill model:
- Thin Skill Layer: Lightweight Markdown guidance (.agents/skills, .claude/rules) residing in agent context to guide high-level decisions without consuming excessive context tokens.
- Native Rust Core: The static binary (trustkit) executed for all state mutations, BLAKE3 cryptographic sealing, Tree-Sitter AST extractions, Git worktree provisioning, and GitHub Projects synchronization.

## 2. Workspace Architecture

The repository is structured as a modular Cargo workspace following strict hexagonal isolation:

- crates/trustkit-core: Pure domain models (std only), lifecycle state machines, contextual next-step resolver (NextStepResolver), structural graphs, abstract ports, and zero external dependencies.
- crates/trustkit-db: SQLite WAL persistence (.trustkit/state.db), FTS5 BM25 search index, relational graphs with Shrink-Guard, and parallel BLAKE3 hashing.
- crates/trustkit-gh: GitHub CLI and GraphQL v2 integration for project boards, issues, on-demand pull requests, and commit identity guarding.
- crates/trustkit-ast: Tree-sitter AST extraction for Rust and TypeScript, structural coupling metrics, anti-cheat test assertion audit, and BFS/DFS context slicing.
- crates/trustkit-orch: Git worktrees isolation with Copy-on-Write (reflink), asynchronous Tokio task supervision, native MCP server, and Gauntlet critic swarms.
- crates/trustkit-cli: Unified command-line interface entry point (trustkit binary) and Dual-Audience presenters (ASCII NEXT STEP footer and null-free next_steps JSON array).

## 3. Build, Test, and Quality Commands

All agents must verify changes locally before completing tasks:

- Compile workspace: cargo build --workspace
- Run test suite: cargo test --workspace
- Check formatting: cargo fmt --check
- Run linter: cargo clippy --workspace --all-targets -- -D warnings
- Release build: cargo build --workspace --release

## 4. Constitutional Invariants and Agent Non-Negotiables

Agents interacting with or developing TRustKit must strictly follow these rules:

1. Zero Emojis: Absolute prohibition of emojis or graphic icons in source code, documentation, markdown files, terminal outputs, or git commits.
2. Zero AI Trailers: Absolute prohibition of automated AI authorship metadata in git commits (such as Co-Authored-By, Generated-by, or model tags).
3. Deterministic State Mutations: All lifecycle transitions and artifact validations must be executed via the trustkit binary, never by guessing or manual file editing.
4. Strict TDD Cycle: Test-Driven Development is mandatory for all logic. The RED phase (demonstrated failure) must be recorded before proceeding to the GREEN phase (proven success).
5. On-Demand Pull Requests: Pull requests are created only upon explicit human developer request.
6. Protected Branch Safety: Never attempt direct pushes to protected branches (main, master, staging, develop). Work must be performed in isolated feature branches or worktrees.
7. Mandatory Next-Step Guidance: At the end of every command or turn, agents must read the next_steps array from the binary JSON output (or the NEXT STEP: ASCII block) and present the recommended next step to the user.
