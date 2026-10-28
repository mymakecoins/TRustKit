// E2E tests for `trustkit spec init --from-prd`.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_spec_init_from_prd_success_json_mode() {
    let dir = tempdir().expect("create temp dir");
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).expect("create docs dir");

    let prd_file = docs_dir.join("PRD_Sample.md");
    let prd_content = r#"# Central Auth Service

## 1. Regras de Negocio

| ID | Descricao |
| :--- | :--- |
| RN-001 | Todo usuario administrativo deve utilizar autenticacao em dois fatores |
| RN-002 | Senhas devem expirar apos 90 dias de inatividade |

## 2. Requisitos Funcionais

| ID | Descricao |
| :--- | :--- |
| RF-001 | Validar token TOTP de 6 digitos no login |
| RF-002 | Bloquear sessao e disparar e-mail de redefinicao de senha aos 90 dias |
"#;
    fs::write(&prd_file, prd_content).expect("write prd file");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(dir.path())
        .args([
            "spec",
            "init",
            "--from-prd",
            "docs/PRD_Sample.md",
            "--name",
            "central-auth",
            "--output-dir",
            "specs/001-central-auth",
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify zero null in output
    assert!(
        !stdout.contains(": null") && !stdout.contains(":null"),
        "stdout must not contain null: {stdout}"
    );

    // Verify JSON payload structure
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(json["status"], "SPEC_INITIALIZED");
    assert_eq!(json["topic_slug"], "central-auth");
    assert_eq!(json["rules_extracted_count"], 2);
    assert_eq!(json["requirements_extracted_count"], 2);

    let generated_file = dir
        .path()
        .join("specs/001-central-auth/central-auth-spec.md");
    assert!(
        generated_file.exists(),
        "generated spec file must exist at {:?}",
        generated_file
    );

    let content = fs::read_to_string(&generated_file).expect("read generated spec");
    assert!(content.contains("Central Auth Service") || content.contains("central-auth"));
    assert!(content.contains("RN-001"));
    assert!(content.contains("RN-002"));
    assert!(content.contains("RF-001"));
    assert!(content.contains("RF-002"));
    assert!(content.contains("```mermaid"));
    assert!(content.contains("Matriz de Rastreabilidade"));
}

#[test]
fn test_spec_init_missing_prd_file_fails_with_exit_code_2() {
    let dir = tempdir().expect("create temp dir");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.current_dir(dir.path())
        .args([
            "spec",
            "init",
            "--from-prd",
            "docs/inexistente.md",
            "--json",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("SPEC_INGESTION_ERROR"));
}

#[test]
fn test_spec_init_prd_without_rules_fails_with_exit_code_2() {
    let dir = tempdir().expect("create temp dir");
    let prd_file = dir.path().join("empty_prd.md");
    fs::write(&prd_file, "# Documento Sem Regras\n\nApenas notas.").expect("write empty prd");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.current_dir(dir.path())
        .args(["spec", "init", "--from-prd", "empty_prd.md", "--json"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("SPEC_INGESTION_ERROR"));
}
