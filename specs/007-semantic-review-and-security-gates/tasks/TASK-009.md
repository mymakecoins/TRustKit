# TASK-009: Atualizar Documentacao Bilingue (`README.md` e `README.en.md`) com Novos Gates e Comandos

> **Camada:** Documentacao Principal
> **Estimativa:** P
> **Status:** Concluido

depends_on: [TASK-005, TASK-008]
layer: CLI
Arquivos Esperados:
- README.md
- README.en.md
test_file: crates/trustkit-cli/src/cli_contract_help_tests.rs

---

## Contexto
Para manter a documentacao raiz do repositorio em sincronia com as novas capacidades da feature `007-semantic-review-and-security-gates`, e necessario atualizar tanto o `README.md` (Portugues do Brasil) quanto o `README.en.md` (Ingles) mantendo paridade de secoes, exemplos de linha de comando e invariantes constitucionais.

## Escopo
Atualizar em `README.md` e `README.en.md`:
- Secao **Gate 4**: Incluir o passo de revisao semantica de codigo na fase Refactor (`trustkit task review <TASK_ID> [--effort <low|medium|high|max>]` e flag `--code-review <nivel>` em `trustkit task complete`).
- Secao **Gate 5 & Gate 6**: Documentar o modelo de interacao da revisao de seguranca opcional (`security-review`), incluindo:
  - Decisao antecipada via parametros CLI (`--security-review` e `--skip-security-review`) em `trustkit gauntlet run` e `trustkit done`.
  - Prompt interativo TTY (`[s/N]`) quando nenhuma flag for passada e fallback automatico no modo `--json`.
  - Configuracao declarativa `[governance.gauntlet]` (`security_review_mode = "prompt" | "always" | "disabled"` e `min_confidence_threshold = 8`).
- Nova secao **Auditoria Global Pre-Release**: Documentar o comando desacoplado `trustkit audit security [--profile <quick|standard|deep>] [--json]`.

### Fora do escopo
- Alteracao de diagramas HTML/SVG em `docs/diagrams/` (tratada na TASK-010).

## Dependencias
- TASK-005
- TASK-008

## Premissas
- Paridade estrutural entre `README.md` e `README.en.md`.
- Proibicao absoluta de emojis Unicode ou trailers de autoria IA (RN-007 / RN-013).

## Criterio de Aceite
- [x] **Dado** os arquivos `README.md` e `README.en.md` atualizados, **quando** inspecionados pelo `InvariantLexicalScanner`, **entao** passam com zero violacoes de emojis ou trailers de IA.
- [x] **Dado** os exemplos CLI documentados em `README.md` e `README.en.md`, **quando** comparados ao contrato Clap (`cli_contract_help_tests.rs`), **entao** refletem fielmente as flags `--security-review`, `--skip-security-review`, `task review` e `audit security`.

## Notas Tecnicas
- Seguir a formatacao existente das secoes 4 a 8 em ambos os arquivos.

## Rastreabilidade
- Origem: RF-041, RN-009
