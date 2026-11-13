# TASK-003: Calculador AST de Merge Danger Door e Blast Radius

layer: CORE
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-core/tests/certification_merge_pr_use_case_tests.rs
expected_files:
  - crates/trustkit-core/src/certification_merge_pr_use_case.rs
  - crates/trustkit-core/tests/certification_merge_pr_use_case_tests.rs

## Contexto
Calculador AST de Merge Danger Door e Blast Radius atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** o diff consolidado do ciclo pronto para abertura de Pull Request via `trustkit pr`
- **Quando** o calculador de risco analisa os arquivos e nos afetados no grafo estrutural
- **Entao** mudancas que alteram migracoes de banco (`migrations.rs`), schemas persistidos ou contratos de portas publicas sao classificadas como `OneWayDoor`, enquanto mudancas restritas a adaptadores internos ou testes sao classificadas como `TwoWayDoor`, acompanhadas do `BlastRadiusLevel` e incluidas obrigatoriamente no corpo do PR.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-merge-danger-door-blast-radius.json
