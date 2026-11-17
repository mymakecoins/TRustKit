# TASK-004: Agregador DualAxisReviewReport Sem Reranking Cruzado

layer: CLI
depends_on: ["TASK-003", "TASK-002"]
target_test_file: crates/trustkit-cli/tests/dual_axis_presenter_tests.rs
expected_files:
  - crates/trustkit-cli/src/dual_axis_presenter.rs
  - crates/trustkit-cli/tests/dual_axis_presenter_tests.rs

## Contexto
Agregador DualAxisReviewReport Sem Reranking Cruzado atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** um `DualAxisReviewReport` onde o eixo `StandardsAndSmells` obteve veredito `PASS` com zero problemas, mas o eixo `SpecCompliance` obteve veredito `FAIL`
- **Quando** o resultado consolidado do comando `trustkit task review <TASK_ID>` e calculado
- **Entao** o veredito global da revisao e `FAIL`, mantendo as secoes `standards_axis` e `spec_axis` independentes no JSON e no terminal ASCII.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-dual-axis-non-compensatory.json
