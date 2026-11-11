// Integration tests for TASK-001: SpecMatrixAtomizer.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::time::Instant;
use trustkit_ast::SpecMatrixAtomizer;
use trustkit_core::entities::AtomizationSource;
use trustkit_core::ports::TaskAtomizerPort;

const SAMPLE_SPEC_MARKDOWN: &str = r#"# Especificacao: autonomous-task-atomization

## 1. Product Trio

- **PM**: Garantir atomizacao deterministica de especificacoes formais.
- **Tech Lead**: Extrair da Matriz de Rastreabilidade (secao 4), Mermaid (secao 2) e Gherkin (secao 3).
- **Design / QA**: Validar extracao sem campos vazios em menos de 50ms.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    AstMatrixParser[Extracao AST da Matriz Gherkin] --> TaskSynthesizer[Sintese de tasks/TASK-NNN.md no Disco]
    TaskSynthesizer --> SafeWorktreeMerge[Merge Acumulativo Seguro]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Atomizacao automatica nominal (Atendimento de RF-001 e RF-002)

- **Dado** uma especificacao formal valida contendo requisitos funcionais e cenarios Gherkin
- **Quando** o comando de atomizacao for executado sobre o documento
- **Entao** o motor deve decompor a entrada em tarefas atomicas ordenadas.

### Cenario 2: Merge acumulativo seguro (Atendimento de RF-008)

- **Dado** uma onda de execucao contendo multiplas tarefas atomicas
- **Quando** o provedor de worktrees realizar a integracao
- **Entao** todos os commits devem ser preservados sem force-reset.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | AstMatrixParser | crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs | EV-001-spec-matrix-ast-atomization.json |
| RN-002 | RF-002 | TASK-002 | TaskSynthesizer | crates/trustkit-orch/tests/task_synthesis_fs_tests.rs | EV-002-task-file-synthesis.json |
| RN-008 | RF-008 | TASK-008 | SafeWorktreeMerge | crates/trustkit-orch/tests/safe_worktree_merge_tests.rs | EV-008-safe-worktree-merge.json |
"#;

#[test]
fn test_atomize_markdown_extracts_ordered_tasks_under_50ms() {
    let start = Instant::now();
    let tasks = SpecMatrixAtomizer::atomize_markdown(SAMPLE_SPEC_MARKDOWN)
        .expect("should atomize valid spec markdown");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Atomization must complete in under 50ms, took {}ms",
        elapsed.as_millis()
    );
    assert_eq!(tasks.len(), 3, "Expected 3 atomic tasks from matrix");

    let t1 = &tasks[0];
    assert_eq!(t1.task_id, "TASK-001");
    assert_eq!(t1.business_rule_id, "RN-001");
    assert_eq!(t1.functional_req_id, "RF-001");
    assert_eq!(t1.mermaid_node, "AstMatrixParser");
    assert_eq!(t1.architectural_layer, "AST");
    assert_eq!(
        t1.target_test_file,
        "crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs"
    );
    assert!(!t1.expected_files.is_empty());
    assert!(t1.depends_on.is_empty());
    assert!(!t1.given_clause.is_empty());
    assert!(!t1.when_clause.is_empty());
    assert!(!t1.then_clause.is_empty());

    let t2 = &tasks[1];
    assert_eq!(t2.task_id, "TASK-002");
    assert_eq!(t2.architectural_layer, "ORCH");
    assert_eq!(t2.depends_on, vec!["TASK-001".to_string()]);

    let t3 = &tasks[2];
    assert_eq!(t3.task_id, "TASK-008");
    assert_eq!(t3.architectural_layer, "ORCH");
    assert_eq!(t3.depends_on, vec!["TASK-002".to_string()]);
}

#[test]
fn test_task_atomizer_port_extracts_real_project_spec() {
    let content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/specs/spec-autonomous-task-atomization.md"),
    )
    .expect("read project spec");

    let atomizer = SpecMatrixAtomizer::new();
    let source = AtomizationSource::SpecificationMarkdown {
        spec_path: "docs/specs/spec-autonomous-task-atomization.md".to_string(),
        markdown_content: content,
    };

    let drafts = atomizer
        .atomize_source(&source)
        .expect("atomize real specification");
    assert_eq!(drafts.len(), 9);
    for draft in &drafts {
        assert!(!draft.task_id.is_empty());
        assert!(!draft.title.is_empty());
        assert!(!draft.architectural_layer.is_empty());
        assert!(!draft.target_test_file.is_empty());
        assert!(!draft.expected_files.is_empty());
        assert!(!draft.given_clause.is_empty());
        assert!(!draft.when_clause.is_empty());
        assert!(!draft.then_clause.is_empty());
    }
}
