# TASK-002: Caso de Uso query_wave_view

layer: CORE
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-core/tests/query_wave_view_use_case_tests.rs
expected_files:
  - crates/trustkit-core/src/query_wave_view_use_case.rs
  - crates/trustkit-core/tests/query_wave_view_use_case_tests.rs

## Contexto
Caso de Uso query_wave_view atendendo RN-002 e RF-063 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo ativo registrado no banco SQLite WAL (`.trustkit/state.db`) com tarefas distribuidas em multiplas ondas
- **Quando** o caso de uso `query_wave_view` consultar as tarefas atomicas persistidas
- **Entao** as tarefas sao agrupadas por `wave_index` crescente, ordenadas internamente por `task_id` alfanumerico e expõem `task_id`, `architectural_layer`, `title`, lista de dependencias (`depends_on`) e status do ciclo TDD (`tdd_lifecycle_state`).

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-063
- Evidencia: EV-002-query-wave-view-use-case.json
