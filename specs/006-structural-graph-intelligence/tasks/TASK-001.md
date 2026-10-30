# TASK-001: Implementar Entidades de Dominio do Grafo Estrutural e Contratos de Porta

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Concluido

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/graph_entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/src/graph_entities_tests.rs

---

## Contexto
Para absorver a inteligencia de grafo estrutural no TRustKit mantendo o Principio Constitucional I (Isolamento Hexagonal estrito com `std` only) e II (Proibicao absoluta de `NULL`), e necessario modelar os nos, arestas com classificacao honesta de confianca, metricas de God Nodes, relatorio de integridade e Casos Especiais no crate `trustkit-core`.

## Escopo
Modelar as estruturas de dominio e portas:
- `GraphNodeKind`: `CodeFile`, `PublicSymbol`, `KnowledgeNote`, `SpecRule`, `AtomicTaskNode`.
- `EdgeConfidence`: `Extracted` (AST ou link explicito), `Inferred` (resolucao heuristica de caminho/modulo), `Ambiguous` (referencia parcial nao resolvida).
- `EdgeRelationKind`: `Imports`, `DefinesSymbol`, `ReferencesSymbol`, `WikiLinksTo`, `TracesTo`.
- `StructuralNode` e `StructuralEdge` (origem, destino, tipo de relacao, confianca e arquivo de origem).
- `GodNodeMetric` (`node_id`, `in_degree`, `out_degree`, `total_degree`, `coupled_neighbors`) e `SurprisingEdge` (arestas que cruzam camadas arquiteturais invalidas).
- `GraphHealthReport` (`total_nodes`, `total_edges`, `dangling_edges`, `self_loop_edges`, `collapsed_edges_count`, `shrink_guard_triggered`, `verdict: GraphHealthVerdict`).
- Casos especiais livres de `null`: `GraphTraversalOutcome::VisitedSlice(GraphContextSlice)` / `GraphTraversalOutcome::EmptyGraphSlice` e `ShortestPathOutcome::PathFound(Vec<String>)` / `ShortestPathOutcome::NoPathConnected`.
- Trait `StructuralGraphRepositoryPort` e `StructuralGraphAnalyzerPort` em `crates/trustkit-core/src/ports.rs`.

### Fora do escopo
- Extracao `tree-sitter`, algoritmos `petgraph` ou persistencia em SQLite.

## Dependencias
- Nenhuma

## Premissas
- Zero dependencias externas em `trustkit-core` (`std` apenas).
- Proibicao estrita de valores `null` na serializacao JSON manual e nas structs.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** uma aresta estrutural instanciada, **quando** classificada com `EdgeConfidence::Extracted`, `Inferred` ou `Ambiguous`, **entao** preserva todos os metadados sem campos nulos.
- [x] **Dado** uma consulta de caminho sem conexao entre dois nos, **quando** representada por `ShortestPathOutcome::NoPathConnected`, **entao** serializa deterministicamente com lista vazia `[]` e variante explicita.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-core/src/knowledge_entities.rs`.

## Rastreabilidade
- Origem: RF-025, RN-001
