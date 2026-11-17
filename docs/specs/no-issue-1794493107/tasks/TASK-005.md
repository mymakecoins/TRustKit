# TASK-005: Saida CLI e JSON em trustkit wave plan

layer: CLI
depends_on: ["TASK-004"]
target_test_file: crates/trustkit-cli/tests/e2e_wave_plan.rs
expected_files:
  - crates/trustkit-cli/src/e2e_wave_plan.rs
  - crates/trustkit-cli/tests/e2e_wave_plan.rs

## Contexto
Saida CLI e JSON em trustkit wave plan atendendo RN-005 e RF-005 no componente Node5.

## Criterios de Aceitacao (Gherkin)
- **Dado** um plano de ondas valido processado pelo `trustkit wave plan`
- **Quando** a saida e renderizada nos modos ASCII humano e `--json`
- **Entao** cada tarefa exibe seu `slice_kind` e `blast_radius_score` sem campos nulos e sem emojis.

## Rastreabilidade
- Regra de Negocio: RN-005
- Requisito Funcional: RF-005
- Evidencia: EV-005-cli-wave-plan-expand-contract.json
