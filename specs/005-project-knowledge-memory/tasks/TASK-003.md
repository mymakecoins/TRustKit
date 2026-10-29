# TASK-003: Implementar Tabelas FTS5, Grafo de Adjacencia e Busca BM25 no SQLite

> **Camada:** Persistencia / Banco de Dados
> **Estimativa:** M
> **Status:** Concluida

depends_on: [TASK-001]
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/knowledge_ops.rs
- crates/trustkit-db/src/migrations.rs
test_file: crates/trustkit-db/tests/knowledge_fts5_search_tests.rs

---

## Contexto
Para proporcionar recuperacao textual instantanea sem varredura linear de arquivos a cada consulta, o `.trustkit/state.db` deve manter uma tabela virtual SQLite FTS5 (`knowledge_pages_fts`) e uma tabela relacional de arestas (`knowledge_wiki_links`).

## Escopo
Implementar na camada `trustkit-db`:
- DDL em `migrations.rs` criando a tabela virtual `knowledge_pages_fts USING fts5(slug UNINDEXED, title, tags, body_content, updated_at UNINDEXED, blake3_digest UNINDEXED, tokenize = 'porter unicode61')` e a tabela `knowledge_wiki_links (source_slug TEXT NOT NULL, target_slug TEXT NOT NULL, PRIMARY KEY (source_slug, target_slug))`.
- Operacoes em `knowledge_ops.rs` para indexar documento (`upsert_knowledge_index`), remover, consultar vizinhos de grafo (`query_adjacent_links`) e buscar por termos com ranqueamento BM25 e extracao de snippets (`search_knowledge_fts5`).

### Fora do escopo
- Leitura ou escrita direta de arquivos `.md` no sistema de arquivos.

## Dependencias
- TASK-001

## Premissas
- Manter modo WAL e restricoes `NOT NULL` em todas as tabelas relacionais.
- Consultas FTS5 devem sanitizar caracteres especiais de sintaxe para evitar erros de query (Zero Panic).

## Criterio de Aceite
- [x] **Dado** multiplos documentos indexados no SQLite FTS5, **quando** executada busca textual por palavras-chave, **entao** retorna os registros ordenados por relevância BM25 em `< 3ms` com os links adjacentes preenchidos.
- [x] **Dado** uma string de busca com caracteres reservados, **quando** consultada, **entao** o adaptador sanitiza a expressao sem lancar panico ou erro SQL.

## Notas Tecnicas
- Arquivos alvo: `crates/trustkit-db/src/migrations.rs` e `crates/trustkit-db/src/knowledge_ops.rs`.

## Rastreabilidade
- Origem: RF-019, RN-003
