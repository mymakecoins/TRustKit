# TASK-007: Implementar Casos de Uso de Travessia (path, explain) e Gerador de GRAPH_REPORT.md

> **Camada:** Casos de Uso de Dominio
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-003, TASK-004, TASK-005, TASK-006]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/structural_graph_use_cases.rs
- crates/trustkit-core/src/use_cases.rs
test_file: crates/trustkit-core/tests/structural_graph_use_cases_tests.rs

---

## Contexto
Com os adaptadores de extracao AST, persistencia SQLite com Shrink-Guard e auditoria de saude prontos, a camada de aplicacao em `trustkit-core` deve orquestrar a construcao incremental do grafo, a resolucao de menor caminho (`path`), a explicacao estrutural de um no (`explain`) e a emissao do relatorio auditavel `.trustkit/wiki/graph/GRAPH_REPORT.md`.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/structural_graph_use_cases.rs`:
- `BuildStructuralGraphUseCase`: coordena a deteccao de arquivos alterados via manifesto `BLAKE3`, extracao de nos/arestas, validacao Shrink-Guard, persistencia no SQLite e geracao deterministica do Markdown `GRAPH_REPORT.md` (contendo resumo do corpus, God Nodes, Surprising Connections, Scores de Coesao brutos e status de saude do grafo, sem emojis).
- `FindShortestGraphPathUseCase`: encontra o menor caminho direcionado ou nao-direcionado entre dois nos (`source_id`, `target_id`), retornando `ShortestPathOutcome::PathFound` ou `ShortestPathOutcome::NoPathConnected`.
- `ExplainGraphNodeUseCase`: retorna a ficha completa de um no (tipo, arquivo de origem, grau de entrada/saida, vizinhos de entrada/saida com nivel de confianca `Extracted`/`Inferred` e notas Wiki vinculadas).

### Fora do escopo
- Parsing de argumentos CLI (`clap`) ou transporte JSON-RPC.

## Dependencias
- TASK-003
- TASK-004
- TASK-005
- TASK-006

## Premissas
- `trustkit-core` permanece 100% `std` only, orquestrando exclusivamente via traits de porta (`&dyn StructuralGraphRepositoryPort`, `&dyn StructuralGraphAnalyzerPort`, `&dyn KnowledgeRepositoryPort`).
- Proibicao absoluta de emojis no `GRAPH_REPORT.md` gerado.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um grafo valido no repositorio, **quando** `BuildStructuralGraphUseCase` e executado, **entao** persiste o grafo e grava `.trustkit/wiki/graph/GRAPH_REPORT.md` contendo as secoes `God Nodes`, `Surprising Connections`, `Module Cohesion Scores` e `Graph Health Audit`.
- [x] **Dado** dois nos conectados por 3 saltos, **quando** `FindShortestGraphPathUseCase` e invocado, **entao** retorna a sequencia exata de 4 nos em `ShortestPathOutcome::PathFound`.

## Notas Tecnicas
- Reexportar os casos de uso em `crates/trustkit-core/src/use_cases.rs`.

## Rastreabilidade
- Origem: RF-031, RN-007
