# TASK-001: Modelos de Politicas de Cache Batching e Metricas no HarnessRetroEngine

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/tool_caching_policy_tests.rs
expected_files:
  - crates/trustkit-core/src/tool_caching_policy.rs
  - crates/trustkit-core/src/harness_retro_engine.rs
  - crates/trustkit-core/tests/tool_caching_policy_tests.rs

## Contexto
Modelos de Politicas de Cache Batching e Metricas no HarnessRetroEngine atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** os contratos puros de dominio em `trustkit-core` e um conjunto de evidencias de execucao do ciclo SDD
- **Quando** `ToolCachingPolicy`, `BatchExecutionConfig` e `HarnessRetroEngine::analyze_events` processarem configuracoes de `trustkit.toml` e eventos de execucao com cache ou batching ativos
- **Entao** valida os parametros de politica, quantifica deterministicamente a taxa de reaproveitamento de cache e o tempo medio de ciclo em `ToolEconomyMetrics`, e reflete a economia de ferramentas na retrospectiva do Gate 6.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-tool-caching-policy-core.json
