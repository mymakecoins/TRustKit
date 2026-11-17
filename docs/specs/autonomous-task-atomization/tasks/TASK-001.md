# TASK-001: Extracao AST da Matriz de Rastreabilidade e Cenarios Gherkin

layer: AST
depends_on: []
target_test_file: crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs
expected_files:
  - crates/trustkit-ast/src/spec_matrix_atomizer.rs
  - crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs

## Contexto
Extrair deterministicamente da tabela da Matriz de Rastreabilidade (secao 4), do diagrama Mermaid (secao 2) e dos cenarios Gherkin (secao 3) de um documento `spec.md` a lista de rascunhos de tarefas atomicas (`AtomicTaskDraft`), inferindo a camada arquitetural, arquivos esperados, arquivo de teste alvo e dependencias topologicas.

## Criterios de Aceitacao (Gherkin)
- **Dado** um conteudo Markdown de especificacao formal contendo Matriz de Rastreabilidade e cenarios Gherkin
- **Quando** o `SpecMatrixAtomizer::atomize_markdown` for invocado
- **Entao** deve retornar a colecao ordenada de tarefas atomicas sem campos nulos e em tempo inferior a 50ms.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-spec-matrix-ast-atomization.json
