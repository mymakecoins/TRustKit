// E2E tests for `trustkit wave plan`.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

const VALID_SPEC_CONTENT: &str = r#"# Especificacao: Auth Service

## 1. Product Trio
- **PM**: Gestao de regras de negocio.
- **Tech Lead**: Seguranca.
- **Design / QA**: Criterios de aceitacao.

## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart LR
    A[Inicio] --> B[Fim]
```

## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Login
- **Dado** usuario cadastrado
- **Quando** informa credenciais
- **Entao** recebe token de autenticacao

## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/auth.spec.ts | SC-001.png |
"#;

#[test]
fn test_wave_plan_success_json_mode() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();

    // 1. Congelar especificacao para cumprir a pre-condicao (spec_lock_state: "FROZEN")
    let spec_path = base_dir.join("specs/auth/auth-spec.md");
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).expect("create spec dir");
    }
    fs::write(&spec_path, VALID_SPEC_CONTENT).expect("write valid spec");

    let mut lock_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    lock_cmd
        .current_dir(base_dir)
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

    // 2. Criar tarefas atomicas
    let tasks_dir = base_dir.join("specs/auth/tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");

    let t1 = r#"# TASK-001: Modelo de Dados
depends_on: []
Arquivos Esperados:
- src/auth.rs
"#;
    let t2 = r#"# TASK-002: Validador TOTP
depends_on:
- TASK-001
Arquivos Esperados:
- src/totp.rs
"#;
    let t3 = r#"# TASK-003: Logger de Auditoria
depends_on: []
Arquivos Esperados:
- src/auth.rs
"#;

    fs::write(tasks_dir.join("TASK-001.md"), t1).expect("write TASK-001");
    fs::write(tasks_dir.join("TASK-002.md"), t2).expect("write TASK-002");
    fs::write(tasks_dir.join("TASK-003.md"), t3).expect("write TASK-003");

    // 3. Executar wave plan
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(base_dir)
        .args([
            "wave",
            "plan",
            "--tasks-dir",
            tasks_dir.to_str().unwrap(),
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
    assert_eq!(json["status"], "WAVES_PLANNED");
    assert_eq!(json["total_tasks"], 3);
    assert_eq!(json["total_waves"], 2);

    let waves = json["waves"].as_array().expect("waves array");
    assert_eq!(waves.len(), 2);
    assert_eq!(waves[0]["wave_index"], 1);
    assert_eq!(waves[0]["tasks"][0]["task_id"], "TASK-001");

    let wave_2_tasks = waves[1]["tasks"].as_array().expect("wave 2 tasks");
    let wave_2_ids: Vec<&str> = wave_2_tasks
        .iter()
        .map(|t| t["task_id"].as_str().unwrap())
        .collect();
    assert!(wave_2_ids.contains(&"TASK-002"));
    assert!(wave_2_ids.contains(&"TASK-003"));
}

#[test]
fn test_wave_plan_cycle_detected_fails_with_exit_code_4() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();

    // Congelar especificacao
    let spec_path = base_dir.join("specs/auth/auth-spec.md");
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).expect("create spec dir");
    }
    fs::write(&spec_path, VALID_SPEC_CONTENT).expect("write valid spec");

    let mut lock_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    lock_cmd
        .current_dir(base_dir)
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

    // Criar tarefas circulares
    let tasks_dir = base_dir.join("specs/auth/tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");

    let t1 = r#"# TASK-001: Task 1
depends_on:
- TASK-002
Arquivos Esperados:
- src/a.rs
"#;
    let t2 = r#"# TASK-002: Task 2
depends_on:
- TASK-001
Arquivos Esperados:
- src/b.rs
"#;

    fs::write(tasks_dir.join("TASK-001.md"), t1).expect("write TASK-001");
    fs::write(tasks_dir.join("TASK-002.md"), t2).expect("write TASK-002");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(base_dir)
        .args([
            "wave",
            "plan",
            "--tasks-dir",
            tasks_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(4);

    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains(": null") && !combined.contains(":null"),
        "output must not contain null: {combined}"
    );
    assert!(
        combined.contains("CYCLE_DETECTED") || combined.contains("CycleDetected"),
        "output must declare CycleDetected: {combined}"
    );
}

#[test]
fn test_wave_plan_fails_when_spec_not_frozen_exit_code_1() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();

    // Sem congelar spec
    let tasks_dir = base_dir.join("specs/auth/tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");

    let t1 = r#"# TASK-001: Task 1
depends_on: []
Arquivos Esperados:
- src/a.rs
"#;
    fs::write(tasks_dir.join("TASK-001.md"), t1).expect("write TASK-001");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.current_dir(base_dir)
        .args([
            "wave",
            "plan",
            "--tasks-dir",
            tasks_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(1);
}
