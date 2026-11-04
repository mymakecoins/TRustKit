---
description: "Governance rules for Spec-Driven Development specifications and task artifacts"
paths:
  - "specs/**"
---

# SDD Specifications and Governance Invariants

When reading or modifying files under specs/:

1. Spec Kit Artifact Hierarchy:
   - Every feature must maintain spec.md, plan.md, and tasks.md under its respective spec directory.
   - Do not manually fabricate completion marks. Status transitions are verified mechanically via `trustkit gate` commands.

2. Deterministic Gates:
   - Gate 1: Specification completeness and consistency check.
   - Gate 2: Implementation planning and architectural validation.
   - Gate 3: Task atomization, dependency topological ordering, and GitHub board sync.
   - Gate 4: Test-Driven Development execution (RED -> GREEN verification).
   - Gate 5: Gauntlet Loop V2 multi-agent consensus validation.
   - Gate 6: Cryptographic sealing (BLAKE3 hash generation) and completion freeze.

3. Immutability:
   - Once a task or spec is sealed, its hash is recorded in .trustkit/state.db. Any manual drift will trigger an invariant violation.
