# TASK-005: Caso de Uso query_wave_view

layer: CORE
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-core/tests/wave_view_empty_cycle_tests.rs
expected_files:
  - crates/trustkit-core/src/wave_view_empty_cycle.rs
  - crates/trustkit-core/tests/wave_view_empty_cycle_tests.rs

## Contexto
Caso de Uso query_wave_view atendendo RN-005 e RF-066 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo ativo que ainda nao passou pelo Gate 3 (`trustkit wave plan`) ou nao possui tarefas cadastradas
- **Quando** `trustkit wave view` for executado
- **Entao** o comando retorna erro semantico `DomainViolation::PreconditionNotMet` ou mensagem amigavel indicando que o planejamento de ondas deve ser executado previamente, sem provocar panic ou unwrap falho.

## Rastreabilidade
- Regra de Negocio: RN-005
- Requisito Funcional: RF-066
- Evidencia: EV-005-wave-view-empty-cycle.json
