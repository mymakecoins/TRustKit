# TASK-002: Orientacao Contextual de Autoria RED no NextStepStatusPolicy

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/src/next_step_status_policy_tests.rs
expected_files:
  - crates/trustkit-core/src/next_step_status_policy.rs
  - crates/trustkit-core/src/next_step_status_policy_tests.rs

## Contexto
Aprimoramento da NextStepStatusPolicy para que tarefas em estado AwaitingRed emitam orientacao para a escrita do teste de assercao falhando no arquivo alvo antes da execucao de task test --phase red, atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo ativo com tarefa pendente aguardando a fase RED
- **Quando** a politica de proximo passo de status for calculada
- **Entao** o proximo passo orienta a escrita do teste falhando no arquivo de teste alvo.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-status-policy-red-authoring.json
