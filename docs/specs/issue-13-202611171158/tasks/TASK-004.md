# TASK-004: Renderizador Textual Formatado por Ondas

layer: CLI
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-cli/tests/cli_wave_view_json_tests.rs
expected_files:
  - crates/trustkit-cli/src/cli_wave_view_json.rs
  - crates/trustkit-cli/tests/cli_wave_view_json_tests.rs

## Contexto
Renderizador Textual Formatado por Ondas atendendo RN-004 e RF-065 no componente Node5.

## Criterios de Aceitacao (Gherkin)
- **Dado** que a flag `--json` e fornecida na chamada `trustkit wave view --json`
- **Quando** o payload JSON de resposta for serializado
- **Entao** todos os campos opcionais ausentes sao omitidos ou serializados com tipos sentinela (vetores vazios ou enums explicitos), proibindo a presenca de literais `null` no JSON.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-065
- Evidencia: EV-004-cli-wave-view-json.json
