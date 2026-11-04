# Tasks: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

**Input**: Design documents from `specs/002-spec-v2-dag-planner/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`
**Tests**: Abordagem TDD rigorosa exigida pelo Principio III da Constituicao do TRustKit (`RED -> GREEN -> REFACTOR`).
**Organization**: Tarefas agrupadas por historias de usuario para viabilizar implementacao e homologacao independentes.

## Format: `[ID] [P?] [Story] Description with file path`

- **[P]**: Tarefa paralelizavel (arquivos distintos, sem dependencias bloqueantes)
- **[Story]**: Identificador da Historia de Usuario (`[US1]`, `[US2]`, `[US3]`)
- Caminho completo e exato do arquivo incluido em todas as tarefas

---

## Phase 1: Setup (Shared Infrastructure & Dependencies)

**Purpose**: Preparacao de dependencias e infraestrutura compartilhada no Cargo Workspace

- [x] T001 Adicionar dependencia petgraph = "0.7" em crates/trustkit-ast/Cargo.toml
- [x] T002 [P] Configurar modulos publicos e reexportacoes em crates/trustkit-ast/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Infraestrutura de dominio, portas, erros e persistencia SQLite compartilhada que bloqueia todas as historias de usuario

**CRITICAL**: Nenhuma historia de usuario pode iniciar antes da conclusao desta fase

- [x] T003 [P] Implementar entidades de dominio UpstreamDocument, SpecificationV2, TraceabilityRow, TaskNode, WaveSchedule e ExecutionWave sem campos nulos em crates/trustkit-core/src/entities.rs
- [x] T004 [P] Definir variantes de erro de dominio MissingBusinessRules, TraceabilityGap e CycleDetected no enum DomainViolation em crates/trustkit-core/src/errors.rs
- [x] T005 [P] Definir interfaces de portas abstratas PrdSpecificationIngestorPort, SpecValidatorPort e WaveExecutionPlannerPort em crates/trustkit-core/src/ports.rs
- [x] T006 [P] Implementar validacoes e transicoes typestate FSM para Gates 1, 2 e 3 em crates/trustkit-core/src/gates_fsm.rs
- [x] T007 [P] Expandir o adaptador SQLite com os metodos update_cycle_spec_lock e save_atomic_tasks em crates/trustkit-db/src/sqlite_adapter.rs
- [x] T008 [P] Implementar mapeadores de linha e serializadores 100% NOT NULL para artifact_seals e atomic_tasks em crates/trustkit-db/src/row_mappers.rs
- [x] T009 Implementar testes unitarios em memoria para as novas entidades e persistencia em crates/trustkit-core/src/special_cases.rs e crates/trustkit-db/src/sqlite_adapter_tests.rs

**Checkpoint**: Fundacao completa - implementacao das historias de usuario liberada

---

## Phase 3: User Story 1 - Ingestao Upstream de Especificacao a Partir de PRD/ADR (Priority: P1) 🎯 MVP

**Goal**: Inicializar automaticamente um esqueleto de especificacao Spec V2 a partir de tabelas de regras de negocio e requisitos em Markdown (PRD/ADR) via `trustkit spec init --from-prd`

**Independent Test**: Fornecer arquivo Markdown de PRD contendo tabelas RN e RF e verificar se a Spec V2 gerada contem Product Trio, Gherkin e Matriz de Rastreabilidade pre-populados, abortando caso o arquivo nao exista ou nao possua regras.

### Tests for User Story 1 (TDD RED)

- [x] T010 [P] [US1] Criar teste unitario RED para extracao de RN/RF e slugificacao de titulo H1 em crates/trustkit-ast/tests/prd_ingestor_tests.rs
- [x] T011 [P] [US1] Criar teste de integracao E2E RED para execucao do comando trustkit spec init --from-prd em crates/trustkit-cli/tests/e2e_spec_init.rs

### Implementation for User Story 1

- [x] T012 [P] [US1] Implementar o adaptador de parser Markdown PrdSpecificationIngestor compativel com PrdSpecificationIngestorPort em crates/trustkit-ast/src/prd_ingestor.rs
- [x] T013 [US1] Implementar caso de uso InitializeSpecFromPrdUseCase em crates/trustkit-core/src/use_cases/initialize_spec_from_prd.rs
- [x] T014 [P] [US1] Adicionar definicao do subcomando CLI spec init com flags --from-prd, --name, --output-dir e --json em crates/trustkit-cli/src/cli_contract.rs
- [x] T015 [US1] Implementar o handler spec_init_cmd e formatacao de saida JSON sem campos null em crates/trustkit-cli/src/commands/spec_init_cmd.rs
- [x] T016 [US1] Integrar InitializeSpecFromPrdUseCase e PrdSpecificationIngestor no Composition Root em crates/trustkit-cli/src/bootstrap.rs

