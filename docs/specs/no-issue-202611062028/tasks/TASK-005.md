# TASK-005: Comandos CLI trustkit milestones e Integracao com trustkit init --milestone

> **Camada:** CLI
> **Estimativa:** L
> **Status:** Concluido

depends_on:
- TASK-003
- TASK-004
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/cli_contract.rs
- crates/trustkit-cli/src/commands/milestones_cmd.rs
- crates/trustkit-cli/src/commands/init_cmd.rs
- crates/trustkit-cli/src/commands/mod.rs
- crates/trustkit-cli/src/presenter.rs
test_file: crates/trustkit-cli/tests/cli_milestone_and_init_tests.rs

---

## Contexto
O desenvolvedor humano ou agente precisa invocar o planejamento de marcos via CLI com o novo comando `trustkit milestones plan --prd <PRD> --adr <ADR>`, listar os marcos existentes com `trustkit milestones list`, inspecionar detalhes com `trustkit milestones inspect <ID>`, e inicializar ciclos SDD pontuais via `trustkit init --milestone <CAMINHO>`.

## Escopo
- Adicionar variantes no `Cli` e argumentos em `cli_contract.rs` para o comando `milestones` e a flag `--milestone` no `InitArgs`.
- Implementar `crates/trustkit-cli/src/commands/milestones_cmd.rs` gerenciando as sub-acoes `plan`, `list` e `inspect`.
- Atualizar `init_cmd.rs` e `initialize_cycle.rs` para aceitar `milestone_path` opcional, associando o ciclo ao milestone planejado.
- Implementar formatacao de saida texto e JSON deterministicamente sem null no `presenter.rs`.
- Escrever testes em `crates/trustkit-cli/tests/cli_milestone_and_init_tests.rs`.
