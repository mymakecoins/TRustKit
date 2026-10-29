# TASK-001: Implementar Entidades de Dominio e Portas de Memoria Perene

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Concluida

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/knowledge_entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/src/knowledge_entities_tests.rs

---

## Contexto
O subsistema de memoria perene e base de conhecimento do repositorio necessita de modelos de dominio puros, tipos de casos especiais livres de nulos e traits de porta (Hexagonal Architecture) para desacoplar as regras de negocio da persistencia em disco e do indice SQLite FTS5.

## Escopo
Modelar as estruturas de dominio e portas:
- `KnowledgeDocument`: slug, titulo, lista de tags, corpo em Markdown, carimbo de atualizacao e digest BLAKE3.
- `WikiLink`: origem (`source_slug`) e destino (`target_slug`).
- `KnowledgeSearchQuery` e `KnowledgeSearchHit` (slug, titulo, score BM25, snippet destacado e links vizinhos).
- Casos especiais em enum: `KnowledgeLookupOutcome::Found(KnowledgeDocument)` / `KnowledgeLookupOutcome::DocumentNotFound` e `KnowledgeSearchOutcome::Results(Vec<KnowledgeSearchHit>)` / `KnowledgeSearchOutcome::EmptyKnowledgeList`.
- Trait `KnowledgeRepositoryPort` em `ports.rs`.

### Fora do escopo
- Implementacao de I/O em sistema de arquivos ou queries SQLite.

## Dependencias
- Nenhuma

## Premissas
- Proibicao estrita de valores `null` na serializacao e nas structs.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um documento de conhecimento valido, **quando** instanciado no dominio, **entao** preserva todos os atributos e metadados sem campos opcionais nulos.
- [x] **Dado** uma consulta sem resultados, **quando** representada pelo padrao de Caso Especial, **entao** serializa como lista vazia imutavel `[]` com variante explicita.

## Notas Tecnicas
- Seguir estritamente o Principio Constitucional I (Isolamento Hexagonal) e II (Proibicao de NULL).

## Rastreabilidade
- Origem: RF-017, RN-001
