# TASK-001: Entidades e Modelos WaveViewReport

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/wave_view_entities_tests.rs
expected_files:
  - crates/trustkit-core/src/wave_view_entities.rs
  - crates/trustkit-core/tests/wave_view_entities_tests.rs

## Contexto
Entidades e Modelos WaveViewReport atendendo RN-001 e RF-062 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** que o crate `trustkit-core` define os modelos de consulta de ciclo e tarefas
- **Quando** a requisicao `QueryWaveViewRequest` for processada pelo caso de uso `query_wave_view`
- **Entao** o modelo de saida `WaveViewReport` e suas entidades aninhadas (`WaveViewItem`, `TaskViewItem`) sao construidos utilizando tipos estritos de `std`, 100% livres de valores nulos e sem violar o isolamento hexagonal.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-062
- Evidencia: EV-001-wave-view-entities.json
