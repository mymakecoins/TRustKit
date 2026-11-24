# TASK-002: Cache de Compilacao em Worktrees Agrupamento de Comandos e Cache AST

layer: ORCH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-orch/tests/tool_cache_batch_executor_tests.rs
expected_files:
  - crates/trustkit-orch/src/tool_cache_batch_executor.rs
  - crates/trustkit-orch/src/worktree_provider.rs
  - crates/trustkit-orch/src/builder_agent_dispatcher.rs
  - crates/trustkit-orch/src/async_supervisor.rs
  - crates/trustkit-orch/tests/tool_cache_batch_executor_tests.rs

## Contexto
Cache de Compilacao em Worktrees Agrupamento de Comandos e Cache AST atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** multiplas tarefas atomicas executadas em worktrees isolados e ondas consecutivas em `trustkit-orch`
- **Quando** `WorktreeIsolationProvider`, `BuilderAgentDispatcher`, `AsyncExecutionSupervisor` e `AstWaveAnalysisCache` forem acionados com politicas de cache e agrupamento habilitadas
- **Entao** propaga `RUSTC_WRAPPER=sccache` ou `target-dir` compartilhado seguro entre worktrees, agrupa verificacoes e testes compativeis por crate ou onda preservando a rastreabilidade por tarefa e reutiliza analises AST e de acoplamento estrutural para arquivos cujo digest BLAKE3 nao mudou.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-tool-cache-batch-orch.json
