# TASK-005: Apresentacao CLI e Persistencia SQLite no Gate 4

layer: CLI
depends_on: ["TASK-004"]
target_test_file: crates/trustkit-cli/tests/e2e_dual_axis_review_tests.rs
expected_files:
  - crates/trustkit-cli/src/e2e_dual_axis_review.rs
  - crates/trustkit-cli/tests/e2e_dual_axis_review_tests.rs

## Contexto
Apresentacao CLI e Persistencia SQLite no Gate 4 atendendo RN-005 e RF-005 no componente Node5.

## Criterios de Aceitacao (Gherkin)
- **Dado** a conclusao de uma tarefa no Gate 4 com a flag `--code-review medium` ou superior
- **Quando** ambos os eixos (`StandardsAndSmells` e `SpecCompliance`) retornam `PASS`
- **Entao** a evidencia dual-axis e gravada no SQLite WAL e a tarefa e marcada como concluida com codigo de saida `0`.

## Rastreabilidade
- Regra de Negocio: RN-005
- Requisito Funcional: RF-005
- Evidencia: EV-005-e2e-gate4-dual-review.json
