// Acceptance tests for GitHub issue body atomization (TASK-004, RN-004, RF-004).
// Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::AtomizationSource;
use trustkit_core::ports::TaskAtomizerPort;
use trustkit_gh::IssueBodyAtomizer;

#[test]
fn test_atomize_issue_with_traceability_table() {
    let issue_number = 42;
    let title = "Feature Spec Issue";
    let body = r#"
## Context
This is an automated issue for feature delivery.

### Criterios de Aceitacao (Gherkin)
- **Dado** o corpo textual de uma issue do GitHub
- **Quando** o IssueBodyAtomizer processar o payload
- **Entao** deve derivar as tarefas atomicas correspondentes mantendo rastreabilidade

### Matriz de Rastreabilidade
| Regra Negocio | Requisito Funcional | Tarefa | Componente | Teste | Evidencia |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | AstParser | crates/trustkit-ast/tests/parser_tests.rs | EV-001.json |
| RN-002 | RF-002 | TASK-002 | GhSync | crates/trustkit-gh/tests/sync_tests.rs | EV-002.json |
"#;

    let drafts =
        IssueBodyAtomizer::atomize_issue(issue_number, title, body).expect("atomize success");
    assert_eq!(drafts.len(), 2);

    let t1 = &drafts[0];
    assert_eq!(t1.task_id, "TASK-001");
    assert_eq!(t1.architectural_layer, "AST");
    assert_eq!(
        t1.target_test_file,
        "crates/trustkit-ast/tests/parser_tests.rs"
    );
    assert!(t1.context_summary.contains("Issue #42"));

    let t2 = &drafts[1];
    assert_eq!(t2.task_id, "TASK-002");
    assert_eq!(t2.architectural_layer, "GH");
    assert_eq!(
        t2.target_test_file,
        "crates/trustkit-gh/tests/sync_tests.rs"
    );
    assert!(t2.context_summary.contains("Issue #42"));
}

#[test]
fn test_atomize_issue_with_checklist() {
    let issue_number = 99;
    let title = "Checklist Issue";
    let body = r#"
## Tarefas
- [ ] TASK-010: Implement Core Logic
- [x] TASK-020: Implement Database Storage
"#;

    let drafts =
        IssueBodyAtomizer::atomize_issue(issue_number, title, body).expect("atomize success");
    assert_eq!(drafts.len(), 2);
    assert_eq!(drafts[0].task_id, "TASK-010");
    assert_eq!(drafts[0].title, "Implement Core Logic");
    assert!(drafts[0].context_summary.contains("Issue #99"));
    assert_eq!(drafts[1].task_id, "TASK-020");
    assert_eq!(drafts[1].title, "Implement Database Storage");
}

#[test]
fn test_atomize_issue_fallback_single_task() {
    let issue_number = 101;
    let title = "Quick fix bug in CLI";
    let body = "Please fix the argument parser bug.";

    let drafts =
        IssueBodyAtomizer::atomize_issue(issue_number, title, body).expect("atomize success");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].task_id, "TASK-001");
    assert_eq!(drafts[0].title, "Quick fix bug in CLI");
    assert!(drafts[0].context_summary.contains("Issue #101"));
    assert_eq!(drafts[0].business_rule_id, "RN-101");
    assert_eq!(drafts[0].functional_req_id, "RF-101");
}

#[test]
fn test_atomize_source_port_contract() {
    let atomizer = IssueBodyAtomizer::new();
    let source = AtomizationSource::GitHubIssueBody {
        issue_number: 777,
        title: "Integration Test Issue".to_string(),
        body_markdown: "- [ ] TASK-005: Verify port contract".to_string(),
    };

    let drafts = atomizer.atomize_source(&source).expect("atomize source");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].task_id, "TASK-005");
    assert!(drafts[0].context_summary.contains("Issue #777"));
}
