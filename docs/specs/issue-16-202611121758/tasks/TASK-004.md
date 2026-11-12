# TASK-004: Renderizacao de Mensagem Semantica e Proximos Passos no Presenter

layer: CLI
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-cli/src/presenter_tests.rs
expected_files:
  - crates/trustkit-cli/src/presenter.rs
  - crates/trustkit-cli/src/presenter_tests.rs

## Contexto
Renderizacao de Mensagem Semantica e Proximos Passos atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** a conclusao bem-sucedida do comando `trustkit close`
- **Quando** a saida for renderizada em modo TTY humano ou `--json`
- **Entao** o payload exibe a transicao para ReadyForTest e recomenda como proximo passo primario `trustkit pr --target develop` sem campos nulos e sem emojis.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-close-presenter-null-free.json
