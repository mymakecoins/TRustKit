# TASK-005: Scaffolding de Teste RED Compilavel na Worktree

layer: AST
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-ast/tests/red_test_scaffolder_tests.rs
expected_files:
  - crates/trustkit-ast/src/red_test_scaffolder.rs
  - crates/trustkit-ast/tests/red_test_scaffolder_tests.rs

## Contexto
Gerar automaticamente dentro da worktree isolada o esqueleto do arquivo de teste alvo (`target_test_file`) quando este ainda nao existir no disco antes da verificacao da fase RED, garantindo que o arquivo compile sem erros de sintaxe e contenha um teste explicito que falhe por assercao de dominio.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma tarefa atomica cuja propriedade `target_test_file` aponta para um arquivo de teste ainda inexistente na worktree
- **Quando** o `RedTestScaffolder` for acionado antes da verificacao da fase RED
- **Entao** deve criar o arquivo de teste compilavel com assercao de falha explicita referenciando a tarefa e seus criterios.

## Rastreabilidade
- Regra de Negocio: RN-005
- Requisito Funcional: RF-005
- Evidencia: EV-005-red-test-scaffolder.json
