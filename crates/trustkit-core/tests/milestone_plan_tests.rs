// Unit tests for Milestone Domain and Planning Use Case (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    Milestone, MilestoneCapability, MilestonePlan, MilestoneStatus, SourceCitation,
    TechnicalConstraint,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{MilestoneExtractorPort, MilestoneRepositoryPort};
use trustkit_core::use_cases::plan_milestones::{
    PlanMilestonesRequest, execute, render_index_markdown, render_milestone_markdown,
};

struct MockMilestoneExtractor {
    plan_to_return: Result<MilestonePlan, DomainViolation>,
}

impl MilestoneExtractorPort for MockMilestoneExtractor {
    fn extract_milestone_plan(
        &self,
        _prd_path: &str,
        _adr_path: &str,
    ) -> Result<MilestonePlan, DomainViolation> {
        self.plan_to_return.clone()
    }
}

struct MockMilestoneRepository {
    saved_milestones: Mutex<Vec<Milestone>>,
}

impl MilestoneRepositoryPort for MockMilestoneRepository {
    fn save_milestone(&self, milestone: &Milestone) -> Result<(), DomainViolation> {
        self.saved_milestones
            .lock()
            .unwrap()
            .push(milestone.clone());
        Ok(())
    }

    fn find_milestone_by_id(
        &self,
        milestone_id: &str,
    ) -> Result<Option<Milestone>, DomainViolation> {
        let guard = self.saved_milestones.lock().unwrap();
        Ok(guard
            .iter()
            .find(|m| m.milestone_id == milestone_id)
            .cloned())
    }

    fn list_milestones(&self, _plan_id: Option<&str>) -> Result<Vec<Milestone>, DomainViolation> {
        Ok(self.saved_milestones.lock().unwrap().clone())
    }

    fn update_milestone_status(
        &self,
        milestone_id: &str,
        status: MilestoneStatus,
        _updated_at: &str,
    ) -> Result<(), DomainViolation> {
        let mut guard = self.saved_milestones.lock().unwrap();
        if let Some(m) = guard.iter_mut().find(|m| m.milestone_id == milestone_id) {
            m.status = status;
        }
        Ok(())
    }
}

fn sample_valid_plan() -> MilestonePlan {
    let citation_prd = SourceCitation::new("docs/PRD_Sample.md", 10, 25, "1. Proposito");
    let citation_adr = SourceCitation::new("docs/ADR_Sample.md", 30, 45, "2. Decisao");

    let cap1 = MilestoneCapability::new(
        "CAP-001",
        "Autenticacao Basica",
        "Usuarios devem autenticar com login e senha",
        citation_prd,
    );

    let tc1 = TechnicalConstraint::new(
        "TC-001",
        "Rust Puro",
        "Core deve rodar exclusivamente com standard library",
        citation_adr,
    );

    let m1 = Milestone {
        milestone_id: "M01".to_string(),
        plan_id: "PLAN-001".to_string(),
        sequence_index: 1,
        title: "Fundacao e Core".to_string(),
        slug: "fundacao-e-core".to_string(),
        file_path: "docs/milestones/M01-fundacao-e-core.md".to_string(),
        goal: "Disponibilizar dominio e entidades isoladas".to_string(),
        dependencies: vec![],
        capabilities: vec![cap1],
        technical_constraints: vec![tc1],
        status: MilestoneStatus::Planned,
        created_at: "2026-11-06T00:00:00Z".to_string(),
        updated_at: "2026-11-06T00:00:00Z".to_string(),
    };

    MilestonePlan {
        plan_id: "PLAN-001".to_string(),
        prd_path: "docs/PRD_Sample.md".to_string(),
        adr_path: "docs/ADR_Sample.md".to_string(),
        milestones: vec![m1],
        created_at: "2026-11-06T00:00:00Z".to_string(),
    }
}

#[test]
fn test_source_citation_format() {
    let single = SourceCitation::new("docs/PRD.md", 42, 42, "Secao");
    assert_eq!(single.format_citation(), "docs/PRD.md#L42");

    let range = SourceCitation::new("docs/ADR.md", 10, 20, "Secao");
    assert_eq!(range.format_citation(), "docs/ADR.md#L10-L20");
}

