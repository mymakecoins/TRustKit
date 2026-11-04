// Tests for SpecLinter (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::time::Instant;
use trustkit_ast::{SpecLintVerdict, SpecLinter};

const VALID_SPEC: &str = r#"# Especificacao: 009-sample-feature

## 1. Product Trio

- **PM**: Regras de negocio definidas.
- **Tech Lead**: Arquitetura hexagonal.
- **Design / QA**: Criterios de aceitacao BDD.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[Entrada] --> Node2[Saida]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Execucao Nominal

- **Dado** ambiente pronto
- **Quando** executa
- **Entao** conclui com sucesso

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/unit.rs | EV-001.json |
"#;

#[test]
fn test_spec_linter_clean_pass_on_valid_spec() {
    let verdict = SpecLinter::lint_markdown(VALID_SPEC);
    assert_eq!(verdict, SpecLintVerdict::CleanPass);
    assert!(verdict.is_clean());
    assert!(verdict.violations().is_empty());
}

#[test]
fn test_spec_linter_detects_missing_title_and_mermaid_violations() {
    let invalid_spec = r#"
## 1. Product Trio
- **PM**: Ok
- **Tech Lead**: Ok
- **QA**: Ok

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
graph TD
    A --> B
```

## 3. Criterios de Aceitacao (Gherkin)
- **Dado** teste
- **Quando** roda
- **Entao** passa

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa |
| RN-001 | RF-001 | TASK-001 |
"#;
    let verdict = SpecLinter::lint_markdown(invalid_spec);
    assert!(!verdict.is_clean());
    let codes: Vec<String> = verdict
        .violations()
        .iter()
        .map(|v| v.code.clone())
        .collect();
    assert!(codes.contains(&"SPEC-001".to_string()));
    assert!(codes.contains(&"SPEC-003".to_string()));
}

#[test]
fn test_spec_linter_detects_ai_trailers_and_emojis() {
    let dirty_spec = format!(
        "{}\nCo-Authored-By: Claude <noreply@anthropic.com>\nLinha com emoji de alerta: \u{1F680}\n",
        VALID_SPEC
    );
    let verdict = SpecLinter::lint_markdown(&dirty_spec);
    assert!(!verdict.is_clean());
    let codes: Vec<String> = verdict
        .violations()
        .iter()
        .map(|v| v.code.clone())
        .collect();
    assert!(codes.contains(&"SPEC-006".to_string()));
    assert!(codes.contains(&"SPEC-007".to_string()));
}

#[test]
fn test_spec_linter_performance_benchmark_sub_3ms() {
    let start = Instant::now();
    for _ in 0..100 {
        let _ = SpecLinter::lint_markdown(VALID_SPEC);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() * 1000.0 / 100.0;
    assert!(
        avg_ms < 3.0,
        "SpecLinter average time exceeded 3ms threshold: {:.4}ms",
        avg_ms
    );
}
