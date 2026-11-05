# TASK-002: Implementar Entidades e Caso de Uso run_autonomous_cycle

> **Camada:** Dominio Core
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/run_autonomous_cycle.rs
- crates/trustkit-core/src/entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/tests/autonomous_lifecycle_tests.rs

---

## Contexto
O nucleo de dominio precisa de um caso de uso que orquestre deterministamente a transicao dos Gates 0 a 3: inicializacao do ciclo a partir de `--issue` ou `--spec`, linter sintatico com validacao imediata, congelamento BLAKE3 e calculo do plano topologico de ondas.

## Escopo
- Criar `AutonomousCycleRequest` e `AutonomousCycleOutcome` em `trustkit-core`.
- Implementar maquina de estado que valida os pre-requisitos de cada gate antes da transicao.
- Garantir zero alocacoes nulas e erro tipado `DomainViolation`.
