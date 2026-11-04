# TASK-013: Definir Comandos e Flags CLI Clap para Tasks, Wave Run, Gauntlet, Done, Merge e PR

> **Camada:** API  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O binário `trustkit` precisa expor a interface de linha de comando para os Gates 4, 5 e 6, garantindo suporte tanto ao modo humano (texto limpo, sem emojis) quanto ao modo agente (`--json` estrito, sem valores nulos).

## Escopo
Definir em `crates/trustkit-cli/src/cli_contract.rs`:
- Subcomando `task`:
  - `task test <TASK_ID> --phase <red|green>`: execução de teste com fiscalização da fase TDD.
  - `task complete <TASK_ID>`: conclusão de tarefa atômica.
- Subcomando `wave run`:
  - `wave run --wave <WAVE_NUMBER>`: disparo das tarefas da onda em worktrees.
- Expansão de `gauntlet`:
  - `gauntlet run`: disparo concorrente da Camada 0 e Camada 1.
- Subcomando `doctor`:
  - Diagnóstico e sumário de falhas quando o disjuntor de segurança é acionado.
- Subcomando `done`:
  - Certificação final do ciclo, compilação do relatório e geração do comentário oficial da issue.
- Subcomando `merge`:
  - `merge --target <BRANCH>`: execução do merge determinístico na branch alvo.
- Subcomando `pr`:
  - Abertura de Pull Request sob demanda para `develop`.
- Subcomando `mcp serve`:
  - Início do servidor MCP sobre `stdio`.

### Fora do escopo
- Lógica de execução dos handlers (coberta por TASK-014).

## Dependencias
- TASK-003, TASK-004, TASK-005, TASK-006 — Requer os casos de uso de domínio.

## Premissas
- Todos os subcomandos devem herdar o flag global `--json`.
- A saída em modo agente deve ser minificada e sem campos `null`.

## Criterio de Aceite
- [x] **Dado** os comandos `task test TASK-001 --phase red`, `wave run --wave 1`, `gauntlet run`, `done`, `merge --target develop` e `pr`, **quando** o parser Clap é invocado, **entao** todos os argumentos e opções são parseados com sucesso.
- [x] **Dado** `--help` invocado em qualquer subcomando, **quando** executado, **entao** a saída não contém nenhum emoji ou caractere proibido.

## Notas Tecnicas
- Arquivo: `crates/trustkit-cli/src/cli_contract.rs`.
- Testes unitários de parsing de argumentos.

## Rastreabilidade
- Origem: RF-005, RF-006, RF-008, RF-010, RF-011, RF-012
