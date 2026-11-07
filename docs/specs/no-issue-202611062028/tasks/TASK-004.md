# TASK-004: Persistencia Transacional de Milestones no SQLite WAL

> **Camada:** DB
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-002
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/migrations.rs
- crates/trustkit-db/src/milestone_ops.rs
- crates/trustkit-db/src/sqlite_adapter.rs
- crates/trustkit-db/src/lib.rs
test_file: crates/trustkit-db/tests/sqlite_milestone_repository_tests.rs

---

## Contexto
Os marcos planejados precisam ser armazenados de maneira persistente e transacional no banco `.trustkit/state.db` (SQLite WAL) com garantia estrita de ausencia de NULLs (100% NOT NULL) e suporte a consultas por status, dependencias e ciclos SDD vinculados.

## Escopo
- Criar migracao para tabela `milestones` em `crates/trustkit-db/src/migrations.rs` com colunas: `milestone_id`, `plan_id`, `sequence_index`, `title`, `slug`, `file_path`, `goal`, `dependencies_json`, `capabilities_json`, `status`, `created_at`, `updated_at`.
- Implementar `milestone_ops.rs` com funcoes transacionais `save_milestone`, `find_milestone_by_id`, `list_milestones`, `update_milestone_status`.
- Implementar `MilestoneRepositoryPort` no `SqliteWalRepositoryAdapter`.
- Escrever testes em `crates/trustkit-db/tests/sqlite_milestone_repository_tests.rs`.
