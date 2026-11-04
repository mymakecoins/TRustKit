---
description: "Architectural rules and quality standards for Rust crates in the TRustKit workspace"
paths:
  - "crates/**"
---

# Rust Crates Architecture and Quality Standards

When modifying any files under crates/:

1. Hexagonal Isolation:
   - trustkit-core must remain strictly zero-dependency (std only). No external crates are permitted in its Cargo.toml.
   - Domain models, entities, and port traits live exclusively in trustkit-core.
   - Implementations and adapters belong in their respective crates (trustkit-db, trustkit-gh, trustkit-ast, trustkit-orch).

2. Code Quality and Testing:
   - All public functions and structs must be documented with rustdoc.
   - All new features require accompanying unit and integration tests under the RED -> GREEN TDD cycle.
   - Code must pass `cargo clippy --workspace --all-targets -- -D warnings` with zero warnings.
   - Code formatting must strictly adhere to standard rustfmt (`cargo fmt --check`).

3. Performance and Memory:
   - Prefer zero-cost abstractions and borrow semantics over unnecessary clones or heap allocations.
   - Release profile utilizes LTO (thin), strip, and opt-level 3.
