# TASK-010: Atualizar Diagramas Interativos Archify em `docs/diagrams/`

> **Camada:** Visualizacao Arquitetural (Archify)
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-005, TASK-008]
layer: CLI
Arquivos Esperados:
- docs/diagrams/01-trustkit-architecture.json
- docs/diagrams/01-trustkit-architecture.html
- docs/diagrams/02-trustkit-7gates-workflow.json
- docs/diagrams/02-trustkit-7gates-workflow.html
- docs/diagrams/03-trustkit-task-tdd-sequence.json
- docs/diagrams/03-trustkit-task-tdd-sequence.html
- docs/diagrams/05-trustkit-feature-lifecycle.json
- docs/diagrams/05-trustkit-feature-lifecycle.html
test_file: crates/trustkit-ast/tests/lexical_scanner_tests.rs

---

## Contexto
Os diagramas interativos standalone gerados com a skill `archify` em `docs/diagrams/` representam a topologia do workspace, o fluxo dos 7 Gates, a sequencia TDD de cada tarefa e o ciclo de vida completo de uma feature. Com a introducao de `code-review` (Gate 4), `security-review` opcional (Gates 5 e 6) e `security-audit` (Pre-Release), os JSONs fonte e os HTMLs renderizados devem ser atualizados e validados.

## Escopo
Atualizar as especificacoes `.json` e regenerar/validar os arquivos `.html` via pipeline `archify`:
- `01-trustkit-architecture`: Incluir as portas `SemanticCodeReviewerPort`, `SecurityDiffReviewerPort` e `InteractivePromptPort` na camada Core e seus adaptadores em CLI/Orch.
- `02-trustkit-7gates-workflow`: Representar no Gate 4 a etapa `Code Review (Refactor)`, na transicao Gate 5 -> Gate 6 o portao condicional `Security Review (Optional: CLI Flag / TTY Prompt [s/N])` e apos o Gate 6 a trilha desacoplada `Pre-Release Security Audit`.
- `03-trustkit-task-tdd-sequence`: Incluir na sequencia `RED -> GREEN -> REFACTOR` a chamada a `trustkit task review` e o registro de `EvidenceKind::CodeReview` no SQLite WAL.
- `05-trustkit-feature-lifecycle`: Incluir a ramificacao de decisao da `security-review` (`--security-review` / `--skip-security-review` / Prompt `[s/N]`) antes da emissao do `gauntlet-report.md` e a etapa pre-release `trustkit audit security`.

### Fora do escopo
- Alteracao do diagrama `04-trustkit-memory-dataflow` caso o fluxo de indexacao FTS5 nao mude estruturalmente.

## Dependencias
- TASK-005
- TASK-008

## Premissas
- Validacao obrigatoria com o script `validate_html.py` da skill `archify` (zero overlaps de nos/arestas/labels, suporte a tema claro/escuro e exportacao SVG/PNG/WebM).
- Zero emojis Unicode nos arquivos `.json` e `.html` gerados.

## Criterio de Aceite
- [x] **Dado** os arquivos `.json` e `.html` atualizados em `docs/diagrams/`, **quando** validados pelo `validate_html.py` do Archify, **entao** passam sem colisoes geometricas ou erros de layout.
- [x] **Dado** os arquivos atualizados em `docs/diagrams/`, **quando** verificados pelo `InvariantLexicalScanner`, **entao** nao contem emojis Unicode ou trailers de autoria IA.

## Notas Tecnicas
- Seguir as diretrizes da skill `archify` (`/home/mmc/.gemini/config/skills/archify/SKILL.md`).

## Rastreabilidade
- Origem: RF-042, RN-010
