# Indice de Tarefas — 004-worktree-tdd-gauntlet

**Origem:** `docs/PRD_TRustKit.md` (RF-005, RF-006, RF-008, RF-009, RF-010, RF-011, RF-012, RN-001 a RN-016)  
**Gerado em:** Outubro/2026  
**Total de tarefas:** 17  

---

## Tarefas

| ID | Titulo | Camada | Depende de | Origem |
|----|--------|--------|------------|--------|
| TASK-001 | Implementar Entidades e Estados de Dominio para Worktrees, TDD e Gauntlet | Dominio | Nenhuma | RF-005, RF-006, RF-008, RN-002, RN-005 |
| TASK-002 | Definir Portas Abstratas para Worktree, Orquestracao, Evidencias Visuais e Certificacao | Dominio | TASK-001 | RF-005, RF-006, RF-008, RF-010, RN-002 |
| TASK-003 | Implementar Casos de Uso do Ciclo TDD (record_tdd_phase e complete_task) | Dominio | TASK-002 | RF-005, RN-002, RN-004 |
| TASK-004 | Implementar Caso de Uso de Execucao de Onda e Isolamento de Worktree | Dominio | TASK-002 | RF-006, RN-006, RN-007, RN-013 |
| TASK-005 | Implementar Casos de Uso do Gauntlet Camada 1 e Circuit Breaker | Dominio | TASK-002 | RF-008, RF-009, RN-003, RN-005 |
| TASK-006 | Implementar Casos de Uso de Certificacao, Merge e Pull Request | Dominio | TASK-002 | RF-010, RF-011, RF-012, RN-010, RN-011, RN-012 |
| TASK-007 | Implementar Persistencia SQLite WAL para Fases TDD, Worktrees e Certificacao | Banco | TASK-001 | RF-005, RF-008, RF-010, RN-002 |
| TASK-008 | Implementar Provedor de Isolamento de Git Worktrees (WorktreeIsolationProvider) | Integracao | TASK-002 | RF-006, RN-006 |
| TASK-009 | Implementar Despachante Assincrono Concorrente e Capturador de Evidencias | Integracao | TASK-002 | RF-006, RF-008, RN-002 |
| TASK-010 | Implementar Guardiao de Evidencias Visuais e Restaurador Alheio (VisualEvidenceGuard) | Integracao | TASK-002 | RF-008, RN-008 |
| TASK-011 | Implementar Servidor MCP stdio para Ferramentas de SDD do Agente Hospedeiro | Integracao | TASK-002 | RF-006, RF-008, Secao 5.1.4 |
| TASK-012 | Implementar Gateway de Pull Request e Movimentacao de Card Await Staging / In review | Integracao | TASK-002 | RF-011, RF-012, RN-009, RN-012, RN-014 |
| TASK-013 | Definir Comandos e Flags CLI Clap para Tasks, Wave Run, Gauntlet, Done, Merge e PR | API | TASK-003, TASK-004, TASK-005, TASK-006 | RF-005, RF-006, RF-008, RF-010, RF-011, RF-012 |
| TASK-014 | Implementar Handlers CLI para Gestao de TDD, Gauntlet, Merge e PR | API | TASK-013 | RF-005, RF-006, RF-008, RF-010, RF-011, RF-012 |
| TASK-015 | Integrar Adaptadores no Composition Root em bootstrap.rs | Infra/Setup | TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-014 | RF-005 a RF-012 |
| TASK-016 | Redigir a Thin Skill Declarativa SKILL.md v4 Integrada ao Binario trustkit | Infra/Setup | TASK-015 | Secao 3.3, Secao 4.2 |
| TASK-017 | Implementar Suite de Testes de Integracao e E2E para Gates 4, 5 e 6 | Infra/Setup | TASK-015 | RF-005, RF-006, RF-008, RF-010, RF-011, RF-012 |

---

## Ordem de Execucao (Ondas)

Tarefas na mesma onda podem ser executadas em paralelo; cada onda depende da anterior.

### Onda 1 — Fundacao de Dominio
- TASK-001 — Implementar Entidades e Estados de Dominio para Worktrees, TDD e Gauntlet

