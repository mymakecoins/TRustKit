# TASK-002: Persistencia Atomica em glossary.md e FTS5

> **Camada:** DB
> **Estimativa:** M
> **Status:** Pendente

depends_on: ['TASK-001']
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/knowledge_ops.rs
- crates/trustkit-db/src/knowledge_wiki_fs.rs
test_file: crates/trustkit-db/tests/knowledge_fts5_search_tests.rs

---

## Contexto
Persistir o Glossario de Linguagem Ubiqua em formato Markdown canonico (.trustkit/wiki/glossary.md) e indexar os termos no SQLite FTS5 BM25.

## Escopo
- Implementar leitura e escrita atomica do glossario em `crates/trustkit-db/src/knowledge_wiki_fs.rs`.
- Adicionar indexacao e busca semantica FTS5 para termos canonicos e termos rejeitados em `crates/trustkit-db/src/knowledge_ops.rs`.
- Garantir que qualquer operacao de escrita assegura integridade sem valores nulos.
- Implementar testes de integracao em `crates/trustkit-db/tests/knowledge_fts5_search_tests.rs`.
