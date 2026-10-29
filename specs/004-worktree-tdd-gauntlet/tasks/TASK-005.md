# TASK-005: Implementar Casos de Uso do Gauntlet Camada 1 e Circuit Breaker

> **Camada:** Dominio  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O Gate 5 é o coração do Gauntlet V2: após a validação ultra-rápida da Camada 0 (AST), o sistema deve orquestrar a Camada 1 composta pelo Swarm de 4 Críticos independentes (Domínio/Adversarial, Contrato, Build/Lint e Playwright E2E). Caso o Gauntlet acumule 5 reprovações consecutivas, o sistema deve acionar o Circuit Breaker (Invariante #5), travando novas execuções até intervenção humana.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/`:
- `run_gauntlet_layer1.rs`:
  - Verificar se a Camada 0 retornou `CleanPass`.
  - Disparar os 4 Críticos via `AsyncExecutionSupervisorPort`.
  - Validar a regra de aprovação unânime (`RN-003`): todos os 4 críticos devem retornar `PASS`.
  - Invocar o `VisualEvidencePort` para restaurar automaticamente screenshots alheias regravadas pelo Playwright.
  - Gravar as evidências individuais de cada crítico no repositório.
- `trip_circuit_breaker.rs`:
  - Incrementar o contador de tentativas (`gauntlet_attempt`).
  - Se `gauntlet_attempt == 5`, alterar o status do ciclo para `CycleStatus::CircuitBreakerTripped` e produzir o sumário para o comando `trustkit doctor`.

### Fora do escopo
- Execução concreta do Playwright no sistema operacional (delegada ao adaptador de orquestração).

## Dependencias
- TASK-002 — Requer portas de orquestração assíncrona, evidências visuais e repositório.

## Premissas
- A Camada 1 só pode ser acionada se a Camada 0 estiver 100% limpa.
- O Circuit Breaker dispara estritamente na 5ª falha consecutiva.

## Criterio de Aceite
- [x] **Dado** uma Camada 0 reprovada, **quando** o Gauntlet é acionado, **entao** a Camada 1 nem sequer é disparada.
- [x] **Dado** que 3 críticos passaram e o Playwright falhou, **quando** avaliado, **entao** o veredito geral é `FAIL` e o contador de tentativas é incrementado.
- [x] **Dado** a 5ª tentativa reprovada consecutiva, **quando** finalizada, **entao** o ciclo transiciona para `CircuitBreakerTripped` e novas execuções são bloqueadas.

## Notas Tecnicas
- Arquivos: `crates/trustkit-core/src/use_cases/run_gauntlet_layer1.rs`, `crates/trustkit-core/src/use_cases/trip_circuit_breaker.rs`.
- Testes com mocks das portas cobrindo os cenários de aprovação unânime e disparo do disjuntor.

## Rastreabilidade
- Origem: RF-008, RF-009, RN-003, RN-005
