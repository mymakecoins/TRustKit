# TASK-002: Orquestrador de Auto-Atualizacao Git e Cargo com Disjuntor Semantico

layer: ORCH
depends_on: [TASK-001]
target_test_file: crates/trustkit-orch/tests/self_update_orchestrator_tests.rs
expected_files:
  - crates/trustkit-orch/src/self_update_orchestrator.rs
  - crates/trustkit-orch/src/lib.rs
  - crates/trustkit-orch/tests/self_update_orchestrator_tests.rs

## Contexto
Implementar `UpdateCommandRunnerPort`, `ProcessUpdateCommandRunner` e `SelfUpdateOrchestrator` em `crates/trustkit-orch/src/self_update_orchestrator.rs` suportando verificacao `--check`, reconstrucao `--force`, fluxo de repositorio fonte (fetch, verificacao de arvore limpa, fast-forward/merge de `main` e `cargo build --release`), fallback via `cargo install --git`, simulacao controlada via `TRUSTKIT_UPDATE_SIMULATE` e disjuntor semantico preservando os binarios ativos em caso de erro, atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um contexto de repositorio fonte ou instalacao global gerenciada com commits local e remoto de `main`
- **Quando** o orquestrador executar `SelfUpdateOrchestrator::execute_update` com `--check`, `--force` ou diante de arvore suja ou falha de compilacao
- **Entao** `--check` nao altera arquivos locais, atualizacoes aplicam fast-forward/build ou `cargo install --git`, e falhas acionam `DomainViolation::InfrastructureBoundaryFault` preservando intactos os binarios ativos.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-self-update-orchestrator.json
