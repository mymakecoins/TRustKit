# SDD Gate Verification Workflow

Run deterministic Spec-Driven Development gate checks across the repository using the native TRustKit engine.

## Usage

Execute the validation gate:
```bash
cargo run --bin trustkit -- gate check --feature $FEATURE_NAME
```

Ensure all invariants pass:
1. Architectural consistency across spec.md, plan.md, and tasks.md.
2. TDD RED/GREEN phase evidence recorded.
3. BLAKE3 integrity check on sealed artifacts.
