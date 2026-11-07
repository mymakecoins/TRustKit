# TASK-006: Sincronizacao Documental, Manuais Markdown e HTML, README e Archify

> **Camada:** DOCS
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-005
layer: DOCS
Arquivos Esperados:
- README.md
- docs/PRD_TRustKit.md
- docs/ADR_001_TRustKit_Architecture.md
- docs/manuals/user-guide.md
- docs/manuals/user-guide.html
test_file: crates/trustkit-cli/tests/doc_and_manual_sync_tests.rs

---

## Contexto
Toda alteracao arquitetural e novas capacidades do TRustKit devem ser documentadas rigorosamente no README, no PRD, no ADR-001, nos manuais de usuario em Markdown e HTML autonomo com CSS embutido, e refletidas nos diagramas de arquitetura Archify.

## Escopo
- Atualizar a secao de comandos e jornada de SDD no `README.md` incluindo o passo previo de milestones.
- Atualizar `docs/PRD_TRustKit.md` e `docs/ADR_001_TRustKit_Architecture.md` com a nova fase de engenharia e os contratos do comando `milestones`.
- Atualizar o manual do usuario em Markdown (`docs/manuals/user-guide.md`) e gerar a versao compilada HTML standalone (`docs/manuals/user-guide.html`).
- Assegurar conformidade constitucional: zero emojis, zero trailers de autoria IA e validacao em `crates/trustkit-cli/tests/doc_and_manual_sync_tests.rs`.
