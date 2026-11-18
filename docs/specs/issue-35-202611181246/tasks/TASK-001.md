# TASK-001: Contratos de Dominio e NextStepResolver para Auto-Atualizacao

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/self_update_domain_tests.rs
expected_files:
  - crates/trustkit-core/src/update_entities.rs
  - crates/trustkit-core/src/next_step_status_policy.rs
  - crates/trustkit-core/src/entities.rs
  - crates/trustkit-core/src/ports.rs
  - crates/trustkit-core/src/lib.rs
  - crates/trustkit-core/tests/self_update_domain_tests.rs

## Contexto
Definir os tipos puros de dominio (`SelfUpdateStatus`, `UpdateInstallStrategy`, `SelfUpdateRequest`, `SelfUpdateReport`, `SelfUpdatePort`) em `crates/trustkit-core/src/update_entities.rs` e implementar `NextStepResolver::for_update` em `crates/trustkit-core/src/next_step_status_policy.rs`, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** os tipos de dominio de auto-atualizacao `SelfUpdateStatus`, `UpdateInstallStrategy`, `SelfUpdateRequest`, `SelfUpdateReport` e `SelfUpdatePort`
- **Quando** o estado de verificacao ou atualizacao for avaliado pelo resolvedor contextual `NextStepResolver::for_update`
- **Entao** o sistema retorna recomendacoes deterministicas priorizadas direcionando para `trustkit update` quando houver atualizacao pendente ou `trustkit doctor` e `trustkit graph build` apos o processo.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-self-update-domain.json
