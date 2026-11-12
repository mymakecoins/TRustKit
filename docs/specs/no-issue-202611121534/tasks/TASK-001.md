# TASK-001: Parser de Tarefas e Extracao de Alvos em TASK-NNN.md

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/src/ast_entities_tests.rs
expected_files:
  - crates/trustkit-core/src/ast_entities_tests.rs

## Contexto
Parser de Tarefas e Extracao de Alvos em TASK-NNN.md atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** um conjunto de tarefas em `tasks/TASK-NNN.md` onde cada tarefa corta verticalmente camadas distintas sem modificar assinaturas de simbolos de alto `in_degree`
- **Quando** o analisador `WaveCouplingAnalyzer` inspeciona as tarefas contra o grafo estrutural
- **Entao** as tarefas sao classificadas como `VerticalTracerBullet` e agendadas em ondas paralelas para execucao imediata nas Git Worktrees.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-task-slice-kind-entities.json
