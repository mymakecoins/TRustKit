# TASK-005: Tratamento de Disjuntor (Circuit Breaker) e Diagnostico

> **Camada:** Dominio Core
> **Estimativa:** P
> **Status:** Pendente

depends_on: [TASK-002]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/run_autonomous_cycle.rs
test_file: crates/trustkit-core/tests/autonomous_circuit_breaker_tests.rs

---

## Contexto
O modo autonomo nunca deve entrar em loop infinito se houver falha de especificacao no Gate 1, violacao de TDD no Gate 4 ou reprovacao consecutiva no Gauntlet (Gate 5). O disjuntor de seguranca deve disparar, interromper o fluxo e emitir diagnostico estruturado.

## Escopo
- Fiscalizar o limite de falhas e disparar codigo de saida semantico apropriado (1 a 6).
- Emitir diagnostico acionavel para intervencao humana.
