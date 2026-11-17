# TASK-004: Atomizacao Dirigida por Corpo de GitHub Issue

layer: GH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-gh/tests/issue_body_atomizer_tests.rs
expected_files:
  - crates/trustkit-gh/src/issue_body_atomizer.rs
  - crates/trustkit-gh/tests/issue_body_atomizer_tests.rs

## Contexto
Permitir que o comando `trustkit auto --issue <ID>` extraia requisitos funcionais, criterios Gherkin e tabelas de rastreabilidade diretamente do corpo Markdown de uma GitHub Issue sincronizada, convertendo-os em tarefas atomicas estruturadas.

## Criterios de Aceitacao (Gherkin)
- **Dado** o corpo textual de uma issue do GitHub contendo especificacao ou lista de requisitos funcionais
- **Quando** o `IssueBodyAtomizer` processar o payload da issue
- **Entao** deve derivar as tarefas atomicas correspondentes mantendo rastreabilidade com o numero da issue.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-issue-body-atomization.json
