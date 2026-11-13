# TASK-001: Protocolo de Reproducao Minima em 6 Fases no trustkit doctor

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/src/gauntlet_entities_tests.rs
expected_files:
  - crates/trustkit-core/src/gauntlet_entities_tests.rs

## Contexto
Protocolo de Reproducao Minima em 6 Fases no trustkit doctor atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo SDD com falhas registradas no Gauntlet ou com o Circuit Breaker acionado
- **Quando** o operador ou agente executa `trustkit doctor --json`
- **Entao** o diagnostico retorna o `DiagnosticBugLoopPlan` estruturado nas 6 fases (Loop que fica vermelho, Minimizacao de Repro, 3 a 5 Hipoteses Falsificaveis, Instrumentacao com prefixo `[DEBUG-xxxx]`, Correcao na Costura Certa e Limpeza).

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-diagnostic-loop-plan-entities.json
