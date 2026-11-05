# TASK-007: Unificar Fluxos de Especificacao na Thin Skill trustkit-sdd

> **Camada:** Skill / Prompt Agêntico
> **Estimativa:** P
> **Status:** Pendente

depends_on: [TASK-006]
layer: SKILL
Arquivos Esperados:
- .agents/skills/trustkit-sdd/SKILL.md
test_file: crates/trustkit-ast/tests/skill_trustkit_sdd_tests.rs

---

## Contexto
Com a disponibilidade dos comandos nativos `trustkit spec prompt`, `trustkit spec lint` e das ferramentas MCP no binario, a Thin Skill do TRustKit (`.agents/skills/trustkit-sdd/SKILL.md`) deve ser atualizada para instruir agentes sobre o fluxo completo de especificacao nativa do Gate 1 e Gate 2, tornando desnecessario o uso de `sdd-specify`.

## Escopo
Atualizar `.agents/skills/trustkit-sdd/SKILL.md`:
- Documentar a jornada do Gate 1 para formulacao de especificacao:
  - Uso de `trustkit spec prompt` ou ferramentas MCP para extracao de contexto e scaffold.
  - Redacao do `spec.md` sob estrita proibicao de emojis e trailers de IA.
  - Validacao continua com `trustkit spec lint`.
  - Congelamento no Gate 2 com `trustkit spec lock`.
- Incluir instrucoes para agentes sobre auto-correcao via saida estruturada do linter.

### Fora do escopo
- Alteracao de comandos de outros Gates (Gate 3 a Gate 6).

## Dependencias
- TASK-006

## Premissas
- Preservar limite de tokens da Thin Skill sem inflar desnecessariamente o contexto do agente.
- Ausencia rigorosa de emojis no arquivo de skill.

## Criterio de Aceite
- [ ] **Dado** o arquivo `.agents/skills/trustkit-sdd/SKILL.md`, **quando** auditado pelo scanner lexico, **entao** nao contem emojis, trailers de IA e descreve explicitamente os comandos `trustkit spec prompt` e `trustkit spec lint`.
- [ ] **Dado** uma instrucao de especificacao ao agente, **quando** orientada pela skill, **entao** direciona para o uso das ferramentas nativas do TRustKit.

## Notas Tecnicas
- Validar formato com os testes de invariantes em `crates/trustkit-ast/tests/lexical_scanner_tests.rs`.

## Rastreabilidade
- Origem: RF-049, RN-007
