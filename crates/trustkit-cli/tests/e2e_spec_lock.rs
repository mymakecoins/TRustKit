// E2E tests for `trustkit spec lock`.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

const VALID_SPEC_CONTENT: &str = r#"# Especificacao: Motor de Pagamentos

## 1. Product Trio
- **PM**: Gestao de regras de negocio.
- **Tech Lead**: Integracao de gateways e seguranca.
- **Design / QA**: Criterios de aceitacao e suites E2E.

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart LR
    A[Requisicao PIX] --> B[Validacao de Saldo]
    B --> C[Emissao de Comprovante]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Pagamento via PIX com Saldo Positivo
- **Dado** cliente com saldo de R$ 100,00
- **Quando** solicita transferencia PIX de R$ 30,00
- **Entao** o saldo e debitado para R$ 70,00 e o comprovante gerado

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/pix.spec.ts | SC-001.png |
"#;

const ORPHAN_RULE_SPEC_CONTENT: &str = r#"# Especificacao: Modulo com Lacuna

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart LR
    A[Inicio] --> B[Fim]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Sucesso
- **Dado** pre-condicao valida
- **Quando** evento ocorre
- **Entao** resultado esperado

## Regras de Negocio
| ID | Descricao |
| :--- | :--- |
| RN-001 | Regra devidamente coberta |
| RN-002 | Regra orfa sem tarefa |

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
| RN-002 | RF-002 | - | Node2 | tests/e2e/test.spec.ts | - |
"#;

const INVALID_MERMAID_SPEC_CONTENT: &str = r#"# Especificacao: Mermaid Invalido

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
pie title Frutas
    "Macas" : 40
    "Bananas" : 60
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Cenario
- **Dado** entrada
- **Quando** processo
- **Entao** saida

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/test.spec.ts | SC-001.png |
"#;

#[test]
fn test_spec_lock_success_json_mode() {
    let dir = tempdir().expect("create temp dir");
    let spec_path = dir.path().join("specs/payment/payment-spec.md");
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).expect("create spec dir");
    }
    fs::write(&spec_path, VALID_SPEC_CONTENT).expect("write valid spec");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(dir.path())
        .args([
            "spec",
            "lock",
            "--spec",
            spec_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains(": null") && !stdout.contains(":null"),
        "stdout must not contain null: {stdout}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(json["status"], "SPEC_FROZEN");
    assert_eq!(json["spec_lock_state"], "FROZEN");
    assert_eq!(json["rules_count"], 1);
    assert_eq!(json["requirements_count"], 1);

    let digest = json["blake3_digest"]
        .as_str()
        .expect("blake3_digest string");
    assert_eq!(
        digest.len(),
        64,
        "BLAKE3 digest must be 64 hexadecimal characters"
    );
}

#[test]
fn test_spec_lock_traceability_gap_fails_with_exit_code_3() {
    let dir = tempdir().expect("create temp dir");
    let spec_path = dir.path().join("specs/gap/gap-spec.md");
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).expect("create spec dir");
    }
    fs::write(&spec_path, ORPHAN_RULE_SPEC_CONTENT).expect("write gap spec");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(dir.path())
        .args([
            "spec",
            "lock",
            "--spec",
            spec_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(3);

    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains(": null") && !combined.contains(":null"),
        "output must not contain null: {combined}"
    );
    assert!(
        combined.contains("TraceabilityGap"),
        "output must declare TraceabilityGap violation: {combined}"
    );
    assert!(
        combined.contains("RN-002"),
        "output must specify the orphan rule RN-002: {combined}"
    );
}

#[test]
fn test_spec_lock_invalid_mermaid_fails_with_exit_code_3() {
    let dir = tempdir().expect("create temp dir");
    let spec_path = dir.path().join("specs/mermaid/bad-spec.md");
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).expect("create spec dir");
    }
    fs::write(&spec_path, INVALID_MERMAID_SPEC_CONTENT).expect("write bad mermaid spec");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.current_dir(dir.path())
        .args([
            "spec",
            "lock",
            "--spec",
            spec_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(3);
}
