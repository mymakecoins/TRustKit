# TASK-002: Persistencia SQLite WAL para Listagem e Atualizacao de Ciclos Ativos

layer: DB
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-db/tests/sqlite_active_cycles_tests.rs
expected_files:
  - crates/trustkit-db/src/sqlite_active_cycles.rs
  - crates/trustkit-db/tests/sqlite_active_cycles_tests.rs

## Contexto
Persistencia SQLite WAL para Listagem e Atualizacao de Ciclos Ativos atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** o banco de dados `.trustkit/state.db` contendo ciclos em status `ACTIVE` e `MERGED`
- **Quando** o adaptador `SqliteWalRepositoryAdapter::list_active_cycles` for invocado
- **Entao** retorna todos os registros com `lifecycle_status = 'ACTIVE'` e permite a atualizacao atomica em lote para `MERGED` sem corromper a integridade WAL.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-sqlite-active-cycles.json
