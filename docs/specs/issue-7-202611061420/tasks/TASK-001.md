# TASK-001: Corrigir Exit Code e Stream de Saida para Help e Version no Main CLI

> **Camada:** CLI
> **Estimativa:** P
> **Status:** Pendente

depends_on: []
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/main.rs
test_file: crates/trustkit-cli/tests/cli_help_exit_code_tests.rs

---

## Contexto
Atualmente, qualquer erro retornado por `Cli::try_parse()` e capturado em `main.rs` e forca `ExitCode::from(1)` alem de direcionar a saida exclusivamente para `eprintln!`. No clap v4, comandos informativos como `--help`, `-h`, `--version` e `-V` sao retornados como `Err(clap::Error)` com exit_code igual a 0 e `use_stderr()` falso.

## Escopo
- Modificar o casamento de padrao em `crates/trustkit-cli/src/main.rs`.
- Se `clap_err.use_stderr()` for falso (ou exit code for 0), direcionar a impressao para stdout (`println!("{clap_err}")`) e retornar `ExitCode::from(0)`.
- Se `clap_err.use_stderr()` for verdadeiro, manter a saida em stderr e retornar `ExitCode::from(code as u8)`.
- Garantir comportamento compativel quando `--json` ou `TRUSTKIT_AGENT_MODE` estiver ativo.
