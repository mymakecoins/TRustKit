# TASK-003: Equivalencia Contratual dos Comandos do Ciclo SDD

layer: CLI
depends_on:
  - TASK-002
target_test_file: crates/trustkit-cli/tests/tk_subcommands_contract_tests.rs
expected_files:
  - crates/trustkit-cli/tests/tk_subcommands_contract_tests.rs

## Contexto
Desenvolver suite de testes de integracao validando que todos os subcomandos do ciclo SDD (ex: status, doctor, wave, task, context, gauntlet) comportam-se de forma estritamente identica quando invocados via tk ou trustkit, tanto em saida ASCII quanto em formato JSON estruturado null-free.

## Criterios de Aceitacao (Gherkin)
- **Dado** qualquer subcomando do ciclo SDD como status, doctor ou task
- **Quando** o subcomando for invocado via tk ou trustkit tanto em modo texto quanto em modo json
- **Entao** a resposta, codigo de saida, veredito e contrato JSON emitidos sao estritamente equivalentes.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-tk-subcommands-contract.json