#[test]
fn test_milestone_status_parse_and_display() {
    assert_eq!(
        "Planned".parse::<MilestoneStatus>().unwrap(),
        MilestoneStatus::Planned
    );
    assert_eq!(
        "in_progress".parse::<MilestoneStatus>().unwrap(),
        MilestoneStatus::InProgress
    );
    assert_eq!(
        "completed".parse::<MilestoneStatus>().unwrap(),
        MilestoneStatus::Completed
    );
    assert_eq!(
        "cancelled".parse::<MilestoneStatus>().unwrap(),
        MilestoneStatus::Cancelled
    );
    assert!("invalid".parse::<MilestoneStatus>().is_err());
}

#[test]
fn test_render_milestone_markdown_contains_expected_sections() {
    let plan = sample_valid_plan();
    let m = &plan.milestones[0];
    let md = render_milestone_markdown(m);

    assert!(md.contains("# M01: Fundacao e Core"));
    assert!(md.contains("## 1. Objetivo Verificavel"));
    assert!(md.contains("## 2. Capacidades de Negocio (PRD)"));
    assert!(md.contains("## 3. Restricoes Tecnicas e Decisoes (ADR)"));
    assert!(md.contains("## 4. Criterios de Conclusao Verificaveis"));
    assert!(md.contains("docs/PRD_Sample.md#L10-L25"));
    assert!(md.contains("docs/ADR_Sample.md#L30-L45"));
}

#[test]
fn test_render_index_markdown() {
    let plan = sample_valid_plan();
    let index = render_index_markdown(&plan);

    assert!(index.contains("# Indice de Milestones do Projeto"));
    assert!(index.contains("docs/PRD_Sample.md"));
    assert!(index.contains("docs/ADR_Sample.md"));
    assert!(index.contains("| M01 | Fundacao e Core | Planned |"));
}

#[test]
fn test_plan_milestones_execute_nominal() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("trustkit_plan_test_{now}"));
    let out_dir = temp_dir.to_str().unwrap().to_string();

    let plan = sample_valid_plan();
    let extractor = MockMilestoneExtractor {
        plan_to_return: Ok(plan),
    };

    let repo = MockMilestoneRepository {
        saved_milestones: Mutex::new(Vec::new()),
    };

    let req = PlanMilestonesRequest {
        prd_path: "docs/PRD_Sample.md".to_string(),
        adr_path: "docs/ADR_Sample.md".to_string(),
        output_dir: out_dir,
        now_iso: "2026-11-06T12:00:00Z".to_string(),
    };

    let outcome = execute(req, &extractor, Some(&repo)).expect("execution should succeed");
    assert_eq!(outcome.generated_artifacts.len(), 2); // 1 milestone + 1 index
    assert_eq!(repo.saved_milestones.lock().unwrap().len(), 1);

    // Verify written files
    let index_file = temp_dir.join("INDEX.md");
    assert!(index_file.exists());
    let m1_file = temp_dir.join("M01-fundacao-e-core.md");
    assert!(m1_file.exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plan_milestones_detects_self_dependency_cycle() {
    let mut plan = sample_valid_plan();
    plan.milestones[0].dependencies = vec!["M01".to_string()];

    let extractor = MockMilestoneExtractor {
        plan_to_return: Ok(plan),
    };

    let req = PlanMilestonesRequest {
        prd_path: "docs/PRD_Sample.md".to_string(),
        adr_path: "docs/ADR_Sample.md".to_string(),
        output_dir: String::new(),
        now_iso: "2026-11-06T12:00:00Z".to_string(),
    };

    let result = execute(req, &extractor, None);
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainViolation::CycleDetected { cycle_path, .. } => {
            assert_eq!(cycle_path, vec!["M01", "M01"]);
        }
        other => panic!("Expected CycleDetected error, got {:?}", other),
    }
}
