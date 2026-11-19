# TASK-003: Inspecao Git de Branches e Ciclos Integrados na Branch Alvo

layer: GH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-gh/tests/git_branch_integration_tests.rs
expected_files:
  - crates/trustkit-gh/src/git_branch_integration.rs
  - crates/trustkit-gh/tests/git_branch_integration_tests.rs

## Contexto
Inspecao Git de Branches e Ciclos Integrados na Branch Alvo atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** um repositorio Git com historico de branches e commits de ciclos anteriores
- **Quando** o `GitInspector::is_branch_merged_into` testar se a branch ou commit base de um ciclo ja e ancestral da branch alvo
- **Entao** retorna verdadeiro se o commit ja estiver integrado no historico alvo e falso caso a branch ainda possua alteracoes exclusivas pendentes.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-git-branch-integration.json
