# TASK-004: Sincronizacao de Documentacao, Manual do Usuario e Thin Skill

layer: CLI
depends_on: [TASK-003]
target_test_file: crates/trustkit-cli/tests/update_documentation_sync_tests.rs
expected_files:
  - README.md
  - docs/manual/MANUAL_DO_USUARIO.md
  - skills/trustkit-sdd/SKILL.md
  - .agents/skills/trustkit-sdd/SKILL.md
  - crates/trustkit-cli/tests/update_documentation_sync_tests.rs

## Contexto
Atualizar `README.md`, `docs/manual/MANUAL_DO_USUARIO.md`, `skills/trustkit-sdd/SKILL.md` e `.agents/skills/trustkit-sdd/SKILL.md` documentando os comandos `trustkit update`, `tk update --check` e `tk update --force` sem emojis e sem AI trailers, validados por suite de sincronizacao documental, atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** os documentos `README.md`, `docs/manual/MANUAL_DO_USUARIO.md`, `skills/trustkit-sdd/SKILL.md` e `.agents/skills/trustkit-sdd/SKILL.md`
- **Quando** a suite de sincronizacao documental validar os guias de uso do comando `update`
- **Entao** todos documentam `trustkit update`, `tk update --check` e `tk update` com zero emojis e zero trailers de IA.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-update-documentation-sync.json
