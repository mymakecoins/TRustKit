# TASK-017: Implementar Suite de Testes de Integracao e E2E para Gates 4, 5 e 6

> **Camada:** Infra/Setup  
> **Estimativa:** G  
> **Status:** Concluído  

---

## Contexto
Para homologar a Fase 4 e certificar o MVP do TRustKit com determinismo absoluto, é necessária uma suíte automatizada de testes E2E que execute a jornada completa dos Gates 4 a 6 em repositórios Git temporários isolados.

## Escopo
Implementar em `crates/trustkit-cli/tests/e2e_gates_4_to_6.rs`:
- Teste E2E de ciclo TDD no Gate 4:
  - Criação de tarefa atômica.
  - Tentativa de `--phase green` sem RED -> falha com Exit Code 3.
  - Execução de `--phase red` com teste falhando -> avanço para `RedVerified`.
  - Execução de `--phase green` com teste passando -> avanço para `GreenVerified`.
  - Conclusão com `trustkit task complete`.
- Teste E2E de execução de onda com worktree efêmera:
  - Disparo de `trustkit wave run --wave 1`.
  - Verificação de isolamento em `.trustkit/worktrees/` e merge determinístico na branch de trabalho.
- Teste E2E do Gauntlet V2 e Circuit Breaker:
  - Camada 0 aprovada -> disparo de críticos.
  - Simulação de 5 falhas consecutivas -> verificação de `CircuitBreakerTripped` e acionamento de `trustkit doctor`.
- Teste E2E do Gate 6:
  - Execução de `trustkit done` -> validação do `gauntlet-report.md` gerado.
  - Execução de `trustkit merge --target develop`.
  - Execução de `trustkit pr` -> verificação de payload e validação de `Ref #<id>`.

### Fora do escopo
- Chamadas reais a servidores remotos do GitHub (utilizar repositórios Git locais em `tempfile` e mocks da CLI `gh`).

## Dependencias
- TASK-015 — Requer Composition Root e todos os comandos integrados.

## Premissas
- A suíte deve executar em modo determinístico e ser compatível com `cargo test`.

## Criterio de Aceite
- [x] **Dado** a suíte de testes E2E, **quando** executada com `cargo test --test e2e_gates_4_to_6`, **entao** todos os cenários passam com sucesso (`test result: ok`).
- [x] **Dado** os cenários de falha (TDD fraudulento, Circuit Breaker, emojis, PR automático), **quando** disparados, **entao** o binário encerra com os códigos de saída exatos previstos na especificação.

## Notas Tecnicas
- Arquivo: `crates/trustkit-cli/tests/e2e_gates_4_to_6.rs`.
- Utilizar `assert_cmd`, `predicates` e `tempfile`.

## Rastreabilidade
- Origem: RF-005, RF-006, RF-008, RF-009, RF-010, RF-011, RF-012
