# Implementation Plan: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

**Branch**: `002-spec-v2-dag-planner` | **Date**: 2026-10-27 | **Spec**: [specs/002-spec-v2-dag-planner/spec.md](spec.md)

**Input**: Feature specification from `specs/002-spec-v2-dag-planner/spec.md`

## Summary

Implementar os componentes e fluxos correspondentes a Fase 2 do TRustKit (Gates 1, 2 e 3):
1. **Ingestor Upstream (`PrdSpecificationIngestor`)**: Parse nativo em Markdown de documentos PRD/ADR para extracao de regras de negocio e requisitos funcionais, gerando o esqueleto pre-populado da Spec V2 (`trustkit spec init --from-prd`).
2. **Validacao Estrutural e Selo Criptografico (`SpecLockValidator` & `ArtifactSeal`)**: Inspecao de diagramas Mermaid (`flowchart`), blocos Gherkin e fechamento matematico da Matriz de Rastreabilidade, com computacao de digest `BLAKE3` gravado em SQLite (`trustkit spec lock`).
3. **Planejador de Ondas via Grafo Aciclico Direcionado (`WaveExecutionPlanner`)**: Construcao de DAG atraves de `petgraph`, deteccao de ciclos e agrupamento em ondas topologicas com isolamento deterministico de tarefas que tocam arquivos comuns (`trustkit wave plan`).

---

## Technical Context

**Language/Version**: Rust 2024 Edition (toolchain 1.82+)

**Primary Dependencies**:
- `trustkit-core`: Nenhuma dependencia externa (`std` exclusivo)
- `trustkit-ast`: `petgraph = "0.7"` (DAG e ordenacao topologica)
- `trustkit-db`: `rusqlite = "0.32"`, `blake3 = "1.5"`, `rayon = "1.10"`
- `trustkit-cli`: `clap = "4.5"`, `serde = "1.0"`, `serde_json = "1.0"`, `tokio = "1.40"`

**Storage**: SQLite 3 embarcado em modo WAL (`.trustkit/state.db`) com esquema relacional 100% `NOT NULL`

**Testing**: `cargo test --workspace` (unitarios com fakes em memoria em `< 1ms`, integracao de adapters e testes E2E com `assert_cmd` e `tempfile`)

**Target Platform**: Linux x86_64 e aarch64 (ambiente nativo e pipelines de CI/CD)

**Project Type**: CLI nativo de governanca de ciclo de vida de software e harness SDD

**Performance Goals**:
- Ingestao upstream de PRD de 500 linhas: `< 10ms` (meta spec: `< 100ms`)
- Validacao e congelamento criptografico (`spec lock`): `< 5ms` (meta spec: `< 15ms`)
- Construcao e ordenacao topologica do DAG de 100 tarefas: `< 3ms` (meta spec: `< 10ms`)

**Constraints**:
- Zero utilizacao de `unwrap()`, `expect()` ou `panic!()` em codigo de producao
- Zero representacao de `NULL` em banco de dados e em cargas JSON (`: null` estritamente proibido)
- Zero dependencias externas em `trustkit-core`
- Zero caracteres emoji ou rodapes gerados por IA em arquivos de codigo, mensagens de commit ou artefatos

**Scale/Scope**: Fase 2 do PRD_TRustKit.md (Gates 1, 2 e 3)

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio Constitucional | Status Pre-Design | Status Pos-Design | Justificativa / Evidencia |
| :--- | :---: | :---: | :--- |
| **I. Strict Hexagonal Isolation** | PASS | PASS | `trustkit-core` mantem `std` exclusivo. Novas entidades (`SpecificationV2`, `WaveSchedule`) e portas abstratas (`PrdSpecificationIngestorPort`, `SpecValidatorPort`, `WaveExecutionPlannerPort`) ficam estritamente em `trustkit-core::ports`. |
| **II. Absolute Prohibition of NULL** | PASS | PASS | Estados modelados como Enums (`SpecLockState`, `TraceabilityGap`). Campos no SQLite usam Special Cases (`'DRAFT_UNVERIFIED'`, `'FROZEN'`). Saidas JSON emitem colecoes vazias `[]` e estruturas tipadas. |
| **III. Test-First & Anti-Cheat** | PASS | PASS | TDD obrigatorio em todas as camadas (`RED -> GREEN`). Testes unitarios do dominio isolados em memoria com fakes. |
| **IV. Resilient Error Handling** | PASS | PASS | Todos os erros propagados atraves do enum `DomainViolation` e mapeados para exit codes POSIX no boundary de `trustkit-cli`. Proibido `unwrap()` em producao. |
| **V. Separation of Construction & Execution** | PASS | PASS | Injecao de dependencias isolada exclusivamente no Composition Root (`crates/trustkit-cli/src/bootstrap.rs`). |
| **VI. Ubiquitous Language & Clean Code** | PASS | PASS | Vocabulario alinhado ao PRD (`ArtifactSeal`, `WaveSchedule`, `SpecLockState`). Arquivos modulares mantidos abaixo de 250 LOC. |
| **VII. Cryptographic Immutability** | PASS | PASS | Selamento da Spec com digest `BLAKE3` gravado na tabela `artifact_seals`. |

