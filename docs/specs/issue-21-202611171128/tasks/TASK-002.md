# TASK-002: Resolucao Dinamica de Argv0 e Paridade Clap

layer: CLI
depends_on:
  - TASK-001
target_test_file: crates/trustkit-cli/tests/tk_cli_parity_tests.rs
expected_files:
  - crates/trustkit-cli/src/main.rs
  - crates/trustkit-cli/src/cli_contract.rs
  - crates/trustkit-cli/tests/tk_cli_parity_tests.rs

## Contexto
Implementar a resolucao dinamica do nome do binario atraves de argv[0] no main.rs e nos contratos da CLI para garantir que tk e trustkit exibam seus respectivos nomes em mensagens de help, versao e usage, mantendo a execucao dos comandos e subcomandos perfeitamente identica e estavel.

## Criterios de Aceitacao (Gherkin)
- **Dado** o binario tk executado no terminal com as flags --help e --version
- **Quando** o parser de linha de comando processar os argumentos com base em argv0
- **Entao** o retorno exibe codigo de saida zero com o nome de invocacao correspondente preservando compatibilidade sem panics.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-tk-cli-parity.json
