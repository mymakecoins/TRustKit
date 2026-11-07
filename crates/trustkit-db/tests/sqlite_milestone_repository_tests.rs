// Integration tests for SQLite WAL Milestone repository (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_core::entities::{
    Milestone, MilestoneCapability, MilestonePlan, MilestoneStatus, SourceCitation,
    TechnicalConstraint,
};
use trustkit_core::ports::MilestoneRepositoryPort;
use trustkit_db::SqliteWalRepositoryAdapter;

fn make_sample_milestone(
    id: &str,
    plan_id: &str,
    seq: usize,
    status: MilestoneStatus,
) -> Milestone {
    let cap = MilestoneCapability::new(
        format!("CAP-{id}"),
        "Capacidade de Teste",
        "Descricao da capacidade",
        SourceCitation::new("docs/PRD.md", 10, 20, "Requisitos"),
    );
    let con = TechnicalConstraint::new(
        format!("CON-{id}"),
        "Restricao de Teste",
        "Descricao da restricao",
        SourceCitation::new("docs/ADR.md", 30, 40, "Arquitetura"),
    );

    Milestone {
        milestone_id: id.to_string(),
        plan_id: plan_id.to_string(),
        sequence_index: seq,
        title: format!("Marco {id}"),
        slug: format!("marco-{id}"),
        file_path: format!("docs/milestones/{id}.md"),
        goal: format!("Objetivo do marco {id}"),
        dependencies: vec!["M00".to_string()],
        capabilities: vec![cap],
        technical_constraints: vec![con],
        status,
        created_at: "2026-11-06T12:00:00Z".to_string(),
        updated_at: "2026-11-06T12:00:00Z".to_string(),
    }
}

#[test]
fn test_save_and_find_milestone_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open adapter");

    let m1 = make_sample_milestone("M01", "PLAN-A", 1, MilestoneStatus::Planned);
    repo.save_milestone(&m1).expect("save milestone");

    let found = repo
        .find_milestone_by_id("M01")
        .expect("find query")
        .expect("should find milestone");

    assert_eq!(found.milestone_id, "M01");
    assert_eq!(found.plan_id, "PLAN-A");
    assert_eq!(found.sequence_index, 1);
    assert_eq!(found.title, "Marco M01");
    assert_eq!(found.status, MilestoneStatus::Planned);
    assert_eq!(found.dependencies, vec!["M00"]);
    assert_eq!(found.capabilities.len(), 1);
    assert_eq!(found.capabilities[0].capability_id, "CAP-M01");
    assert_eq!(found.capabilities[0].citation.start_line, 10);
    assert_eq!(found.technical_constraints.len(), 1);
    assert_eq!(found.technical_constraints[0].constraint_id, "CON-M01");

    let not_found = repo.find_milestone_by_id("M99").expect("query");
    assert!(not_found.is_none());
}

#[test]
fn test_list_milestones_with_and_without_filter() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open adapter");

    let m1 = make_sample_milestone("M01", "PLAN-1", 1, MilestoneStatus::Planned);
    let m2 = make_sample_milestone("M02", "PLAN-1", 2, MilestoneStatus::Planned);
    let m3 = make_sample_milestone("M03", "PLAN-2", 1, MilestoneStatus::Planned);

    repo.save_milestone(&m1).expect("save 1");
    repo.save_milestone(&m2).expect("save 2");
    repo.save_milestone(&m3).expect("save 3");

    let all = repo.list_milestones(None).expect("list all");
    assert_eq!(all.len(), 3);

    let plan1 = repo.list_milestones(Some("PLAN-1")).expect("list plan 1");
    assert_eq!(plan1.len(), 2);
    assert_eq!(plan1[0].milestone_id, "M01");
    assert_eq!(plan1[1].milestone_id, "M02");

    let plan2 = repo.list_milestones(Some("PLAN-2")).expect("list plan 2");
    assert_eq!(plan2.len(), 1);
    assert_eq!(plan2[0].milestone_id, "M03");
}

#[test]
fn test_update_milestone_status() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open adapter");

    let m1 = make_sample_milestone("M01", "PLAN-X", 1, MilestoneStatus::Planned);
    repo.save_milestone(&m1).expect("save");

    repo.update_milestone_status("M01", MilestoneStatus::InProgress, "2026-11-06T14:00:00Z")
        .expect("update status");

    let updated = repo
        .find_milestone_by_id("M01")
        .expect("find")
        .expect("must exist");
    assert_eq!(updated.status, MilestoneStatus::InProgress);
    assert_eq!(updated.updated_at, "2026-11-06T14:00:00Z");

    // Updating non-existent milestone should return error
    let err = repo.update_milestone_status(
        "M_UNKNOWN",
        MilestoneStatus::Completed,
        "2026-11-06T14:05:00Z",
    );
    assert!(err.is_err());
}

#[test]
fn test_save_milestone_plan() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open adapter");

    let m1 = make_sample_milestone("M01", "PLAN-BATCH", 1, MilestoneStatus::Planned);
    let m2 = make_sample_milestone("M02", "PLAN-BATCH", 2, MilestoneStatus::Planned);

    let plan = MilestonePlan {
        plan_id: "PLAN-BATCH".to_string(),
        prd_path: "docs/PRD.md".to_string(),
        adr_path: "docs/ADR.md".to_string(),
        milestones: vec![m1, m2],
        created_at: "2026-11-06T10:00:00Z".to_string(),
    };

    repo.save_milestone_plan(&plan).expect("save plan");

    let stored = repo
        .list_milestones(Some("PLAN-BATCH"))
        .expect("list batch");
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].milestone_id, "M01");
    assert_eq!(stored[1].milestone_id, "M02");
}
