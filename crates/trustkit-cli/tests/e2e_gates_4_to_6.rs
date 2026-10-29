// E2E tests for Gates 4, 5, and 6 (TASK-017).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::tempdir;
use trustkit_core::entities::{
    CriticKind, CycleStatus, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

const VALID_SPEC: &str = r#"# Especificacao: Core Feature
## 1. Product Trio
- **PM**: Regras.
- **Tech Lead**: Arquitetura.
- **QA**: Criterios.
## 2. Diagrama de Fluxo (Mermaid)
```mermaid
flowchart TD
    A[Inicio] --> B[Fim]
```
## 3. Criterios de Aceitacao (Gherkin)
### Cenario 1: Execucao
- **Dado** ambiente pronto
- **Quando** executa tarefa
- **Entao** conclui com sucesso
## 4. Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Playwright | Screenshot |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/e2e/core.spec.ts | SC-001.png |
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
    fs::write(path.join("README.md"), "# Repo\n").expect("write readme");
    run(&["add", "README.md"]);
    run(&["commit", "-m", "chore: #45 initial commit"]);
    run(&["branch", "develop"]);
}

fn setup_mock_gh(dir: &Path) -> PathBuf {
    let bin_dir = dir.join("mock_bin");
    fs::create_dir_all(&bin_dir).expect("create mock bin dir");
    let gh_path = bin_dir.join("gh");
    let script = r#"#!/bin/sh
if [ "$1" = "auth" ]; then
    echo "Logged in to github.com account marcos-rodoind"
    exit 0
elif [ "$1" = "api" ]; then
    echo "marcos-rodoind"
    exit 0
elif [ "$1" = "pr" ] && [ "$2" = "create" ]; then
    echo "https://github.com/org/repo/pull/45"
    exit 0
elif [ "$1" = "issue" ]; then
    echo "https://github.com/org/repo/issues/45"
    exit 0
fi
exit 0
"#;
    fs::write(&gh_path, script).expect("write mock gh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&gh_path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&gh_path, perms).expect("set perms");
    }
    bin_dir
}

fn bootstrap_cycle_and_spec(base_dir: &Path) -> (String, PathBuf) {
    init_git_repo(base_dir);

    let mut init_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let init_assert = init_cmd
        .current_dir(base_dir)
        .args(["init", "--issue", "45", "--json"])
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(init_assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains(": null"));
    let json: Value = serde_json::from_str(&stdout).expect("json");
    let cycle_id = json["cycle_id"].as_str().expect("cycle_id").to_string();
    let spec_dir_rel = json["spec_dir"].as_str().expect("spec_dir");
    let spec_dir = base_dir.join(spec_dir_rel);

    fs::write(spec_dir.join("spec.md"), VALID_SPEC).expect("write spec");

    let mut lock_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    lock_cmd
        .current_dir(base_dir)
        .args(["spec", "lock", "--json"])
        .assert()
        .success()
        .code(0);

    let tasks_dir = spec_dir.join("tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");
    fs::write(
        tasks_dir.join("TASK-001.md"),
        "# TASK-001: Core Task\ndepends_on: []\nArquivos Esperados:\n- src/core.rs\n",
    )
    .expect("write task");

    let mut plan_cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    plan_cmd
        .current_dir(base_dir)
        .args(["wave", "plan", "--json"])
        .assert()
        .success()
        .code(0);

    (cycle_id, spec_dir)
}

