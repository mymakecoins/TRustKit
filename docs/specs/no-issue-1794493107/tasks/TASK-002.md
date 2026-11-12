# TASK-002: Cruzamento com In-Degree no Grafo AST

layer: AST
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-ast/tests/wave_coupling_analyzer_tests.rs
expected_files:
  - crates/trustkit-ast/src/wave_coupling_analyzer.rs
  - crates/trustkit-ast/tests/wave_coupling_analyzer_tests.rs

## Contexto
Cruzamento com In-Degree no Grafo AST atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma tarefa unica que modifica diretamente um trait ou struct central cujo `in_degree` no grafo estrutural excede o limiar de seguranca (ex: mais de 8 dependentes diretos)
- **Quando** o comando `trustkit wave plan --json` analisa o plano de ondas
- **Entao** o planejador identifica o risco de quebra em cascata (`HighBlastRadiusDetected`) e exige ou recomenda a decomposicao no padrao `Expand-Contract`.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-blast-radius-coupling.json
