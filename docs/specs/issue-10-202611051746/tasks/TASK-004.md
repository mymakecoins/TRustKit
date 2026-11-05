# TASK-004: Integracao de Gates 5 e 6 com Certificacao e Merge

> **Camada:** CLI
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001, TASK-003]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/auto_cmd.rs
- crates/trustkit-cli/src/bootstrap.rs
test_file: crates/trustkit-cli/tests/e2e_auto_lifecycle.rs

---

## Contexto
Apos a aprovacao de todas as ondas no Gate 4, o orquestrador autonomo deve submeter as modificacoes ao Gauntlet Loop V2 (Camada 0 AST-audit e Camada 1 com suíte de testes e criticos), gerar o relatorio `gauntlet-report.md` via `done` e realizar o merge deterministico na branch de destino (`--target`).

## Escopo
- Integrar chamadas programaticas aos handlers de Gauntlet run, Done e Merge.
- Exibir log legivel no terminal ASCII e formatar json estruturado quando `--json` estiver ativo.
