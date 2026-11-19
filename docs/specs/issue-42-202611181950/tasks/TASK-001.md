# TASK-001: Contratos de Dominio e Caso de Uso de Reconciliacao de Ciclos

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/cycle_reconciliation_domain_tests.rs
expected_files:
  - crates/trustkit-core/src/cycle_reconciliation_domain.rs
  - crates/trustkit-core/tests/cycle_reconciliation_domain_tests.rs

## Contexto
Contratos de Dominio e Caso de Uso de Reconciliacao de Ciclos atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o ciclo de vida SDD e os contratos de dominio em `trustkit-core`
- **Quando** o caso de uso de reconciliacao avaliar multiplos ciclos ativos existentes no repositorio
- **Entao** classifica com precisao quais ciclos estao orfaos no Gate 0 sem tarefas, quais possuem tarefas ativas e resolve os proximos passos deterministicos via `NextStepResolver`.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-cycle-reconciliation-domain.json
