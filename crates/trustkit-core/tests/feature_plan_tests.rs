// Unit tests for Feature Domain and Planning Use Case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArchitecturalTaskLayer, FeaturePlan, FeatureStatus, FeatureTaskBlueprint, Milestone,
    MilestoneCapability, MilestoneStatus, NextStepPriority, NextStepResolver, PlannedFeature,
    SourceCitation, TechnicalConstraint,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{FeatureExtractorPort, FeatureRepositoryPort, MilestoneRepositoryPort};
use trustkit_core::use_cases::plan_features::{
    PlanFeaturesRequest, execute, render_feature_spec_markdown, render_features_index_markdown,
};

struct MockMilestoneRepo {
    milestone: Option<Milestone>,
}

impl MilestoneRepositoryPort for MockMilestoneRepo {
    fn save_milestone(&self, _milestone: &Milestone) -> Result<(), DomainViolation> {
        Ok(())
    }

    fn find_milestone_by_id(
        &self,
        milestone_id: &str,
    ) -> Result<Option<Milestone>, DomainViolation> {
        Ok(self
            .milestone
            .as_ref()
            .filter(|m| m.milestone_id == milestone_id)
            .cloned())
    }

    fn list_milestones(&self, _plan_id: Option<&str>) -> Result<Vec<Milestone>, DomainViolation> {
        Ok(self.milestone.iter().cloned().collect())
    }

