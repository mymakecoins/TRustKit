# Indice de Tarefas — 005-project-knowledge-memory

**Origem:** `specs/005-project-knowledge-memory/spec.md` (RF-017 a RF-024, RN-001 a RN-008)  
**Gerado em:** Outubro/2026  
**Total de tarefas:** 8  

---

## Tarefas

| ID | Titulo | Camada | Depende de | Origem |
|----|--------|--------|------------|--------|
| TASK-001 | Implementar Entidades de Dominio e Portas de Memoria Perene | CORE | Nenhuma | RF-017, RN-001 |
| TASK-002 | Implementar Extrator Lexico de Frontmatter YAML e Wikilinks | AST | TASK-001 | RF-018, RN-002 |
| TASK-003 | Implementar Tabelas FTS5, Grafo de Adjacencia e Busca BM25 no SQLite | DB | TASK-001 | RF-019, RN-003 |
| TASK-004 | Implementar Repositorio de Arquivos Wiki e Reconstrucao de Indice | DB | TASK-002, TASK-003 | RF-017, RF-020, RN-004 |
| TASK-005 | Implementar Caso de Uso de Ingestao Automatica no Gate 2 (Spec Lock) | CORE | TASK-004 | RF-021, RN-005 |
| TASK-006 | Implementar Caso de Uso de Ingestao Automatica no Gate 4 (Task Complete) | CORE | TASK-004 | RF-022, RN-006 |
| TASK-007 | Implementar Caso de Uso de Ingestao de Incidente no Gate 5 (Gauntlet e Doctor) | CORE | TASK-004 | RF-023, RN-007 |
| TASK-008 | Expor Comandos CLI trustkit memory e Tools Nativas no Servidor MCP | CLI | TASK-005, TASK-006, TASK-007 | RF-024, RN-008 |

---

## Ordem de Execucao (Ondas DAG)

Tarefas na mesma onda possuem conjuntos disjuntos de arquivos esperados e podem ser executadas simultaneamente em Git Worktrees isoladas.

### Onda 1 — Fundacao de Dominio
- TASK-001 — Implementar Entidades de Dominio e Portas de Memoria Perene

### Onda 2 — Motor Lexico e Indexador SQLite FTS5
- TASK-002 — Implementar Extrator Lexico de Frontmatter YAML e Wikilinks
- TASK-003 — Implementar Tabelas FTS5, Grafo de Adjacencia e Busca BM25 no SQLite

### Onda 3 — Sincronizador de Arquivos Wiki e Reconstrucao Deterministica
- TASK-004 — Implementar Repositorio de Arquivos Wiki e Reconstrucao de Indice

### Onda 4 — Gatilhos de Ingestao Automatica nos Gates do SDD
- TASK-005 — Implementar Caso de Uso de Ingestao Automatica no Gate 2 (Spec Lock)
- TASK-006 — Implementar Caso de Uso de Ingestao Automatica no Gate 4 (Task Complete)
- TASK-007 — Implementar Caso de Uso de Ingestao de Incidente no Gate 5 (Gauntlet e Doctor)

### Onda 5 — Interface CLI e Servidor MCP Nativo
- TASK-008 — Expor Comandos CLI trustkit memory e Tools Nativas no Servidor MCP

---

## Mapa de Rastreabilidade

| Requisito de Origem | Tarefas que o Cobrem |
|---------------------|----------------------|
| RF-017 (Escrita e Leitura Atomica Markdown) | TASK-001, TASK-004 |
| RF-018 (Extracao de Wikilinks e Grafo) | TASK-002 |
| RF-019 (Busca FTS5 com Ranqueamento BM25) | TASK-003 |
| RF-020 (Reconstrucao Deterministica do Indice) | TASK-004 |
| RF-021 (Ingestao Automatica Gate 2 - Spec Lock) | TASK-005 |
| RF-022 (Ingestao Automatica Gate 4 - Task Complete) | TASK-006 |
| RF-023 (Ingestao Automatica Gate 5 - Incidentes) | TASK-007 |
| RF-024 (CLI e Servidor MCP Nativo) | TASK-008 |