### Onda 2 — Portas Abstratas e Persistencia
- TASK-002 — Definir Portas Abstratas para Worktree, Orquestracao, Evidencias Visuais e Certificacao
- TASK-007 — Implementar Persistencia SQLite WAL para Fases TDD, Worktrees e Certificacao

### Onda 3 — Casos de Uso Core
- TASK-003 — Implementar Casos de Uso do Ciclo TDD (record_tdd_phase e complete_task)
- TASK-004 — Implementar Caso de Uso de Execucao de Onda e Isolamento de Worktree
- TASK-005 — Implementar Casos de Uso do Gauntlet Camada 1 e Circuit Breaker
- TASK-006 — Implementar Casos de Uso de Certificacao, Merge e Pull Request

### Onda 4 — Adaptadores de Infraestrutura e Integracao
- TASK-008 — Implementar Provedor de Isolamento de Git Worktrees (WorktreeIsolationProvider)
- TASK-009 — Implementar Despachante Assincrono Concorrente e Capturador de Evidencias
- TASK-010 — Implementar Guardiao de Evidencias Visuais e Restaurador Alheio (VisualEvidenceGuard)
- TASK-011 — Implementar Servidor MCP stdio para Ferramentas de SDD do Agente Hospedeiro
- TASK-012 — Implementar Gateway de Pull Request e Movimentacao de Card Await Staging / In review

### Onda 5 — Contratos de Interface de Linha de Comando (CLI)
- TASK-013 — Definir Comandos e Flags CLI Clap para Tasks, Wave Run, Gauntlet, Done, Merge e PR

### Onda 6 — Handlers de Execucao CLI
- TASK-014 — Implementar Handlers CLI para Gestao de TDD, Gauntlet, Merge e PR

### Onda 7 — Composition Root
- TASK-015 — Integrar Adaptadores no Composition Root em bootstrap.rs

### Onda 8 — Thin Skill e Homologacao End-to-End
- TASK-016 — Redigir a Thin Skill Declarativa SKILL.md v4 Integrada ao Binario trustkit
- TASK-017 — Implementar Suite de Testes de Integracao e E2E para Gates 4, 5 e 6

---

## Mapa de Rastreabilidade

| Requisito de Origem | Tarefas que o Cobrem |
|---------------------|----------------------|
| RF-005 (Fiscalizacao TDD) | TASK-001, TASK-002, TASK-003, TASK-007, TASK-013, TASK-014, TASK-015, TASK-017 |
| RF-006 (Isolamento Worktrees) | TASK-001, TASK-002, TASK-004, TASK-008, TASK-009, TASK-013, TASK-014, TASK-015, TASK-017 |
| RF-008 (Swarm Criticos & E2E) | TASK-001, TASK-002, TASK-005, TASK-009, TASK-010, TASK-013, TASK-014, TASK-015, TASK-017 |
| RF-009 (Circuit Breaker) | TASK-001, TASK-005, TASK-013, TASK-014, TASK-017 |
| RF-010 (Certificacao & Relatorio) | TASK-002, TASK-006, TASK-007, TASK-013, TASK-014, TASK-015, TASK-017 |
| RF-011 (Gate de Merge) | TASK-006, TASK-012, TASK-013, TASK-014, TASK-017 |
| RF-012 (PR Sob Demanda) | TASK-006, TASK-012, TASK-013, TASK-014, TASK-017 |
| RN-002 (Evidencia sobre Assercao) | TASK-001, TASK-002, TASK-003, TASK-007, TASK-009 |
| RN-003 (Aprovacao Unanime) | TASK-005, TASK-006, TASK-014 |
| RN-005 (Circuit Breaker 5 Loops) | TASK-001, TASK-005, TASK-014 |
| RN-006 (GitFlow & Arvore) | TASK-004, TASK-008, TASK-014 |
| RN-008 (Evidencias Visuais) | TASK-010, TASK-014, TASK-017 |
| RN-010 (Comentario Encerramento) | TASK-006, TASK-014 |
| RN-012 (PR Sob Demanda) | TASK-006, TASK-012, TASK-014 |
| RN-014 (Identidade GitHub) | TASK-012, TASK-014 |
