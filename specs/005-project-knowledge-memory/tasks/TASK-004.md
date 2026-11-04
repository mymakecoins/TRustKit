# TASK-004: Implementar Repositorio de Arquivos Wiki e Reconstrucao de Indice

> **Camada:** Persistencia / Sistema de Arquivos
> **Estimativa:** M
> **Status:** Concluida

depends_on: [TASK-002, TASK-003]
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/knowledge_wiki_fs.rs
test_file: crates/trustkit-db/tests/knowledge_rebuild_index_tests.rs

---

## Contexto
A pasta `.trustkit/wiki/` versionada no Git atua como a Fonte Unica da Verdade, enquanto o SQLite FTS5 funciona como indice derivado. Este modulo coordena a escrita atomica de arquivos Markdown no disco em sincronia com o indice FTS5 e prove a rotina de reconstrucao integral (`rebuild`) a partir do diretorio.

## Escopo
Implementar em `crates/trustkit-db/src/knowledge_wiki_fs.rs`:
- Gravacao atomica de arquivos `.md` sob `.trustkit/wiki/<slug>.md` com validacao anti-path-traversal, geracao de frontmatter, calculo de hash BLAKE3 e atualizacao imediata do indice FTS5 e links.
- Leitura de documento por slug diretamente do arquivo Markdown.
- Rotina `rebuild_from_wiki_directory` que limpa as tabelas `knowledge_pages_fts` e `knowledge_wiki_links`, varre recursivamente `.trustkit/wiki/**/*.md`, extrai frontmatter e wikilinks e reindexa 100% da base em transacao unica.

### Fora do escopo
- Gatilhos automaticos dos Gates SDD (cobertos por TASK-005 a TASK-007).

## Dependencias
- TASK-002
- TASK-003

## Premissas
- Caso o banco `.trustkit/state.db` seja recriado do zero, `rebuild_from_wiki_directory` deve restaurar 100% da capacidade de busca sem perda de informacao.

## Criterio de Aceite
- [x] **Dado** um diretorio `.trustkit/wiki/` populado com notas em subpastas (`adrs/`, `specs/`, `tasks/`, `incidents/`) e um indice FTS5 vazio, **quando** a reconstrucao e executada, **entao** todas as notas e wikilinks sao indexados e retornam `KnowledgeRebuildSummary` exato.
- [x] **Dado** uma tentativa de gravacao com slug contendo `../`, **quando** validada, **entao** a operacao e recusada com erro de fronteira de seguranca.

## Notas Tecnicas
- Arquivo alvo: `crates/trustkit-db/src/knowledge_wiki_fs.rs`.

## Rastreabilidade
- Origem: RF-017, RF-020, RN-004
