# Indice de Tarefas — 006-structural-graph-intelligence

**Origem:** `specs/006-structural-graph-intelligence/spec.md` (RF-025 a RF-032, RN-001 a RN-008)  
**Gerado em:** Outubro/2026  
**Total de tarefas:** 8  

---

## Tarefas

| ID | Titulo | Camada | Depende de | Origem |
|----|--------|--------|------------|--------|
| TASK-001 | Implementar Entidades de Dominio do Grafo Estrutural e Contratos de Porta | CORE | Nenhuma | RF-025, RN-001 |
| TASK-002 | Implementar Extrator Sintatico AST de Nos e Arestas de Codigo | AST | TASK-001 | RF-026, RN-002 |
| TASK-003 | Implementar Persistencia Incremental com Manifesto BLAKE3 e Shrink-Guard no SQLite | DB | TASK-001 | RF-027, RN-003 |
| TASK-004 | Implementar Context Slicer Multi-Hop (BFS e DFS) Guiado por Grafo | AST | TASK-002 | RF-028, RN-004 |
| TASK-005 | Implementar Analisador de Acoplamento AST e God Nodes no WaveExecutionPlanner | AST | TASK-002 | RF-029, RN-005 |
| TASK-006 | Implementar Auditor de Saude do Grafo (Graph Health Check) e Detector de God Nodes | AST | TASK-002 | RF-030, RN-006 |
| TASK-007 | Implementar Casos de Uso de Travessia (path, explain) e Gerador de GRAPH_REPORT.md | CORE | TASK-003, TASK-004, TASK-005, TASK-006 | RF-031, RN-007 |
| TASK-008 | Expor Subcomandos CLI trustkit graph e Tools Nativas no Servidor MCP | CLI | TASK-007 | RF-032, RN-008 |

---

## Ordem de Execucao (Ondas DAG)

Tarefas na mesma onda possuem conjuntos disjuntos de arquivos esperados e podem ser executadas simultaneamente em Git Worktrees isoladas.

### Onda 1 — Fundacao de Dominio e Contratos
- TASK-001 — Implementar Entidades de Dominio do Grafo Estrutural e Contratos de Porta

### Onda 2 — Extrator AST e Repositorio SQLite Incremental com Shrink-Guard
- TASK-002 — Implementar Extrator Sintatico AST de Nos e Arestas de Codigo
- TASK-003 — Implementar Persistencia Incremental com Manifesto BLAKE3 e Shrink-Guard no SQLite

### Onda 3 — Motores de Topologia: Multi-Hop Slicer, Acoplamento de Ondas e Graph Health Check
- TASK-004 — Implementar Context Slicer Multi-Hop (BFS e DFS) Guiado por Grafo
- TASK-005 — Implementar Analisador de Acoplamento AST e God Nodes no WaveExecutionPlanner
- TASK-006 — Implementar Auditor de Saude do Grafo (Graph Health Check) e Detector de God Nodes

### Onda 4 — Orquestracao de Casos de Uso e Emissao do GRAPH_REPORT.md
- TASK-007 — Implementar Casos de Uso de Travessia (path, explain) e Gerador de GRAPH_REPORT.md

### Onda 5 — Interface CLI e Servidor MCP Nativo
- TASK-008 — Expor Subcomandos CLI trustkit graph e Tools Nativas no Servidor MCP

---

## Mapa de Rastreabilidade

| Requisito de Origem | Tarefas que o Cobrem |
|---------------------|----------------------|
| RF-025 (Entidades e Classificacao Honesta de Arestas) | TASK-001 |
| RF-026 (Extracao Sintatica AST Tree-Sitter) | TASK-002 |
| RF-027 (Manifesto Incremental BLAKE3 e Shrink-Guard) | TASK-003 |
| RF-028 (Context Slicing Multi-Hop BFS e DFS) | TASK-004 |
| RF-029 (Deteccao de Acoplamento AST e God Nodes nas Ondas) | TASK-005 |
| RF-030 (Graph Health Check e Auditoria de Fronteiras) | TASK-006 |
| RF-031 (Consultas path/explain e Gerador de GRAPH_REPORT.md) | TASK-007 |
| RF-032 (Subcomando CLI trustkit graph e Tools MCP) | TASK-008 |
