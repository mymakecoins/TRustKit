// E2E test for autonomous SDD lifecycle with Gates 5 and 6 (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::tempdir;

const VALID_SPEC: &str = r#"# Especificacao: Feature Autonoma

## 1. Product Trio

- **PM**: Execucao autonoma de todo o fluxo.
- **Tech Lead**: Orquestracao de gates deterministica sem saltos.
- **Design / QA**: Verificacao completa de Gates 5 e 6 com certificacao e merge.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Gate0[Setup] --> Gate5[Gauntlet V2]
    Gate5 --> Gate6[Certificacao & Merge]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Execucao Autonoma Nominal de Gates 5 e 6

- **Dado** uma especificacao valida e tarefas atomicas completas
- **Quando** o comando trustkit auto for disparado
- **Entao** todos os gates devem ser concluidos e o merge para a branch alvo deve ocorrer.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Gate0 | tests/e2e_auto_lifecycle.rs | EV-004-lifecycle.json |
"#;

fn init_git_repo(path: &Path) {
    let run = |args: &[&str]| {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .expect("git failed");
        assert!(status.success());
    };
    run(&["init", "-b", "main"]);
    run(&["config", "user.email", "dev@example.com"]);
    run(&["config", "user.name", "Dev Tester"]);
    fs::create_dir_all(path.join("src")).expect("create src");
    fs::write(
        path.join("Cargo.toml"),
        r#"[package]
name = "dummy-app"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    )
    .expect("write Cargo.toml");
    fs::write(path.join("src").join("lib.rs"), "pub fn ok() {}\n").expect("write lib.rs");
    fs::write(path.join("README.md"), "# Repo Autonomo\n").expect("write readme");
    run(&["add", "."]);
    run(&["commit", "-m", "chore: #10 initial commit"]);
    run(&["branch", "develop"]);
}

#[test]
fn test_task_004_auto_lifecycle_nominal_gates_5_and_6() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    let spec_path = base_dir.join("docs").join("specs").join("feat-auto");
    fs::create_dir_all(&spec_path).expect("create spec dir");
    fs::write(spec_path.join("spec.md"), VALID_SPEC).expect("write spec");

    let mut auto_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let assert = auto_cmd
        .current_dir(base_dir)
        .args([
            "auto",
            "--spec",
            "docs/specs/feat-auto/spec.md",
            "--target",
            "develop",
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let stdout_str = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json_val: Value = serde_json::from_str(&stdout_str).expect("Valid JSON response expected");

    assert_eq!(json_val["verdict"], "PASS");
    assert!(json_val["gates_cleared"].as_array().is_some());
    let gates: Vec<i64> = json_val["gates_cleared"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    assert!(gates.contains(&5), "Gate 5 (Gauntlet) must be cleared");
    assert!(gates.contains(&6), "Gate 6 (Done & Merge) must be cleared");
}
