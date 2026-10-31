// E2E Test verifying full decoupling and independence from external Spec Kit (TASK-008).
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

const VALID_SPEC_DECOUPLED: &str = r#"# Especificacao: 099-native-feature

## 1. Product Trio

- **PM**: Gestao de regras de negocio nativas sem speckit.
- **Tech Lead**: Arquitetura em Rust sem dependencias probabilisticas no core.
- **Design / QA**: Criterios de aceitacao BDD e integridade estatica.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[Entrada de Prompt] --> Node2[Linter de Especificacao]
    Node2 --> Node3[Selo Criptografico Blake3]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Execucao 100% Nativa

- **Dado** o ambiente com binario nativo trustkit disponivel
- **Quando** o desenvolvedor gera prompt, valida com linter e congela a especificacao
- **Entao** todos os gates sao aprovados sem requisitar o ecossistema externo speckit.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | crates/trustkit-cli/tests/e2e_speckit_decoupling.rs | EV-001-decoupled.json |
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
    fs::write(path.join("README.md"), "# Repo Decoupled\n").expect("write readme");
    run(&["add", "README.md"]);
    run(&["commit", "-m", "chore: #99 initial commit"]);
    run(&["branch", "develop"]);
}

#[test]
fn test_e2e_native_specification_and_speckit_independence_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    // 1. Inicializacao do ciclo no Gate 0
    let mut init_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let init_assert = init_cmd
        .current_dir(base_dir)
        .args(["init", "--issue", "99", "--json"])
        .assert()
        .success()
        .code(0);

    let init_out = String::from_utf8(init_assert.get_output().stdout.clone()).unwrap();
    assert!(!init_out.contains(": null"));
    let init_json: Value = serde_json::from_str(&init_out).expect("json");
    let spec_dir_rel = init_json["spec_dir"].as_str().expect("spec_dir");
    let spec_dir = base_dir.join(spec_dir_rel);

    // 2. Geracao de prompt nativo via CLI trustkit spec prompt
    let mut prompt_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let prompt_assert = prompt_cmd
        .current_dir(base_dir)
        .args([
            "spec",
            "prompt",
            "--idea",
            "Autenticacao JWT nativa sem speckit",
            "--context-hops",
            "2",
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let prompt_out = String::from_utf8(prompt_assert.get_output().stdout.clone()).unwrap();
    assert!(!prompt_out.contains(": null"));
    let prompt_json: Value = serde_json::from_str(&prompt_out).expect("json");
    assert_eq!(prompt_json["verdict"], "PASS");
    assert!(prompt_json["scaffold_markdown"].as_str().unwrap().contains("## 1. Product Trio"));

    // 3. Gravacao do spec.md com especificacao nativa
    let spec_file = spec_dir.join("spec.md");
    fs::create_dir_all(&spec_dir).expect("create spec dir");
    fs::write(&spec_file, VALID_SPEC_DECOUPLED).expect("write spec");

    // 4. Validacao estatica via trustkit spec lint
    let mut lint_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let lint_assert = lint_cmd
        .current_dir(base_dir)
        .args([
            "spec",
            "lint",
            "--file",
            spec_file.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let lint_out = String::from_utf8(lint_assert.get_output().stdout.clone()).unwrap();
    assert!(!lint_out.contains(": null"));
    let lint_json: Value = serde_json::from_str(&lint_out).expect("json");
    assert_eq!(lint_json["verdict"], "PASS");
    assert_eq!(lint_json["violations_count"], 0);

    // 5. Congelamento criptografico no Gate 2 via trustkit spec lock
    let mut lock_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let lock_assert = lock_cmd
        .current_dir(base_dir)
        .args(["spec", "lock", "--json"])
        .assert()
        .success()
        .code(0);

    let lock_out = String::from_utf8(lock_assert.get_output().stdout.clone()).unwrap();
    assert!(!lock_out.contains(": null"));
    let lock_json: Value = serde_json::from_str(&lock_out).expect("json");
    assert_eq!(lock_json["status"], "SPEC_FROZEN");
    assert_eq!(lock_json["spec_lock_state"], "FROZEN");

    // 6. Criacao de tarefa atomica e planejamento de ondas no Gate 3 via trustkit wave plan
    let tasks_dir = spec_dir.join("tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");
    fs::write(
        tasks_dir.join("TASK-001.md"),
        "# TASK-001: Implementar Core Nativo\ndepends_on: []\nArquivos Esperados:\n- src/core.rs\n",
    )
    .expect("write task");

    let mut plan_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let plan_assert = plan_cmd
        .current_dir(base_dir)
        .args(["wave", "plan", "--json"])
        .assert()
        .success()
        .code(0);

    let plan_out = String::from_utf8(plan_assert.get_output().stdout.clone()).unwrap();
    assert!(!plan_out.contains(": null"));
    let plan_json: Value = serde_json::from_str(&plan_out).expect("json");
    assert_eq!(plan_json["status"], "WAVES_PLANNED");
    assert_eq!(plan_json["total_tasks"], 1);
}
