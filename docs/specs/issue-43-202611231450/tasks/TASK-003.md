# TASK-003: Diagnostico de Cache no Doctor Configuracao TOML e Paridade Documental

layer: CLI
depends_on: ["TASK-001", "TASK-002"]
target_test_file: crates/trustkit-cli/tests/cli_doctor_cache_and_config_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/doctor_cmd.rs
  - crates/trustkit-cli/src/commands/init_cmd.rs
  - crates/trustkit-cli/src/gate_presenter.rs
  - crates/trustkit-cli/tests/cli_doctor_cache_and_config_tests.rs

## Contexto
Diagnostico de Cache no Doctor Configuracao TOML e Paridade Documental atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** um projeto governado pelo TRustKit com `trustkit.toml` configurado e um ciclo SDD ativo
- **Quando** o operador ou agente executar `trustkit doctor --json` ou inicializar o projeto com `trustkit init`
- **Entao** o CLI carrega as politicas de `[harness.execution]` do `trustkit.toml`, detecta a presenca do `sccache` no ambiente, reporta o status do cache local orientando a instalacao caso ausente e mantem paridade documental completa.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-doctor-cache-cli.json