#[test]
fn test_e2e_gate4_tdd_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    let (_cycle_id, spec_dir) = bootstrap_cycle_and_spec(base_dir);

    // 1. Tentar green sem red -> Exit Code 3
    let mut green_fraud = Command::cargo_bin("trustkit").expect("trustkit");
    let fraud_assert = green_fraud
        .current_dir(base_dir)
        .args(["task", "test", "TASK-001", "--phase", "green", "--json"])
        .assert()
        .failure()
        .code(3);
    let fraud_out = String::from_utf8(fraud_assert.get_output().stderr.clone()).unwrap();
    assert!(!fraud_out.contains(": null"));
    assert!(fraud_out.contains("TDD_LIFECYCLE_VIOLATION"));

    // 2. Executar red com falha de teste -> RedVerified (Exit Code 0)
    let mut red_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let red_assert = red_cmd
        .current_dir(base_dir)
        .args(["task", "test", "TASK-001", "--phase", "red", "--json"])
        .assert()
        .success()
        .code(0);
    let red_out = String::from_utf8(red_assert.get_output().stdout.clone()).unwrap();
    assert!(!red_out.contains(": null"));
    assert!(red_out.contains("RedVerified"));

    // 3. Preparar teste passando em spec_dir para green
    fs::write(
        spec_dir.join("Cargo.toml"),
        "[package]\nname = \"task_pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"lib.rs\"\n",
    )
    .expect("write cargo toml");
    fs::write(
        spec_dir.join("lib.rs"),
        "#[test]\nfn task_001_check() { assert_eq!(2 + 2, 4); }\n",
    )
    .expect("write lib");

    // 4. Executar green passando -> GreenVerified (Exit Code 0)
    let mut green_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let green_assert = green_cmd
        .current_dir(base_dir)
        .args(["task", "test", "TASK-001", "--phase", "green", "--json"])
        .assert()
        .success()
        .code(0);
    let green_out = String::from_utf8(green_assert.get_output().stdout.clone()).unwrap();
    assert!(!green_out.contains(": null"));
    assert!(green_out.contains("GreenVerified"));

    // 5. Concluir tarefa atômica
    let mut comp_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let comp_assert = comp_cmd
        .current_dir(base_dir)
        .args(["task", "complete", "TASK-001", "--json"])
        .assert()
        .success()
        .code(0);
    let comp_out = String::from_utf8(comp_assert.get_output().stdout.clone()).unwrap();
    assert!(!comp_out.contains(": null"));
    assert!(comp_out.contains("Committed"));
}

#[test]
fn test_e2e_gate4_wave_run_worktree_isolation() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    let (_cycle_id, _spec_dir) = bootstrap_cycle_and_spec(base_dir);

    let mut wave_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let wave_assert = wave_cmd
        .current_dir(base_dir)
        .args(["wave", "run", "--wave", "1", "--json"])
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(wave_assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains(": null"));
    let json: Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(json["verdict"], "PASS");
    assert!(stdout.contains("TASK-001"));

    // Verifica que worktree efêmera foi limpa após o merge
    let wt_path = base_dir.join(".trustkit/worktrees/TASK-001");
    assert!(!wt_path.exists());
}

