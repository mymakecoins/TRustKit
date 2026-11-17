# TASK-001: Configuracao de Manifesto Cargo com Binario tk

layer: CLI
depends_on: []
target_test_file: crates/trustkit-cli/tests/tk_cargo_binary_manifest_tests.rs
expected_files:
  - crates/trustkit-cli/Cargo.toml
  - crates/trustkit-cli/tests/tk_cargo_binary_manifest_tests.rs

## Contexto
Configurar o manifesto Cargo do crate trustkit-cli adicionando a definicao binaria [[bin]] name = "tk" path = "src/main.rs" ao lado do binario existente trustkit, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o manifesto Cargo do crate trustkit-cli configurado com as entradas binarias trustkit e tk
- **Quando** o comando cargo build for executado no workspace
- **Entao** ambos os binarios target/debug/trustkit e target/debug/tk sao gerados com sucesso no diretorio de destino.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-tk-cargo-binary-manifest.json
