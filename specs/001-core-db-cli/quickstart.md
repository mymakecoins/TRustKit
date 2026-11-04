# Quickstart & Validation Guide — `001-core-db-cli`

**Feature**: `001-core-db-cli`
**Date**: 2026-10-27

---

## 1. Prerequisites

Ensure the following tools are installed on the host Linux environment:
- **Rust Toolchain**: `rustc >= 1.82` (Edition 2024 support), `cargo`, `clippy`, `rustfmt`
- **Git CLI**: `git >= 2.38`
- **GitHub CLI**: `gh >= 2.40` (authenticated via `gh auth status`)
- **Benchmarking (Optional for Gate 5/6)**: `hyperfine`

---

## 2. Build & Architectural Purity Verification

Run these commands from the repository root to compile the workspace and verify the Constitutional Hexagonal Isolation rule:

```bash
# 1. Verify strict linting with zero warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 2. Prove that trustkit-core has zero external dependencies (only std)
cargo tree -p trustkit-core

# 3. Build optimized static release binary
cargo build --workspace --release
```

**Expected Outcome:**
- `cargo clippy` exits with `0` and zero warnings.
- `cargo tree -p trustkit-core` outputs a single line (`trustkit-core v0.1.0 (...)`) with no child dependency nodes.
- Binary is generated at `./target/release/trustkit`.

---

## 3. Running Unit, Integration & E2E Tests

```bash
# 1. Run fast in-memory domain unit tests (< 50ms)
cargo test -p trustkit-core

# 2. Run SQLite WAL & BLAKE3 adapter integration tests
cargo test -p trustkit-db

# 3. Run full workspace unit + binary + E2E test suites
cargo test --workspace
```

---

## 4. End-to-End Validation Scenarios

### Scenario A: Pre-Tool Hook Interception (`< 3ms`)
Verify that forbidden commands are intercepted and safe commands pass:

```bash
# Should PASS with Exit Code 0
./target/release/trustkit hook pre-tool --command "cargo test --workspace" --json

# Should BLOCK with Exit Code 4 (RN-006 Violation)
./target/release/trustkit hook pre-tool --command "git stash" --json
./target/release/trustkit hook pre-tool --command "git add ." --json
```

### Scenario B: Cycle Initialization & Null-Free Status Inspection
Verify cycle creation in an isolated temporary repository or local unlinked cycle:

```bash
# Initialize an unlinked local cycle in JSON mode
./target/release/trustkit init --json

# Query status in JSON mode and verify zero occurrences of ": null"
./target/release/trustkit status --json | grep -v ": null"

# Query status in Human TTY mode and verify ASCII table formatting
./target/release/trustkit status
```

### Scenario C: Cold-Start Performance Benchmark (`p99 < 3ms`)
```bash
hyperfine --warmup 10 './target/release/trustkit status --json'
hyperfine --warmup 10 './target/release/trustkit hook pre-tool --command "git status" --json'
```
