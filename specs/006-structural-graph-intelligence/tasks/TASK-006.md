# TASK-006: Implementar Auditor de Saude do Grafo (Graph Health Check) e Detector de God Nodes

> **Camada:** Analise Estatica AST & Governanca Camada 0
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-002]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/graph_health_auditor.rs
test_file: crates/trustkit-ast/tests/graph_health_auditor_tests.rs

---

## Contexto
Inspirado no passo de integridade nao-destrutiva do Graphify (`Step 4.5 - Graph health check`) e na analise de centralidade (`god_nodes` e `surprising_connections`), o TRustKit precisa auditar mecanicamente arestas quebradas em WikiLinks/Specs, auto-referencias, colapsos de arestas duplicadas e violacoes de fronteira da Arquitetura Hexagonal.

## Escopo
Implementar `GraphHealthAuditor` em `crates/trustkit-ast/src/graph_health_auditor.rs`:
- Deteccao de `dangling_edges`: arestas cujos nos de origem ou destino nao existem no conjunto de nos conhecidos (ex: `[[wikilink]]` apontando para documento inexistente ou arquivo removido).
- Deteccao de `self_loop_edges` (no apontando para si mesmo) e `collapsed_edges_count` (arestas paralelas redundantes entre o mesmo par origem-destino).
- Calculo de `god_nodes(top_n)`: ordenacao deterministica dos nos com maior grau total (`in_degree + out_degree`).
- Deteccao de `surprising_connections`: arestas que cruzam camadas arquiteturais em direcao proibida (ex: crate `trustkit-core` importando qualquer crate externo ou `trustkit-db` importando `trustkit-cli`).
- Calculo de score de coesao por modulo/crate (razao entre arestas internas e arestas totais do modulo, exibindo o numero bruto conforme Honesty Rules).

### Fora do escopo
- Persistencia em banco ou interface CLI.

## Dependencias
- TASK-002

## Premissas
- Execucao em memoria em `< 3ms` para grafos de ate 5.000 nos.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um grafo contendo um `WikiLink` para `adrs/inexistente` e um self-loop `node_a -> node_a`, **quando** `GraphHealthAuditor::audit` e executado, **entao** o `GraphHealthReport` lista exatamente a aresta orfa e o self-loop com `GraphHealthVerdict::DegradedWarning`.
- [x] **Dado** uma aresta onde um modulo de `trustkit-core` importa `trustkit-db`, **quando** auditada, **entao** e reportada em `surprising_connections` como violacao de fronteira hexagonal.

## Notas Tecnicas
- Implementar o trait `StructuralGraphAnalyzerPort` definido em `trustkit-core`.

## Rastreabilidade
- Origem: RF-030, RN-006
