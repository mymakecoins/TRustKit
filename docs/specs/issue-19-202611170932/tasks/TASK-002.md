# TASK-002: Persistencia de Evidencias de Worktree no SQLite

layer: DB
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-db/tests/sqlite_worktree_evidence_tests.rs
expected_files:
  - crates/trustkit-core/src/entities.rs
  - crates/trustkit-db/src/sqlite_adapter.rs
  - crates/trustkit-db/tests/sqlite_worktree_evidence_tests.rs

## Contexto
Gravar informacoes claras e estruturadas nas evidencias de execucao (ExecutionEvidence) no SQLite WAL detalhando a criacao da branch e o checkout efetuado na worktree, atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma tarefa com worktree recem-provisionada
- **Quando** a evidencia de execucao for gravada no repositorio SQLite
- **Entao** o registro armazena os metadados explicitando a branch da tarefa criada e o checkout na worktree.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-sqlite-worktree-evidence.json
