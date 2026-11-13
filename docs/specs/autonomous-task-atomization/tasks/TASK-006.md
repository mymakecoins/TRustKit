# TASK-006: Verificador de Saida RED contra Filtro Vazio e Erro de Compilacao

layer: CORE
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-core/tests/tdd_red_output_verifier_tests.rs
expected_files:
  - crates/trustkit-core/src/use_cases/record_tdd_phase.rs
  - crates/trustkit-core/tests/tdd_red_output_verifier_tests.rs

## Contexto
Refinar a validacao da fase RED em `record_tdd_phase` para inspecionar a saida do executor de testes, rejeitando tanto execucoes onde zero testes foram filtrados (`0 passed; 0 failed` com `ZeroFilteredTestsFault`) quanto falhas de compilacao do compilador Rust (`error[E` ou `could not compile` com `CompilationFailureInRedPhaseFault`).

## Criterios de Aceitacao (Gherkin)
- **Dado** o resultado de execucao do `cargo test` durante o registro da fase RED
- **Quando** o verificador analisar o codigo de saida, stdout e stderr
- **Entao** deve aprovar a fase RED somente se o codigo compilar sem erros e ao menos um teste executado falhar por assercao.

## Rastreabilidade
- Regra de Negocio: RN-006
- Requisito Funcional: RF-006
- Evidencia: EV-006-tdd-red-output-verifier.json
