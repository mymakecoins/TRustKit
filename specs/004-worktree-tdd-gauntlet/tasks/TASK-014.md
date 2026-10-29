# TASK-014: Implementar Handlers CLI para Gestao de TDD, Gauntlet, Merge e PR

> **Camada:** API  
> **Estimativa:** G  
> **Status:** Concluida  

---

## Contexto
Após o parser de argumentos do Clap processar as entradas, os handlers da CLI orquestram a execução dos casos de uso, formatam as respostas para o modo humano ou para o modo agente (`--json`) e garantem o mapeamento semântico dos códigos de saída POSIX (`0` a `6`).

## Escopo
Implementar em `crates/trustkit-cli/src/commands/`:
- `task_cmd.rs`: handlers para `task test` e `task complete`.
- `wave_run_cmd.rs`: handler para `wave run`.
- `gauntlet_run_cmd.rs`: handler para `gauntlet run`.
- `doctor_cmd.rs`: handler para `doctor`.
- `done_cmd.rs`: handler para `done` (certificação e salvamento de `gauntlet-report.md`).
- `merge_cmd.rs`: handler para `merge`.
- `pr_cmd.rs`: handler para `pr`.
- `mcp_cmd.rs`: handler para inicializar o loop do servidor MCP.

### Fora do escopo
- Conexão e instanciação de adaptadores de infraestrutura (feita no composition root em `bootstrap.rs`).

## Dependencias
- TASK-013 — Requer os contratos e comandos do Clap.

## Premissas
- Erros de violação de TDD retornam código de saída `3`.
- Falhas do Gauntlet retornam código de saída `5`.
- Circuit Breaker acionado retorna código de saída `6`.

## Criterio de Aceite
- [x] **Dado** uma execução de `trustkit task test TASK-001 --phase green` sem fase RED prévia, **quando** executada, **entao** o handler encerra com `Exit Code: 3` e diagnóstico claro em `stderr` (ou JSON).
- [x] **Dado** `trustkit done` executado após Gauntlet verde, **quando** finaliza, **entao** o arquivo `gauntlet-report.md` é salvo no diretório da especificação e o comentário oficial é exibido.

## Notas Tecnicas
- Diretório: `crates/trustkit-cli/src/commands/`.
- Modularização estrita mantendo cada arquivo abaixo de 250 LOC.

## Rastreabilidade
- Origem: RF-005, RF-006, RF-008, RF-010, RF-011, RF-012
