# TASK-001: Captura de Diff do Ciclo e Spec Congelada no Gate 2

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/dual_axis_entities_tests.rs
expected_files:
  - crates/trustkit-core/src/dual_axis_entities.rs
  - crates/trustkit-core/tests/dual_axis_entities_tests.rs

## Contexto
Captura de Diff do Ciclo e Spec Congelada no Gate 2 atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o modulo de entidades de revisao semantica no `trustkit-core`
- **Quando** um apontamento de revisao e instanciado
- **Entao** ele e tipado estritamente sob o eixo `StandardsAndSmells` (distinguindo violacao rigida do repositorio de julgamento heuristico `FowlerCodeSmell`) ou sob o eixo `SpecCompliance` (`MissingOrPartialRequirement`, `WrongImplementation`, `ScopeCreep`).

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-dual-axis-and-fowler-entities.json
