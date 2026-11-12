# TASK-001: Implementacao do Caso de Uso CloseCycle com Fechamento de Governanca Nominal

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/src/use_cases/close_cycle_tests.rs
expected_files:
  - crates/trustkit-core/src/use_cases/close_cycle.rs
  - crates/trustkit-core/src/use_cases/close_cycle_tests.rs

## Contexto
Entrada CLI trustkit close e processamento de fechamento formal de governanca atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo SDD ativo no estado Certified aprovado no Gate 6
- **Quando** o desenvolvedor executa o comando `trustkit close`
- **Entao** o ciclo e formalmente encerrado no SQLite, o card e movido para ReadyForTest e a mensagem confirma o fechamento sem alegar merge Git local.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-close-cycle-nominal.json
