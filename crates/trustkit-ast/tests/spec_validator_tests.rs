use trustkit_ast::SpecValidator;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::SpecValidatorPort;

const VALID_SPEC: &str = r#"# Especificacao: Modulo de Pagamentos

## 1. Product Trio
- **PM**: Regras de negocio e valor.
- **Tech Lead**: Arquitetura.
- **Design / QA**: Criterios de aceitacao.

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart LR
    A[Entrada Requisitos] --> B[Validacao RN]
    B --> C[Execucao TDD]
    C --> D[Evidencia Playwright]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Pagamento Aprovado
- **Dado** conta de usuario com saldo suficiente
- **Quando** a cobranca de R$ 50 for processada
- **Entao** o saldo e debitado e o recibo emitido

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/pay.spec.ts | SC-001.png |
"#;

const INVALID_MERMAID_SPEC: &str = r#"# Especificacao: Modulo Quebrado

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
pie title Pets
    "Dogs" : 386
    "Cats" : 85
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Sucesso
- **Dado** ok
- **Quando** ok
- **Entao** ok

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
"#;

const INVALID_GHERKIN_SPEC: &str = r#"# Especificacao: Modulo Gherkin Incompleto

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart TD
    A[Inicio] --> B[Fim]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Incompleto
- **Dado** o usuario logado
- **Quando** tenta acessar a rota protegida
<!-- Falta Entao -->

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
"#;

const ORPHAN_RULE_SPEC: &str = r#"# Especificacao: Modulo com Regra Orfa

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart TD
    A[Inicio] --> B[Fim]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Cenário
- **Dado** pre-condicao
- **Quando** acao
- **Entao** resultado

## Regras de Negocio
| ID | Descricao |
| :--- | :--- |
| RN-001 | Regra com cobertura |
| RN-002 | Regra orfa sem tarefa |

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
| RN-002 | RF-002 | - | Node2 | tests/e2e/test.spec.ts | - |
"#;

#[test]
fn test_valid_spec_passes_all_checks() {
    let validator = SpecValidator;
    let report = validator
        .validate_specification(VALID_SPEC)
        .expect("Valid spec must pass");

    assert!(report.mermaid_valid);
    assert!(report.gherkin_valid);
    assert!(report.orphan_rules.is_empty());
    assert_eq!(report.total_rules, 1);
    assert_eq!(report.total_requirements, 1);
}

#[test]
fn test_invalid_mermaid_fails_validation() {
    let validator = SpecValidator;
    let res = validator.validate_specification(INVALID_MERMAID_SPEC);

    match res {
        Err(DomainViolation::TraceabilityGap { orphan_rule, .. }) => {
            assert_eq!(orphan_rule, "MERMAID_SYNTAX");
        }
        other => panic!("Expected TraceabilityGap for Mermaid syntax, got: {other:?}"),
    }
}

#[test]
fn test_invalid_gherkin_fails_validation() {
    let validator = SpecValidator;
    let res = validator.validate_specification(INVALID_GHERKIN_SPEC);

    match res {
        Err(DomainViolation::TraceabilityGap { orphan_rule, .. }) => {
            assert_eq!(orphan_rule, "GHERKIN_SYNTAX");
        }
        other => panic!("Expected TraceabilityGap for Gherkin syntax, got: {other:?}"),
    }
}

#[test]
fn test_orphan_rule_fails_validation_with_exact_rule_id() {
    let validator = SpecValidator;
    let res = validator.validate_specification(ORPHAN_RULE_SPEC);

    match res {
        Err(DomainViolation::TraceabilityGap {
            orphan_rule,
            message,
        }) => {
            assert_eq!(orphan_rule, "RN-002");
            assert!(
                message.contains("RN-002"),
                "Message must cite the orphan rule"
            );
        }
        other => panic!("Expected TraceabilityGap with RN-002, got: {other:?}"),
    }
}

#[test]
fn test_gherkin_with_nested_keywords_in_description_passes() {
    let spec = r#"# Especificacao: Modulo Complexo

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart LR
    A --> B
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Sucesso
- **Dado** requisito RF-001 habilitado
- **Quando** o processamento com o dado anterior for acionado
- **Entao** o dado deve ser gravado e validado

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
"#;

    let validator = SpecValidator;
    let res = validator.validate_specification(spec);
    assert!(res.is_ok(), "Expected valid spec, got: {res:?}");
}