**Checkpoint**: User Story 1 (MVP) funcional e testavel de forma totalmente independente

---

## Phase 4: User Story 2 - Validacao Estrutural e Congelamento Criptografico da Spec (Priority: P2)

**Goal**: Validar sintaxe Mermaid, formato Gherkin e fechamento matematico da Matriz de Rastreabilidade, calculando o digest BLAKE3 da spec e congelando o ciclo em SQLite via `trustkit spec lock`

**Independent Test**: Submeter uma spec com regra orfa e verificar rejeicao com TraceabilityGap; submeter uma spec fechada e verificar geracao de hash BLAKE3 e transicao para status FROZEN.

### Tests for User Story 2 (TDD RED)

- [x] T017 [P] [US2] Criar teste unitario RED para validacao de sintaxe Mermaid, blocos Gherkin e fechamento da Matriz em crates/trustkit-ast/tests/spec_validator_tests.rs
- [x] T018 [P] [US2] Criar teste de integracao E2E RED para comando trustkit spec lock com cenarios de sucesso e rejeicao em crates/trustkit-cli/tests/e2e_spec_lock.rs

### Implementation for User Story 2

- [x] T019 [P] [US2] Implementar o validador sintatico SpecValidator compativel com SpecValidatorPort em crates/trustkit-ast/src/spec_validator.rs
- [x] T020 [US2] Implementar caso de uso LockSpecificationUseCase gravando digest BLAKE3 na tabela artifact_seals em crates/trustkit-core/src/use_cases/lock_specification.rs
- [x] T021 [P] [US2] Adicionar definicao do subcomando CLI spec lock com flags --spec e --json em crates/trustkit-cli/src/cli_contract.rs
- [x] T022 [US2] Implementar o handler spec_lock_cmd e mapeamento de rejeicao TraceabilityGap com exit code 3 em crates/trustkit-cli/src/commands/spec_lock_cmd.rs
- [x] T023 [US2] Integrar LockSpecificationUseCase e SpecValidator no Composition Root em crates/trustkit-cli/src/bootstrap.rs

**Checkpoint**: User Stories 1 e 2 operacionais e auditaveis criptograficamente

---

## Phase 5: User Story 3 - Planejamento Topologico de Ondas de Execucao via Grafo (Priority: P3)

**Goal**: Analisar tarefas atomicas, construir Grafo Aciclico Direcionado (DAG) com petgraph, detectar ciclos e agrupar tarefas em ondas de execucao concorrente com isolamento deterministico por menor indice TASK-NNN para arquivos compartilhados via `trustkit wave plan`

**Independent Test**: Executar planejamento com tarefas cíclicas e verificar erro CycleDetected com a rota exata; executar planejamento com tarefas sem ciclo que compartilham arquivos e verificar alocacao em ondas sequenciais distintas.

### Tests for User Story 3 (TDD RED)

- [x] T024 [P] [US3] Criar teste unitario RED para deteccao de ciclos e ordenacao deterministica de ondas concorrentes em crates/trustkit-ast/tests/wave_planner_tests.rs
- [x] T025 [P] [US3] Criar teste de integracao E2E RED para comando trustkit wave plan em crates/trustkit-cli/tests/e2e_wave_plan.rs

### Implementation for User Story 3

- [x] T026 [P] [US3] Implementar WaveExecutionPlanner usando petgraph::graph::DiGraph com resolucao de conflitos por TASK-NNN em crates/trustkit-ast/src/wave_planner.rs
- [x] T027 [US3] Implementar caso de uso PlanExecutionWavesUseCase exigindo estado FROZEN da spec em crates/trustkit-core/src/use_cases/plan_execution_waves.rs
- [x] T028 [P] [US3] Adicionar definicao do subcomando CLI wave plan com flags --tasks-dir e --json em crates/trustkit-cli/src/cli_contract.rs
- [x] T029 [US3] Implementar o handler wave_plan_cmd com mapeamento de CycleDetected para exit code 4 em crates/trustkit-cli/src/commands/wave_plan_cmd.rs
- [x] T030 [US3] Integrar PlanExecutionWavesUseCase e WaveExecutionPlanner no Composition Root em crates/trustkit-cli/src/bootstrap.rs

