# TASK-003: Implementar Casos de Uso do Ciclo TDD (record_tdd_phase e complete_task)

> **Camada:** Dominio  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O framework MMC Gauntlet SDD V2 exige fiscalizacao deterministica do ciclo TDD no Gate 4 (`RN-002` e `RN-004`). O agente builder nao pode declarar que o teste passou sem antes provar que ele existiu e falhou (`RED`), impedindo que testes vazios ou dilution passem despercebidos.

## Escopo
Implementar os servicos de aplicacao em `crates/trustkit-core/src/use_cases/`:
- `record_tdd_phase.rs`:
  - Se fase for `Red`: executa o teste via porta de orquestracao. Exige comprovacao de falha (`exit_code != 0`). Grava evidencia no repositorio e atualiza estado da tarefa para `RedVerified`.
  - Se fase for `Green`: verifica se existe `RedVerified` no banco. Se nao houver, rejeita com `TddViolationError::MissingRedPhaseProof`. Se houver, executa teste e exige sucesso (`exit_code == 0`), gravando evidencia e atualizando estado para `GreenVerified`.
- `complete_task.rs`:
  - Valida se a tarefa esta em `GreenVerified`. Se estiver, comuta a tarefa para `Committed` ou pronta para merge na worktree.

### Fora do escopo
- Invocacao direta de CLI ou criacao fisica de worktrees.

## Dependencias
- TASK-002 — Requer portas de persistencia e execucao assincrona.

## Premissas
- Qualquer tentativa de avancar para GREEN sem prova RED registrada no ciclo atual sera rejeitada imediatamente com codigo de erro de dominio.

## Criterio de Aceite
- [x] **Dado** uma tarefa em `AwaitingRedPhase`, **quando** `record_tdd_phase` e chamado com `--phase green`, **entao** o caso de uso rejeita com `TddViolationError::MissingRedPhaseProof`.
- [x] **Dado** uma tarefa em `AwaitingRedPhase`, **quando** `record_tdd_phase` roda com `--phase red` e o teste falha, **entao** a tarefa avanca para `RedVerified`.
- [x] **Dado** uma tarefa em `RedVerified`, **quando** `record_tdd_phase` roda com `--phase green` e o teste passa, **entao** avanca para `GreenVerified`.

## Notas Tecnicas
- Arquivos: `crates/trustkit-core/src/use_cases/record_tdd_phase.rs`, `crates/trustkit-core/src/use_cases/complete_task.rs`.
- Suíte de testes unitarios em memoria cobrindo cenarios nominais e violacoes de ordem.

## Rastreabilidade
- Origem: RF-005, RN-002, RN-004
