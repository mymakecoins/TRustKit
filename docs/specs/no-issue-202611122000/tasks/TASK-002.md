# TASK-002: Sub-avaliador Eixo 2 Spec Compliance e Scope Creep

layer: CORE
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-core/tests/dual_axis_scope_creep_tests.rs
expected_files:
  - crates/trustkit-core/src/dual_axis_scope_creep.rs
  - crates/trustkit-core/tests/dual_axis_scope_creep_tests.rs

## Contexto
Sub-avaliador Eixo 2 Spec Compliance e Scope Creep atendendo RN-002 e RF-002 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** um diff na worktree da tarefa que adiciona estruturas, flags ou comportamentos publicos nao previstos em nenhuma regra `RN-XXX` ou requisito `RF-XXX` da `spec.md` selada
- **Quando** o avaliador do eixo `SpecCompliance` compara o diff com a especificacao
- **Entao** o avaliador registra um achado `SpecComplianceDefect::ScopeCreep` citando o trecho excedente e reprova o eixo `SpecCompliance`.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-spec-scope-creep-detection.json