    fn update_milestone_status(
        &self,
        _milestone_id: &str,
        _status: MilestoneStatus,
        _updated_at: &str,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

struct MockFeatureExtractor {
    plan_to_return: Result<FeaturePlan, DomainViolation>,
}

impl FeatureExtractorPort for MockFeatureExtractor {
    fn extract_features_from_milestone(
        &self,
        _milestone: &Milestone,
        _output_dir: &str,
        _now_iso: &str,
    ) -> Result<FeaturePlan, DomainViolation> {
        self.plan_to_return.clone()
    }
}

struct MockFeatureRepo {
    saved: Mutex<Vec<PlannedFeature>>,
}

impl FeatureRepositoryPort for MockFeatureRepo {
    fn save_feature(&self, feature: &PlannedFeature) -> Result<(), DomainViolation> {
        self.saved.lock().unwrap().push(feature.clone());
        Ok(())
    }

    fn find_feature_by_id(
        &self,
        feature_id: &str,
    ) -> Result<Option<PlannedFeature>, DomainViolation> {
        Ok(self
            .saved
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.feature_id == feature_id)
            .cloned())
    }

    fn list_features(
        &self,
        milestone_id: Option<&str>,
    ) -> Result<Vec<PlannedFeature>, DomainViolation> {
        let guard = self.saved.lock().unwrap();
        Ok(guard
            .iter()
            .filter(|f| milestone_id.map(|m| f.milestone_id == m).unwrap_or(true))
            .cloned()
            .collect())
    }

    fn update_feature_status(
        &self,
        feature_id: &str,
        status: FeatureStatus,
        linked_cycle_id: &str,
        _updated_at: &str,
    ) -> Result<(), DomainViolation> {
        let mut guard = self.saved.lock().unwrap();
        if let Some(f) = guard.iter_mut().find(|f| f.feature_id == feature_id) {
            f.status = status;
            f.linked_cycle_id = linked_cycle_id.to_string();
        }
        Ok(())
    }
}

fn sample_milestone() -> Milestone {
    Milestone {
        milestone_id: "M01".to_string(),
        plan_id: "plan-prd-adr".to_string(),
        sequence_index: 1,
        title: "Pagamento Basico".to_string(),
        slug: "pagamento-basico".to_string(),
        file_path: "docs/milestones/M01-pagamento-basico.md".to_string(),
        goal: "Entregar fluxo base de pagamento".to_string(),
        dependencies: vec![],
        capabilities: vec![MilestoneCapability::new(
            "CAP-001",
            "Autorizacao de Cartao",
            "Processar cobranca com validacao sincrona",
            SourceCitation::new("PRD.md", 10, 20, "Pagamentos"),
        )],
        technical_constraints: vec![TechnicalConstraint::new(
            "ADR-001",
            "Idempotencia Transacional",
            "Chave idempotente obrigatoria em toda requisicao",
            SourceCitation::new("ADR.md", 15, 25, "Resiliencia"),
        )],
        status: MilestoneStatus::Planned,
        created_at: "2026-11-25T00:00:00Z".to_string(),
        updated_at: "2026-11-25T00:00:00Z".to_string(),
    }
}

fn sample_feature_plan(output_dir: &str) -> FeaturePlan {
    let m = sample_milestone();
    let spec_dir = format!("{output_dir}/M01/M01-F01-caminho-feliz");
    let spec_file = format!("{spec_dir}/spec.md");
    let feature = PlannedFeature {
        feature_id: "M01-F01".to_string(),
        milestone_id: "M01".to_string(),
        sequence_index: 1,
        title: "Caminho Feliz: Autorizacao de Cartao".to_string(),
        slug: "caminho-feliz".to_string(),
        spec_dir,
        spec_file_path: spec_file,
        goal: "Implementar fluxo nominal de autorizacao".to_string(),
        dependencies: vec![],
        capabilities: m.capabilities.clone(),
        technical_constraints: m.technical_constraints.clone(),
        task_blueprints: vec![
            FeatureTaskBlueprint {
                task_id: "TASK-001".to_string(),
                layer: ArchitecturalTaskLayer::SchemaAndContract,
                title: "Schema e Contratos de Autorizacao".to_string(),
                business_rule_id: "RN-001".to_string(),
                functional_req_id: "RF-001".to_string(),
                mermaid_node: "TASK_001".to_string(),
                target_test_file: "tests/schema_tests.rs".to_string(),
                evidence_artifact: "schema_verified".to_string(),
                depends_on: vec![],
            },
            FeatureTaskBlueprint {
                task_id: "TASK-002".to_string(),
                layer: ArchitecturalTaskLayer::CoreAndDomain,
                title: "Dominio e Regras de Negocio".to_string(),
                business_rule_id: "RN-001".to_string(),
                functional_req_id: "RF-001".to_string(),
                mermaid_node: "TASK_002".to_string(),
                target_test_file: "tests/domain_tests.rs".to_string(),
                evidence_artifact: "domain_verified".to_string(),
                depends_on: vec!["TASK-001".to_string()],
            },
        ],
        status: FeatureStatus::Planned,
        linked_cycle_id: "UNASSIGNED".to_string(),
        created_at: "2026-11-25T00:00:00Z".to_string(),
        updated_at: "2026-11-25T00:00:00Z".to_string(),
    };

    FeaturePlan {
        milestone_id: "M01".to_string(),
        milestone_title: m.title,
        features: vec![feature],
        created_at: "2026-11-25T00:00:00Z".to_string(),
    }
}

#[test]
fn test_feature_status_and_layer_parsing() {
    assert_eq!(
        "Planned".parse::<FeatureStatus>().unwrap(),
        FeatureStatus::Planned
    );
    assert_eq!(
        "in_progress".parse::<FeatureStatus>().unwrap(),
        FeatureStatus::InProgress
    );
    assert_eq!(
        "completed".parse::<FeatureStatus>().unwrap(),
        FeatureStatus::Completed
    );
    assert_eq!(
        ArchitecturalTaskLayer::SchemaAndContract.as_str(),
        "SCHEMA_AND_CONTRACT"
    );
    assert_eq!(
        ArchitecturalTaskLayer::CoreAndDomain.as_str(),
        "CORE_AND_DOMAIN"
    );
    assert_eq!(
        ArchitecturalTaskLayer::ApiUiAndAdapters.as_str(),
        "API_UI_AND_ADAPTERS"
    );
    assert_eq!(
        ArchitecturalTaskLayer::TelemetryAuditAndE2e.as_str(),
        "TELEMETRY_AUDIT_AND_E2E"
    );
}

#[test]
fn test_render_feature_spec_markdown_has_traceability_and_mermaid() {
    let plan = sample_feature_plan("docs/features");
    let md = render_feature_spec_markdown(&plan.features[0]);
    assert!(md.contains("```mermaid"));
    assert!(
        md.contains(
            "TASK_001[\"Schema e Contratos de Autorizacao\"] --> TASK_002[\"Dominio e Regras de Negocio\"]"
        )
    );
    assert!(md.contains("## 4. Matriz de Rastreabilidade"));
    assert!(md.contains("| RN-001 | RF-001 | TASK-001 | TASK_001 | tests/schema_tests.rs |"));
    assert!(md.contains("## 3. Criterios de Aceitacao (Gherkin)"));

    let idx = render_features_index_markdown(&plan);
    assert!(idx.contains("M01-F01"));
}

#[test]
fn test_plan_features_execute_persists_and_writes_files() {
    let tmp = std::env::temp_dir().join("trustkit_core_feature_plan_test");
    let _ = std::fs::remove_dir_all(&tmp);
    let out_dir = tmp.to_string_lossy().to_string();

    let m_repo = MockMilestoneRepo {
        milestone: Some(sample_milestone()),
    };
    let extractor = MockFeatureExtractor {
        plan_to_return: Ok(sample_feature_plan(&out_dir)),
    };
    let f_repo = MockFeatureRepo {
        saved: Mutex::new(Vec::new()),
    };

    let req = PlanFeaturesRequest {
        milestone_ref: "M01".to_string(),
        output_dir: out_dir.clone(),
        now_iso: "2026-11-25T00:00:00Z".to_string(),
    };

    let outcome = execute(req, &m_repo, &extractor, Some(&f_repo)).expect("plan features");
    assert_eq!(outcome.plan.features.len(), 1);
    assert_eq!(f_repo.list_features(Some("M01")).unwrap().len(), 1);
    assert!(std::path::Path::new(&outcome.plan.features[0].spec_file_path).exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_next_step_resolver_links_milestones_to_features_to_auto() {
    let ms_steps =
        NextStepResolver::for_milestones_plan(Some("docs/milestones/M01.md"), Some("M01"));
    assert_eq!(ms_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(
        ms_steps[0].command,
        "trustkit features plan --milestone M01"
    );

    let feat_steps = NextStepResolver::for_features_plan(Some("M01-F01"), "M01");
    assert_eq!(feat_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(feat_steps[0].command, "trustkit auto --feature M01-F01");
    assert_eq!(feat_steps[1].command, "trustkit auto --milestone M01");

    let inspect_steps = NextStepResolver::for_feature_inspect(
        "M01-F01",
        "docs/features/M01/M01-F01-caminho-feliz/spec.md",
    );
    assert_eq!(inspect_steps[0].command, "trustkit auto --feature M01-F01");
}

#[test]
fn test_red_verifier_rejects_missing_manifest_and_parses_test_command() {
    use trustkit_core::entities::parse_harness_execution_policy;
    use trustkit_core::use_cases::record_tdd_phase::RedVerifier;

    let res_cargo = RedVerifier::verify(
        "TASK-001",
        101,
        "",
        "error: could not find `Cargo.toml` in `/tmp/ts-repo`",
    );
    assert!(matches!(
        res_cargo,
        Err(DomainViolation::CompilationFailureInRedPhaseFault { .. })
    ));

    let res_pnpm = RedVerifier::verify(
        "TASK-001",
        1,
        "",
        "ERR_PNPM_NO_SCRIPT  Missing script: test",
    );
    assert!(matches!(
        res_pnpm,
        Err(DomainViolation::CompilationFailureInRedPhaseFault { .. })
    ));

    let cfg = parse_harness_execution_policy(
        "test_command = \"pnpm test\"\ncheck_command = \"pnpm typecheck\"\n",
    )
    .expect("parse policy");
    assert_eq!(cfg.test_command, "pnpm test");
    assert_eq!(cfg.check_command, "pnpm typecheck");
}
