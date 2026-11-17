# TASK-004: Alinhamento de Instrucoes e Autoria RED na Skill TRustKit SDD

layer: CLI
depends_on:
  - TASK-003
target_test_file: tests/skill_guidance_tests.rs
expected_files:
  - .agents/skills/trustkit-sdd/SKILL.md
  - tests/skill_guidance_tests.rs

## Contexto
Atualizacao formal da documentacao da skill trustkit-sdd explicitando no Gate 4 a etapa de redacao da assercao do teste falhando no target_test_file entre wave run e task test --phase red, atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** a secao do Gate 4 na documentacao da skill trustkit-sdd
- **Quando** o agente ou desenvolvedor consultar o fluxo recomendado
- **Entao** a documentacao explicita a etapa de autoria do teste falhando entre o provisionamento de worktree e a verificacao da fase RED.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-skill-flow-alignment.json
