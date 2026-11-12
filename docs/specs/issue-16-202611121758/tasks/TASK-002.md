# TASK-002: Validacao de Ciclo e Bloqueio de Fechamento para Ciclo Nao Certified

layer: CORE
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-core/src/use_cases/close_cycle_tests.rs
expected_files:
  - crates/trustkit-core/src/use_cases/close_cycle.rs
  - crates/trustkit-core/src/use_cases/close_cycle_tests.rs

## Contexto
Validacao de Ciclo e Gate 6 Certified atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo SDD ativo que ainda nao passou com sucesso pelo Gate 6
- **Quando** o desenvolvedor tenta executar `trustkit close`
- **Entao** o comando falha deterministicamente exigindo a certificacao previa via `trustkit done`.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-close-cycle-uncertified.json
