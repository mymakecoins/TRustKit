# TASK-009: Sincronizar Documentacao Bilingue e Thin Skill trustkit-sdd

> **Camada:** Documentacao / Governanca
> **Estimativa:** S
> **Status:** Pendente

depends_on: [TASK-007, TASK-008]
layer: DOCS
Arquivos Esperados:
- README.md
- README.en.md
- .agents/skills/trustkit-sdd/SKILL.md
test_file: crates/trustkit-ast/tests/skill_trustkit_sdd_tests.rs

---

## Contexto
Com a adicao do suporte a multiplexacao de PTYs via `multiplexador PTY`, emissao de telemetria `OSC 7501`, novas flags no Gate 4 e ferramentas MCP de terminais, a documentacao oficial do projeto e as instrucoes do agente orquestrador na Thin Skill `trustkit-sdd` devem ser atualizadas em estrita paridade bilingue (Portugues do Brasil e Ingles).

## Escopo
Atualizar:
1. `README.md`:
   - Detalhar a integracao nativa com `multiplexador PTY` no Gate 4 e o funcionamento do protocolo `OSC 7501`.
   - Documentar os novos parametros de `trustkit wave run --runtime <auto|pty|native>`.
   - Listar as novas ferramentas MCP de inspecao de terminais.
2. `README.en.md`:
   - Atualizacao espelho em Ingles com identica densidade tecnica e precisao terminologica.
3. `.agents/skills/trustkit-sdd/SKILL.md`:
   - Atualizar a jornada do Gate 4 para orientar o agente orquestrador sobre o uso das novas opcoes de runtime e ferramentas MCP.
4. Validar conformidade de invariantes:
   - Proibicao absoluta de emojis ou simbolos graficos.
   - Proibicao absoluta de trailers de IA.

### Fora do escopo
- Alteracoes em codigo-fonte Rust.

## Dependencias
- TASK-007
- TASK-008

## Premissas
- Estrita paridade estrutural e de conteudo entre `README.md` e `README.en.md`.
- Aprovacao sem advertencias pelo scanner lexico do Gauntlet Camada 0.

## Criterio de Aceite
- [ ] **Dado** os arquivos atualizados `README.md` e `README.en.md`, **quando** auditados pelo scanner lexico, **entao** nenhum emoji Unicode ou trailer de autoria IA e detectado.
- [ ] **Dado** a Thin Skill `trustkit-sdd`, **quando** executada a suite `skill_trustkit_sdd_tests.rs`, **entao** todos os testes passam comprovando integridade de governanca.

## Notas Tecnicas
- Seguir o padrao dos ciclos anteriores em `docs/`.

## Rastreabilidade
- Origem: RF-061, RN-009
