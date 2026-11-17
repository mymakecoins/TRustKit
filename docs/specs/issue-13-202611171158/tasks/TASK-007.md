# TASK-007: Sincronizacao de Manuais e Documentacao

layer: AST
depends_on: ["TASK-003", "TASK-006"]
target_test_file: crates/trustkit-ast/tests/skill_wave_view_docs_tests.rs
expected_files:
  - crates/trustkit-ast/src/skill_wave_view_docs.rs
  - crates/trustkit-ast/tests/skill_wave_view_docs_tests.rs

## Contexto
Sincronizacao de Manuais e Documentacao atendendo RN-007 e RF-068 no componente Node7.

## Criterios de Aceitacao (Gherkin)
- **Dado** a introducao do subcomando `trustkit wave view` e da ferramenta MCP `trustkit_wave_view`
- **Quando** a entrega da funcionalidade for finalizada
- **Entao** toda a documentacao de apoio do projeto (`README.md`, `README.en.md`, manuais da CLI e guias da Thin Skill `.agents/skills/trustkit-sdd/SKILL.md`) deve ser revisada e atualizada exclusivamente nos pontos pertinentes para documentar a sintaxe, flags (`--json`, `--cycle-id`), exemplos de saida e casos de uso de inspecao de ondas, mantendo paridade bilingue e zero emojis.

## Rastreabilidade
- Regra de Negocio: RN-007
- Requisito Funcional: RF-068
- Evidencia: EV-007-wave-view-docs-sync.json
