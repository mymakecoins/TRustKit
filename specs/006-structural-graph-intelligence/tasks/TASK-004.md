# TASK-004: Implementar Context Slicer Multi-Hop (BFS e DFS) Guiado por Grafo

> **Camada:** Analise Estatica AST & Topologia
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-002]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/graph_context_slicer.rs
- crates/trustkit-ast/src/context_slicer.rs
test_file: crates/trustkit-ast/tests/graph_context_slicer_tests.rs

---

## Contexto
O `AstContextSlicer` atual fatia apenas um unico arquivo isolado. Ao incorpora-lo ao grafo estrutural em `petgraph`, o TRustKit pode navegar automaticamente pelas arestas de importacao em largura (`BFS`, contexto amplo de vizinhanca) ou profundidade (`DFS`, cadeia de tipos/chamadas) até preencher o orcamento de linhas/tokens do Builder.

## Escopo
Implementar `GraphContextSlicer` em `crates/trustkit-ast/src/graph_context_slicer.rs` e integrar com `context_slicer.rs`:
- Suporte aos modos de travessia `TraversalMode::Bfs` e `TraversalMode::Dfs` com limite configuravel de saltos (`max_hops`).
- Priorizacao deterministica de arestas `EdgeConfidence::Extracted` sobre `EdgeConfidence::Inferred`.
- Coleta de assinaturas publicas (`pub struct`, `pub enum`, `pub trait`, `pub fn`) do arquivo raiz e dos arquivos alcancados na travessia ate o limite `max_line_budget`.
- Retorno de `GraphTraversalOutcome::VisitedSlice` contendo o codigo fatiado, lista ordenada de nos visitados, arestas atravessadas e linhas consumidas.

### Fora do escopo
- Alteracoes no banco SQLite ou subcomandos de terminal.

## Dependencias
- TASK-002

## Premissas
- Preservar retrocompatibilidade total com chamadas de arquivo unico do `AstContextSlicer`.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um arquivo `service.rs` que importa tipos de `ports.rs` e `entities.rs`, **quando** o `GraphContextSlicer` executa em modo `BFS` com `max_hops = 1` e `budget = 80`, **entao** o resultado inclui as assinaturas publicas de `service.rs` seguidas das assinaturas de `ports.rs` e `entities.rs` sem exceder 80 linhas.
- [x] **Dado** a escolha entre modo `BFS` e `DFS` em um grafo com 3 niveis de profundidade, **quando** executado com orcamento restrito, **entao** a ordem dos arquivos coletados reflete fielmente a busca em largura ou em profundidade.

## Notas Tecnicas
- Utilizar `petgraph::visit::{Bfs, Dfs}` sobre o `DiGraph` construido a partir das arestas estruturais.

## Rastreabilidade
- Origem: RF-028, RN-004