---

## Project Structure

### Documentation (this feature)

```text
specs/002-spec-v2-dag-planner/
├── plan.md              # Este arquivo de planejamento de implementacao
├── research.md          # Pesquisa tecnica e decisoes arquiteturais (Fase 0)
├── data-model.md        # Entidades puras, typestate e esquema relacional (Fase 1)
├── quickstart.md        # Guia executavel de homologacao ponta a ponta (Fase 1)
├── contracts/           # Contratos formais de interfaces e portas de dominio
│   ├── cli-spec-init.md # Contrato do comando 'trustkit spec init --from-prd'
│   ├── cli-spec-lock.md # Contrato do comando 'trustkit spec lock'
│   ├── cli-wave-plan.md # Contrato do comando 'trustkit wave plan'
│   └── domain-ports.md  # Contratos das traits em trustkit-core::ports
└── checklists/
    └── requirements.md  # Checklist de conformidade e qualidade da especificacao
```

### Source Code Layout

```text
crates/
├── trustkit-core/
│   └── src/
│       ├── entities.rs            # UpstreamDocument, SpecificationV2, TaskNode, WaveSchedule
│       ├── ports.rs               # PrdSpecificationIngestorPort, SpecValidatorPort, WaveExecutionPlannerPort
│       ├── errors.rs              # DomainViolation: MissingBusinessRules, TraceabilityGap, CycleDetected
│       └── gates_fsm.rs           # Avanco typestate: Gate 1 -> Gate 2 -> Gate 3
│
├── trustkit-ast/
│   ├── Cargo.toml                 # Adicao de petgraph = "0.7"
│   └── src/
│       ├── lib.rs
│       ├── prd_ingestor.rs        # Parser Markdown upstream (--from-prd)
│       ├── spec_validator.rs      # Validador de Mermaid, Gherkin e Matriz de Rastreabilidade
│       └── wave_planner.rs        # Construcao do DAG (petgraph), deteccao de ciclos e ondas
│
├── trustkit-db/
│   └── src/
│       ├── sqlite_adapter.rs      # Atualizacao de lock da spec e persistencia de tarefas planejadas
│       └── row_mappers.rs         # Mapeadores de artifact_seals e atomic_tasks
│
└── trustkit-cli/
    └── src/
        ├── cli_contract.rs        # Subcomandos 'spec init', 'spec lock' e 'wave plan'
        ├── bootstrap.rs           # Injecao de dependencias do Ingestor, Validador e WavePlanner
        └── commands/
            ├── spec_init_cmd.rs   # Handler CLI para ingestao upstream
            ├── spec_lock_cmd.rs   # Handler CLI para congelamento criptografico
            └── wave_plan_cmd.rs   # Handler CLI para planejamento de ondas via DAG
```

**Structure Decision**: A distribuicao modular reflete a arquitetura do Cargo Workspace já estabelecida na Fase 1. O isolamento de algoritmos de grafo (`petgraph`) e parsers sintaticos dentro de `trustkit-ast` preserva o purismo de `trustkit-core` e evita dispersao de responsabilidades.

---

## Complexity Tracking

> **Zero Violacoes Constitucionais** - Nao ha complexidade acidental ou desvios arquiteturais necessitando justificativa.
