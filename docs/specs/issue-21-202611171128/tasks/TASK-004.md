# TASK-004: Documentacao e Sincronizacao de Skills com Alias tk

layer: CLI
depends_on:
  - TASK-003
target_test_file: crates/trustkit-cli/tests/tk_doc_and_skills_sync_tests.rs
expected_files:
  - skills/trustkit-sdd/SKILL.md
  - .agents/skills/trustkit-sdd/SKILL.md
  - crates/trustkit-cli/tests/tk_doc_and_skills_sync_tests.rs

## Contexto
Atualizar os arquivos de definicao da skill trustkit-sdd em skills/trustkit-sdd/SKILL.md e .agents/skills/trustkit-sdd/SKILL.md, informando a disponibilidade oficial do alias abreviado tk e adicionando testes de sincronizacao de documentacao.

## Criterios de Aceitacao (Gherkin)
- **Dado** os documentos de instrucao técnica e as definicoes de skills do TRustKit SDD
- **Quando** o desenvolvedor ou agente consultar os comandos recomendados
- **Entao** a documentacao orienta explicitamente a disponibilidade e equivalencia do binario abreviado tk como alias oficial.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-tk-doc-and-skills-sync.json
