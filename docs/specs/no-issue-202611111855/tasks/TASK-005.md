# TASK-005: Exposicao CLI, MCP e Sincronizacao Documental de Deep Modules

> **Camada:** CLI
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-002, TASK-003, TASK-004]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/cli_spec_args.rs
- crates/trustkit-cli/src/commands/spec_prompt_cmd.rs
- crates/trustkit-cli/src/commands/graph_cmd.rs
- crates/trustkit-orch/src/mcp_graph_dispatch.rs
- crates/trustkit-orch/src/mcp_spec_dispatch.rs
- README.md
- docs/manuals/user-guide.md
- docs/manuals/user-guide.html
test_file: crates/trustkit-cli/tests/mcp_graph_tools_tests.rs

---

## Contexto
Os diagnosticos de profundidade de modulo, candidatos do Deletion Test, classificacao de costuras e briefings Design It Twice devem estar disponiveis no CLI (`trustkit graph health` e `trustkit spec prompt --design-it-twice`), no servidor MCP (`trustkit_graph_health` e `trustkit_spec_context`) em menos de 10ms e documentados nos manuais oficiais.

## Escopo
- Expor `--design-it-twice` em `trustkit spec prompt` e incluir `shallow_modules`, `module_depth_metrics` e `seam_classifications` em `trustkit graph health`.
- Expor as novas metricas e briefings nas ferramentas MCP `trustkit_graph_health` e `trustkit_spec_context`.
- Atualizar a documentacao oficial (`README.md`, `README.en.md`, `docs/PRD_TRustKit.md`, `docs/ADR_001_TRustKit_Architecture.md`, `docs/manuals/user-guide.md`, `docs/manuals/user-guide.html`, `docs/manual/MANUAL_DO_USUARIO.md`, `docs/manual/manual.html`) com o vocabulario de Deep Modules, Deletion Test, Seams e Design It Twice.
- Escrever testes de integracao em `crates/trustkit-cli/tests/mcp_graph_tools_tests.rs`.
