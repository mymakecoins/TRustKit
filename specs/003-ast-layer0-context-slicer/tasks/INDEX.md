# Indice de Tarefas — 003-ast-layer0-context-slicer

**Origem:** `docs/PRD_TRustKit.md` (RF-007, RF-006, RN-001 a RN-016)  
**Gerado em:** Outubro/2026  
**Total de tarefas:** 16  

---

## Tarefas

| ID | Titulo | Camada | Depende de | Origem |
|----|--------|--------|------------|--------|
| TASK-001 | Implementar Entidades de Dominio da Camada 0 de AST e Slicing | Domínio | Nenhuma | RF-007, RN-001 |
| TASK-002 | Definir Portas Abstratas de Auditoria AST e Context Slicer | Domínio | TASK-001 | RF-007, RN-002 |
| TASK-003 | Configurar Gramaticas e Dependencias Tree-Sitter no Workspace | Infra/Setup | Nenhuma | RF-007, RN-003 |
| TASK-004 | Implementar Scanner Lexico de Invariantes e Proibicoes | Integração | TASK-001 | RF-007, RN-004, RN-007, RN-013 |
| TASK-005 | Implementar Parser Tree-Sitter de Assercoes para Rust | Integração | TASK-003 | RF-007, RN-005 |
| TASK-006 | Implementar Parser Tree-Sitter de Assercoes para TypeScript | Integração | TASK-003 | RF-007, RN-006 |
| TASK-007 | Implementar Auditor Anti-Cheat de Integridade de Testes | Integração | TASK-002, TASK-005, TASK-006 | RF-007, RN-007 |
| TASK-008 | Implementar Fatiador de Contexto AST (AstContextSlicer) | Integração | TASK-002, TASK-003 | RF-006, RF-007, RN-008 |
| TASK-009 | Configurar Reexportacoes e Facade na Crate trustkit-ast | Integração | TASK-004, TASK-007, TASK-008 | RF-007, RN-009 |
| TASK-010 | Implementar Persistencia de Auditorias AST e Evidencias no SQLite | Banco | TASK-001 | RF-007, RN-010 |
| TASK-011 | Implementar Caso de Uso de Auditoria AST da Camada 0 | Domínio | TASK-002, TASK-010 | RF-007, RN-011 |
| TASK-012 | Implementar Caso de Uso de Fatiamento de Contexto AST | Domínio | TASK-002 | RF-006, RF-007, RN-012 |
| TASK-013 | Definir Comandos CLI para Auditoria AST e Context Slicing | API | TASK-011, TASK-012 | RF-007, RN-013 |
| TASK-014 | Implementar Handlers CLI para Execucao de AST e Slicing | API | TASK-013 | RF-007, RN-014 |
| TASK-015 | Integrar Adaptadores no Composition Root em bootstrap.rs | Infra/Setup | TASK-009, TASK-014 | RF-007, RN-015 |
| TASK-016 | Implementar Suite de Testes E2E para Camada 0 de AST | Infra/Setup | TASK-015 | RF-007, RN-016 |

---

## Ordem de Execucao (Ondas)

Tarefas na mesma onda podem ser executadas em paralelo; cada onda depende da anterior.

### Onda 1 — Fundacao & Setup
- TASK-001 — Implementar Entidades de Dominio da Camada 0 de AST e Slicing
- TASK-003 — Configurar Gramaticas e Dependencias Tree-Sitter no Workspace

### Onda 2 — Analise Sintatica, Scanners e Persistencia
- TASK-002 — Definir Portas Abstratas de Auditoria AST e Context Slicer
- TASK-004 — Implementar Scanner Lexico de Invariantes e Proibicoes
- TASK-005 — Implementar Parser Tree-Sitter de Assercoes para Rust
- TASK-006 — Implementar Parser Tree-Sitter de Assercoes para TypeScript
- TASK-010 — Implementar Persistencia de Auditorias AST e Evidencias no SQLite

### Onda 3 — Motores Centrais de AST e Slicing
- TASK-007 — Implementar Auditor Anti-Cheat de Integridade de Testes
- TASK-008 — Implementar Fatiador de Contexto AST (AstContextSlicer)
- TASK-012 — Implementar Caso de Uso de Fatiamento de Contexto AST

### Onda 4 — Casos de Uso e Integracao de Modulos
- TASK-009 — Configurar Reexportacoes e Facade na Crate trustkit-ast
- TASK-011 — Implementar Caso de Uso de Auditoria AST da Camada 0

### Onda 5 — Contratos de Interface de Linha de Comando (CLI)
- TASK-013 — Definir Comandos CLI para Auditoria AST e Context Slicing

### Onda 6 — Handlers de Execucao
- TASK-014 — Implementar Handlers CLI para Execucao de AST e Slicing

### Onda 7 — Composition Root
- TASK-015 — Integrar Adaptadores no Composition Root em bootstrap.rs

### Onda 8 — Homologacao End-to-End
- TASK-016 — Implementar Suite de Testes E2E para Camada 0 de AST

---

## Mapa de Rastreabilidade

| Requisito de Origem | Tarefas que o Cobrem |
|---------------------|----------------------|
| RF-006 (Context Slicing) | TASK-008, TASK-012 |
| RF-007 (Auditoria AST Camada 0) | TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-009, TASK-010, TASK-011, TASK-013, TASK-014, TASK-015, TASK-016 |
| RN-004 (Anti-Cheat Imutavel) | TASK-005, TASK-006, TASK-007, TASK-016 |
| RN-007 (Proibicao de Emojis) | TASK-004, TASK-014, TASK-016 |
| RN-013 (Zero Atribuicao a IA) | TASK-004, TASK-016 |
