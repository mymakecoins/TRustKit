# TASK-001: Orientacao Explicita de Autoria de Teste RED no NextStepResolver

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/src/next_step_guidance_tests.rs
expected_files:
  - crates/trustkit-core/src/next_step_guidance.rs
  - crates/trustkit-core/src/next_step_guidance_tests.rs

## Contexto
Ajuste no NextStepResolver para que ao finalizar wave run com tarefas pendentes, a orientacao instrua expressamente a redigir a assercao de teste falhando no target_test_file antes de rodar task test --phase red, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma onda executada com tarefas pendentes em estado AwaitingRed
- **Quando** o NextStepResolver calcular as recomendacoes de proximo passo para wave run
- **Entao** a descricao da recomendacao primaria orienta expressamente o desenvolvedor a redigir a assercao de teste falhando no target_test_file antes de rodar o comando task test phase red.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-wave-run-next-step-red-guidance.json
