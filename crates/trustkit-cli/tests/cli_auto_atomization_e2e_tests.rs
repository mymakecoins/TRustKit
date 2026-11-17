// Integration tests for TASK-009: Integracao E2E Completa de Atomizacao e Execucao no trustkit auto.
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
use trustkit_ast::SpecMatrixAtomizer;
use trustkit_orch::TaskSynthesizer;

const VALID_ATOMIZABLE_SPEC: &str = r#"# Especificacao: Feature Autonoma Completa

## 1. Product Trio

- **PM**: Execucao autonoma com sintese deterministica de tasks.
- **Tech Lead**: Orquestracao em ondas e verificacao TDD sem falhas de compilacao.
- **QA**: Gate 5 Gauntlet V2 e homologacao de branch develop.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Gate0[Setup] --> Gate3[Ondas]
    Gate3 --> Gate4[TDD]
    Gate4 --> Gate5[Gauntlet]
    Gate5 --> Gate6[Merge]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Pipeline E2E Completo de Atomizacao

- **Dado** uma especificacao formal valida sem tarefas previas
- **Quando** o comando trustkit auto for executado
- **Entao** deve atomizar tarefas, planejar ondas e homologar gates.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Gate3 | tests/auto_spec_test.rs | EV-001-atom.json |
"#;

const EMPTY_BACKLOG_SPEC: &str = r#"# Especificacao Vazia Sem Tarefas

## 1. Product Trio

- **PM**: Spec sem matriz de rastreabilidade.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Gate0[Setup] --> Gate1[Spec]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Spec Sem Tarefas

- **Dado** uma spec sem tabela de rastreabilidade
- **Quando** auto for executado
- **Entao** deve disparar circuit breaker.
"#;

fn init_git_repo(path: &Path) {
    let run = |args: &[&str]| {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .expect("git execution failed");
        assert!(status.success());
    };
    run(&["init", "-b", "main"]);
    run(&["config", "user.email", "dev@example.com"]);
    run(&["config", "user.name", "Dev Tester"]);

    fs::create_dir_all(path.join("src")).expect("src dir");
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
    run(&["commit", "-m", "chore: #1 initial commit"]);
    run(&["branch", "develop"]);
}

#[test]
fn test_task_009_auto_atomizes_and_synthesizes_tasks_when_missing() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    let spec_path = base_dir.join("docs").join("specs").join("feat-atom");
    fs::create_dir_all(&spec_path).expect("create spec dir");
    fs::write(spec_path.join("spec.md"), VALID_ATOMIZABLE_SPEC).expect("write spec");

    // Garantir que a pasta tasks/ NAO existe previamente
    assert!(!spec_path.join("tasks").exists());

    let mut auto_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let assert = auto_cmd
        .current_dir(base_dir)
        .args([
            "auto",
            "--spec",
            "docs/specs/feat-atom/spec.md",
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

    // Verificar que as tasks foram materializadas no disco
    let tasks_dir = spec_path.join("tasks");
    assert!(tasks_dir.exists(), "tasks/ directory must be synthesized");
    assert!(
        tasks_dir.join("TASK-001.md").exists(),
        "TASK-001.md must be generated"
    );

    let gates: Vec<i64> = json_val["gates_cleared"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    assert!(gates.contains(&3), "Gate 3 (Wave Planning) must be cleared");
    assert!(
        gates.contains(&4),
        "Gate 4 (Tasks Implemented) must be cleared"
    );
    assert!(gates.contains(&5), "Gate 5 (Gauntlet V2) must be cleared");
    assert!(gates.contains(&6), "Gate 6 (Done & Merge) must be cleared");
}

#[test]
fn test_task_009_auto_triggers_empty_backlog_circuit_breaker() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    let spec_path = base_dir.join("docs").join("specs").join("empty-spec");
    fs::create_dir_all(&spec_path).expect("create spec dir");
    fs::write(spec_path.join("spec.md"), EMPTY_BACKLOG_SPEC).expect("write spec");

    let mut auto_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let assert = auto_cmd
        .current_dir(base_dir)
        .args([
            "auto",
            "--spec",
            "docs/specs/empty-spec/spec.md",
            "--target",
            "develop",
            "--json",
        ])
        .assert()
        .failure()
        .code(1);

    let stderr_str = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let json_val: Value = serde_json::from_str(&stderr_str).expect("Valid JSON response in stderr");
    assert_eq!(json_val["verdict"], "FAIL");
    assert_eq!(json_val["violation"]["kind"], "EMPTY_TASK_BACKLOG_FAULT");
}

#[test]
fn test_task_009_atomizer_and_synthesizer_in_memory_contract() {
    let drafts = SpecMatrixAtomizer::atomize_markdown(VALID_ATOMIZABLE_SPEC)
        .expect("Atomization must succeed");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].task_id, "TASK-001");
    assert_eq!(drafts[0].business_rule_id, "RN-001");

    let dir = tempdir().expect("tempdir");
    let tasks_dir = dir.path().join("tasks");
    let created = TaskSynthesizer::materialize_tasks_dir(&tasks_dir, &drafts)
        .expect("Synthesis must succeed");
    assert_eq!(created.len(), 1);
    assert!(tasks_dir.join("TASK-001.md").exists());
}
