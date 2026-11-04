# Domain Ports & Trait Contracts (`trustkit-core::ports`)

Contratos abstratos de portas para preservacao do Principio I (Isolamento Hexagonal Estrito - sem dependencias externas em `trustkit-core`).

---

## 1. `PrdSpecificationIngestorPort`
```rust
pub trait PrdSpecificationIngestorPort: Send + Sync {
    fn ingest_from_prd(
        &self,
        prd_path: &str,
        custom_slug: Option<&str>,
    ) -> Result<SpecificationV2, DomainViolation>;
}
```

## 2. `SpecValidatorPort`
```rust
pub trait SpecValidatorPort: Send + Sync {
    fn validate_specification(
        &self,
        spec_content: &str,
    ) -> Result<SpecValidationReport, DomainViolation>;
}

pub struct SpecValidationReport {
    pub mermaid_valid: bool,
    pub gherkin_valid: bool,
    pub orphan_rules: Vec<String>,
    pub total_rules: usize,
    pub total_requirements: usize,
}
```

## 3. `WaveExecutionPlannerPort`
```rust
pub trait WaveExecutionPlannerPort: Send + Sync {
    fn plan_waves(
        &self,
        tasks: &[TaskNode],
    ) -> Result<WaveSchedule, DomainViolation>;
}
```

## 4. `CycleEvidenceRepositoryPort` (Extensoes)
```rust
// Metodos adicionais a serem incluidos na trait existente em trustkit_core::ports:
pub trait CycleEvidenceRepositoryPort: Send + Sync {
    // Existentes:
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation>;
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation>;
    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation>;
    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation>;
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation>;
    fn list_evidences_for_cycle(&self, cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation>;

    // Novos metodos Fase 2:
    fn update_cycle_spec_lock(
        &self,
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation>;

    fn save_atomic_tasks(
        &self,
        cycle_id: &str,
        tasks: &[TaskNode],
        wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation>;
}
```
