# TASK-003: Sub-avaliador Eixo 1 Standards e 12 Fowler Smells

layer: AST
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-ast/tests/fowler_smells_lexical_tests.rs
expected_files:
  - crates/trustkit-ast/src/fowler_smells_lexical.rs
  - crates/trustkit-ast/tests/fowler_smells_lexical_tests.rs

## Contexto
Sub-avaliador Eixo 1 Standards e 12 Fowler Smells atendendo RN-003 e RF-003 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** que o baseline de Fowler sinalizaria um smell heuristico (como `RepeatedSwitches`), mas as regras do repositorio em `AGENTS.md` ou `.claude/rules/` endossam explicitamente `match` exaustivo sem polimorfismo dinamico
- **Quando** o avaliador do eixo `StandardsAndSmells` processa as regras do repositorio
- **Entao** o padrao documentado do repositorio suprime o smell conflitante, evitando falsos positivos contra a arquitetura local.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-repo-standard-override-smell.json
