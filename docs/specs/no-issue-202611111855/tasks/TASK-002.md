# TASK-002: Auditoria de Modulos Rasos e Deletion Test no GraphHealthAuditor

> **Camada:** AST
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/graph_health_auditor.rs
- crates/trustkit-ast/src/module_depth_analyzer.rs
test_file: crates/trustkit-ast/tests/graph_health_auditor_tests.rs

---

## Contexto
O `GraphHealthAuditor` deve detectar modulos rasos (pass-through) que falham no Deletion Test de Ousterhout ao comparar a superficie publica exportada com a densidade de implementacao interna encapsulada, emitindo candidatos de aprofundamento em `shallow_modules`.

## Escopo
- Implementar o calculo de `ModuleDepthMetric` e a deteccao pelo Deletion Test (`ShallowModuleCandidate`) em `trustkit-ast`.
- Integrar a deteccao de `shallow_modules` e `module_depth_metrics` no `GraphHealthAuditor::audit_graph_health`.
- Validar latencia inferior a 15ms e precisao na identificacao de modulos pass-through em `crates/trustkit-ast/tests/graph_health_auditor_tests.rs`.
