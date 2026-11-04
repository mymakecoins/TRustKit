# TASK-003: Implementar Persistencia Incremental com Manifesto BLAKE3 e Shrink-Guard no SQLite

> **Camada:** Persistencia SQLite WAL
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-001]
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/graph_sqlite_repository.rs
- crates/trustkit-db/src/migrations.rs
test_file: crates/trustkit-db/tests/graph_sqlite_repository_tests.rs

---

## Contexto
Para suportar atualizacoes incrementais (`--update`) instantaneas e evitar que uma extracao parcial ou invalida apague silenciosamente o grafo do projeto, o banco `.trustkit/state.db` deve armazenar o manifesto `BLAKE3` de cada arquivo e aplicar a trava Shrink-Guard antes de comitar alteracoes.

## Escopo
Implementar em `crates/trustkit-db`:
- Em `migrations.rs`: adicionar as tabelas `structural_graph_manifests` (`file_path`, `blake3_digest`, `node_count`, `updated_at`), `structural_graph_nodes` e `structural_graph_edges` com restricoes `100% NOT NULL`.
- Em `graph_sqlite_repository.rs`: implementar `StructuralGraphRepositoryPort`:
  - `detect_changed_files`: compara os digests `BLAKE3` atuais com `structural_graph_manifests` para retornar apenas arquivos novos ou modificados.
  - `save_graph_snapshot`: persiste nos e arestas em transacao atomica SQLite WAL, mas verifica previamente o Shrink-Guard (se o novo grafo tiver menos nos que o grafo existente e `force == false`, aborta a escrita e retorna `GraphHealthVerdict::ShrinkBlocked`).

### Fora do escopo
- Extracao sintatica `tree-sitter` ou travessia em `petgraph`.

## Dependencias
- TASK-001

## Premissas
- Todas as colunas SQL devem possuir `NOT NULL` e valores default explicitos.
- Operacoes transacionais em `< 5ms`.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um grafo persistido com 50 nos no SQLite, **quando** uma atualizacao sem `--force` tenta gravar um grafo reduzido com 10 nos, **entao** o Shrink-Guard bloqueia a mutacao, mantem os 50 nos intactos e retorna `GraphHealthVerdict::ShrinkBlocked`.
- [x] **Dado** um conjunto de 20 arquivos onde apenas 2 tiveram alteracao de hash `BLAKE3`, **quando** `detect_changed_files` e executado, **entao** retorna exclusivamente os 2 arquivos modificados.

## Notas Tecnicas
- Seguir as convencoes de `crates/trustkit-db/src/knowledge_sqlite_repository.rs`.

## Rastreabilidade
- Origem: RF-027, RN-003