**Checkpoint**: As tres historias de usuario da Fase 2 integradas e funcionais

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verificacao da Constituicao, linters estritos, analise de arvore de dependencias e homologacao ponta a ponta

- [x] T031 [P] Executar analise estatica estrita sem advertencias em todo o workspace via cargo clippy em crates/trustkit-cli/src/main.rs
- [x] T032 [P] Validar pureza do crate de dominio comprovando zero dependencias externas via cargo tree -p trustkit-core em crates/trustkit-core/Cargo.toml
- [x] T033 Executar a suite completa de testes unitarios e E2E via cargo test --workspace
- [x] T034 Executar os quatro cenarios de homologacao ponta a ponta documentados em specs/002-spec-v2-dag-planner/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Sem dependencias - inicio imediato
- **Foundational (Phase 2)**: Depende do Setup (Phase 1) - BLOQUEIA todas as historias de usuario
- **User Stories (Phase 3, 4, 5)**: Dependem da conclusao da Fase Foundational
  - US1 (Ingestao Upstream): Pode iniciar imediatamente apos Fase 2
  - US2 (Validacao e Congelamento): Pode iniciar apos Fase 2 (consome a spec gerada ou manual)
  - US3 (Planejamento de Ondas): Depende da conclusao da validacao de US2 (spec congelada) para avanco ao Gate 3
- **Polish (Phase 6)**: Depende da conclusao das tres historias de usuario

### Within Each User Story

- Testes TDD (RED) devem ser codificados e comprovadamente falhar antes da implementacao
- Entidades de dominio e portas precedem os adaptadores de infraestrutura
- Adaptadores de AST/DB precedem os comandos CLI
- Registro no Composition Root (`bootstrap.rs`) fecha o ciclo da historia

### Parallel Opportunities

- Na Fase 1: T002 pode rodar em paralelo apos T001
- Na Fase 2: T003, T004, T005, T006, T007 e T008 operam em arquivos distintos e podem ser implementados concorrentemente
- Em cada User Story: Os testes unitarios de AST e os testes E2E do CLI podem ser escritos em paralelo
- Em US1: T012 e T014 podem ser desenvolvidos em paralelo
- Em US2: T019 e T021 podem ser desenvolvidos em paralelo
- Em US3: T026 e T028 podem ser desenvolvidos em paralelo
- Na Fase 6: T031 e T032 rodam em paralelo

---

## Parallel Example: User Story 1

```bash
# Red-phase tests em paralelo:
Task: "T010 [P] [US1] Criar teste unitario RED para extracao de RN/RF e slugificacao de titulo H1 em crates/trustkit-ast/tests/prd_ingestor_tests.rs"
Task: "T011 [P] [US1] Criar teste de integracao E2E RED para execucao do comando trustkit spec init --from-prd em crates/trustkit-cli/tests/e2e_spec_init.rs"

# Implementacao de parser e CLI contract em paralelo:
Task: "T012 [P] [US1] Implementar o adaptador de parser Markdown PrdSpecificationIngestor compativel com PrdSpecificationIngestorPort em crates/trustkit-ast/src/prd_ingestor.rs"
Task: "T014 [P] [US1] Adicionar definicao do subcomando CLI spec init com flags --from-prd, --name, --output-dir e --json em crates/trustkit-cli/src/cli_contract.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Concluir Phase 1 (Setup: adicionar `petgraph`)
2. Concluir Phase 2 (Foundational: portas, entidades e metodos DB)
3. Concluir Phase 3 (User Story 1: `trustkit spec init --from-prd`)
4. **Homologar MVP**: Testar a ingestao de um documento PRD real gerando `<slug>-spec.md` com regras de negocio pre-populadas

### Incremental Delivery

1. Setup + Foundational concluidos -> Base solida
2. Adicionar US1 -> Ingestao upstream validada (MVP entregue)
3. Adicionar US2 -> Congelamento criptografico `BLAKE3` e fechamento da Matriz validado
4. Adicionar US3 -> Planejamento de ondas via DAG e resolucao deterministica de arquivos compartilhados validado
5. Polish -> Clippy com zero warnings, `cargo tree` auditado e quickstart executado com sucesso
