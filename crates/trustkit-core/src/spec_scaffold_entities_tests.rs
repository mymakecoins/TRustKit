use super::*;

#[test]
fn test_spec_prompt_request_validation_nominal() {
    let req =
        SpecPromptRequest::new("Nova funcionalidade de teste", 2, 4000).expect("valid request");
    assert_eq!(req.idea, "Nova funcionalidade de teste");
    assert_eq!(req.context_hops, 2);
    assert_eq!(req.token_budget, 4000);
}

#[test]
fn test_spec_prompt_request_validation_rejects_empty_idea() {
    let err = SpecPromptRequest::new("   ", 2, 4000).unwrap_err();
    match err {
        DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id,
            reason,
            ..
        } => {
            assert_eq!(invariant_id, "RN-001");
            assert!(reason.contains("idea cannot be empty"));
        }
        other => panic!("Expected AstIntegrityOrInvariantViolation, got {other:?}"),
    }
}

#[test]
fn test_spec_prompt_request_validation_rejects_zero_token_budget() {
    let err = SpecPromptRequest::new("Ideia valida", 2, 0).unwrap_err();
    match err {
        DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id,
            reason,
            ..
        } => {
            assert_eq!(invariant_id, "RN-001");
            assert!(reason.contains("token_budget must be greater than zero"));
        }
        other => panic!("Expected AstIntegrityOrInvariantViolation, got {other:?}"),
    }
}

#[test]
fn test_spec_scaffold_template_contains_all_mandatory_sections() {
    let skeleton = SpecScaffoldTemplate::generate_markdown_skeleton(
        "009-sample-feature",
        "Minha ideia inovadora",
    );
    assert!(skeleton.contains("# Especificacao: 009-sample-feature"));
    assert!(skeleton.contains("## 1. Product Trio"));
    assert!(skeleton.contains("## 2. Diagrama de Fluxo (Mermaid)"));
    assert!(skeleton.contains("flowchart LR"));
    assert!(skeleton.contains("## 3. Criterios de Aceitacao (Gherkin)"));
    assert!(skeleton.contains("## 4. Matriz de Rastreabilidade"));
    assert!(skeleton.contains("Minha ideia inovadora"));
}

#[test]
fn test_spec_scaffold_system_instructions_and_token_estimation() {
    let instructions = SpecScaffoldTemplate::build_system_instructions();
    assert!(instructions.contains("Proibicao absoluta de emojis"));
    assert!(instructions.contains("Proibicao absoluta de trailers de autoria IA"));

    let est = SpecScaffoldTemplate::estimate_tokens("12345678");
    assert_eq!(est, 2);
}

#[test]
fn test_design_it_twice_briefs_generates_three_contrasting_architectures_without_null() {
    let req = SpecPromptRequest::new("Motor de indexacao incremental de grafos", 2, 4000)
        .expect("valid request")
        .with_design_it_twice(true);
    assert!(req.design_it_twice);

    let briefs = SpecScaffoldTemplate::generate_design_it_twice_briefs(&req.idea);
    assert_eq!(briefs.len(), 3);

    let strategies: Vec<&str> = briefs.iter().map(|b| b.strategy.as_str()).collect();
    assert_eq!(
        strategies,
        vec![
            "Minimizacao de Interface",
            "Maxima Flexibilidade",
            "Otimizacao para o Chamador Comum",
        ]
    );

    for brief in &briefs {
        assert!(!brief.brief_id.is_empty());
        assert!(!brief.strategy_name.is_empty());
        assert!(!brief.design_constraint.is_empty());
        assert!(!brief.target_seam_focus.is_empty());
        assert!(brief.subagent_prompt.contains(&req.idea));
        assert!(!brief.expected_tradeoffs.is_empty());
        let json = brief.to_json();
        assert!(!json.contains("null"));
        assert!(json.contains(&brief.brief_id));
        assert!(json.contains(brief.strategy.as_str()));
    }
}
