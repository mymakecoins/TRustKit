# TASK-007: Implementar Contrato CLI trustkit wave run com Suporte a Runtime

> **Camada:** Interface / CLI
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-006]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/wave_run_cmd.rs
- crates/trustkit-cli/src/cli_contract.rs
test_file: crates/trustkit-cli/tests/cli_wave_run_runtime_tests.rs

---

## Contexto
O comando CLI `trustkit wave run` e o ponto de entrada operacional do Gate 4. Ele precisa ser estendido com argumentos declarativos para selecao de runtime (`--runtime <auto|herdr|native>`) e comando do agente construtor (`--agent-cmd "<cmd>"`), mantendo saida estruturada JSON livre de `null` e codigos de saida semanticos POSIX.

## Escopo
Implementar em `crates/trustkit-cli/src/commands/wave_run_cmd.rs` e `crates/trustkit-cli/src/cli_contract.rs`:
- Atualizar a estrutura Clap `WaveRunArgs`:
  - `runtime: Option<String>` (padrao: `"auto"`, aceita `"auto"`, `"herdr"`, `"native"`).
  - `agent_cmd: Option<String>` (padrao: configuracao de `trustkit.toml` ou string vazia para fallback).
- Integrar com `SupervisorRuntimeSelector` e o caso de uso `run_interactive_wave`.
- Atualizar a resposta JSON `JsonWaveRunResponse` para incluir:
  - `runtime_used: String`.
  - `tasks_dispatched: Vec<JsonInteractiveTaskSummary>`.
  - Ausencia absoluta de campos `null`.
- Tratar codigos de saida semanticos:
  - `0`: CleanPass (todas as tarefas concluidas e merged).
  - `1`: InfrastructureBoundaryFault (falha de socket, git ou I/O).
  - `2`: SpecOrTraceabilityIncomplete (especificacao nao congelada).

### Fora do escopo
- Alteracao nos demais subcomandos da CLI.

## Dependencias
- TASK-006

## Premissas
- Compatibilidade retroativa: se nenhuma flag nova for informada, o comando se comporta de forma identica ao fluxo estavel anterior.
- Zero uso de emojis na saida de terminal ou JSON.

## Criterio de Aceite
- [ ] **Dado** a invocacao `trustkit wave run --wave 1 --runtime auto --json`, **quando** executado com sucesso, **entao** retorna JSON com `verdict: "PASS"` e o runtime utilizado sem campos `null`.
- [ ] **Dado** uma opcao de runtime invalida como `--runtime invalid`, **quando** executado, **entao** o Clap rejeita com codigo de erro semantico antes de mutar o estado.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-cli/src/commands/wave_run_cmd.rs`.

## Rastreabilidade
- Origem: RF-059, RN-007