#[test]
fn test_e2e_gate5_gauntlet_and_circuit_breaker() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    let (cycle_id, _spec_dir) = bootstrap_cycle_and_spec(base_dir);

    // 1. Gauntlet run sem Layer 0 -> Exit Code 5
    let mut g_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    g_cmd
        .current_dir(base_dir)
        .args(["gauntlet", "run", "--json"])
        .assert()
        .failure()
        .code(5);

    // 2. Executar ast-audit com sucesso
    let clean_file = base_dir.join("clean.rs");
    fs::write(&clean_file, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    let mut ast_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    ast_cmd
        .current_dir(base_dir)
        .args(["gauntlet", "ast-audit", "--files", clean_file.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .code(0);

    // 3. Simular 5 falhas no banco para disparar Circuit Breaker
    let db_path = base_dir.join(".trustkit/state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");
    repo.update_cycle_status(&cycle_id, CycleStatus::CircuitBreakerTripped, 5)
        .expect("update status");

    // 4. Invocacao do trustkit doctor com Circuit Breaker acionado -> Exit Code 6
    let mut doc_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let doc_assert = doc_cmd
        .current_dir(base_dir)
        .args(["doctor", "--json"])
        .assert()
        .failure()
        .code(6);
    let doc_out = String::from_utf8(doc_assert.get_output().stderr.clone()).unwrap();
    assert!(!doc_out.contains(": null"));
    assert!(doc_out.contains("CIRCUIT_BREAKER_TRIPPED"));
}

#[test]
fn test_e2e_gate6_done_merge_pr() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    let (cycle_id, spec_dir) = bootstrap_cycle_and_spec(base_dir);

    // Registrar evidencias aprovadas (Layer 0 + 4 Critics) no banco
    let db_path = base_dir.join(".trustkit/state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let ast_ev = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle_id.clone(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: 0,
        duration_ms: 10,
        sanitized_stdout_tail: "PASS".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "a".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T00:00:00Z".to_string(),
    };
    repo.append_evidence(&ast_ev).expect("append ast evidence");

    for kind in [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ] {
        let ev = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: cycle_id.clone(),
            gate_stage: GateStage::Gate5GauntletCleared,
            gauntlet_attempt: 0,
            actor_role: format!("CRITIC:{}", kind.as_str()),
            evidence_kind: EvidenceKind::CriticVerdict,
            executed_command: format!("critic {}", kind.as_str()),
            exit_code: 0,
            duration_ms: 12,
            sanitized_stdout_tail: "PASS".to_string(),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "b".repeat(64),
            verdict: GateVerdict::Pass,
            recorded_at: "2026-10-28T00:01:00Z".to_string(),
        };
        repo.append_evidence(&ev).expect("append critic");
    }

    // 1. Executar trustkit done -> Gera gauntlet-report.md
    let mut done_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let done_assert = done_cmd
        .current_dir(base_dir)
        .args(["done", "--json"])
        .assert()
        .success()
        .code(0);
    let done_out = String::from_utf8(done_assert.get_output().stdout.clone()).unwrap();
    assert!(!done_out.contains(": null"));
    let done_json: Value = serde_json::from_str(&done_out).expect("valid json");
    assert_eq!(done_json["verdict"], "PASS");

    let report_file = spec_dir.join("gauntlet-report.md");
    assert!(report_file.exists());
    let report_content = fs::read_to_string(&report_file).unwrap();
    assert!(report_content.contains("Gate 6 Certified"));

    // 2. Executar trustkit merge --target develop
    let mut merge_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let merge_assert = merge_cmd
        .current_dir(base_dir)
        .args(["merge", "--target", "develop", "--json"])
        .assert()
        .success()
        .code(0);
    let merge_out = String::from_utf8(merge_assert.get_output().stdout.clone()).unwrap();
    assert!(!merge_out.contains(": null"));
    let merge_json: Value = serde_json::from_str(&merge_out).expect("valid json");
    assert_eq!(merge_json["verdict"], "PASS");

    // 3. Executar trustkit pr --target develop (sob demanda)
    let mock_bin = setup_mock_gh(base_dir);
    let path_env = format!("{}:{}", mock_bin.display(), std::env::var("PATH").unwrap_or_default());

    let mut pr_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    let pr_assert = pr_cmd
        .current_dir(base_dir)
        .env("PATH", &path_env)
        .args(["pr", "--target", "develop", "--json"])
        .assert()
        .success()
        .code(0);
    let pr_out = String::from_utf8(pr_assert.get_output().stdout.clone()).unwrap();
    assert!(!pr_out.contains(": null"));
    let pr_json: Value = serde_json::from_str(&pr_out).expect("valid json");
    assert_eq!(pr_json["verdict"], "PASS");
    assert_eq!(pr_json["target_branch"], "develop");
}

#[test]
fn test_e2e_semantic_failure_exit_codes() {
    let dir = tempdir().expect("tempdir");
    let base_dir = dir.path();
    let (_cycle_id, _spec_dir) = bootstrap_cycle_and_spec(base_dir);

    // Merge em ciclo nao certificado -> Exit Code 1
    let mut merge_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    merge_cmd
        .current_dir(base_dir)
        .args(["merge", "--target", "develop", "--json"])
        .assert()
        .failure()
        .code(1);

    // Emojis proibidos no codigo -> Exit Code 4
    let bad_file = base_dir.join("bad.rs");
    fs::write(&bad_file, "// Contem emoji proibido: \u{1F680}\n").unwrap();
    let mut ast_cmd = Command::cargo_bin("trustkit").expect("trustkit");
    ast_cmd
        .current_dir(base_dir)
        .args(["gauntlet", "ast-audit", "--files", bad_file.to_str().unwrap(), "--json"])
        .assert()
        .failure()
        .code(4);
}
