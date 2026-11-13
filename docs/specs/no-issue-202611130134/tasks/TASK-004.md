# TASK-004: Motor de Retrospectiva de Harness sobre Eventos SQLite WAL

layer: DB
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-db/tests/sqlite_tdd_and_cert_tests.rs
expected_files:
  - crates/trustkit-db/src/sqlite_tdd_and_cert.rs
  - crates/trustkit-db/tests/sqlite_tdd_and_cert_tests.rs

## Contexto
Motor de Retrospectiva de Harness sobre Eventos SQLite WAL atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** o historico de eventos, interceptacoes de `pre-tool` hook e reprovacoes do Gauntlet armazenados em `.trustkit/state.db` durante o ciclo ativo
- **Quando** o comando `trustkit done --json` e executado no Gate 6
- **Entao** o sistema gera um `HarnessRetroReport` apontando oportunidades de melhoria no ambiente (Novas Checagens Automatizadas, Ponteiros de Navegacao, Economia de Ferramentas e Regras de Revisao) ordenadas por severidade.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-harness-retro-wal-analysis.json
