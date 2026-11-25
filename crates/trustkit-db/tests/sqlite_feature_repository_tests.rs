// Integration tests for SQLite WAL Feature Repository.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_core::entities::{
    ArchitecturalTaskLayer, FeatureStatus, FeatureTaskBlueprint, MilestoneCapability,
    PlannedFeature, SourceCitation, TechnicalConstraint,
};
use trustkit_core::ports::FeatureRepositoryPort;
use trustkit_db::SqliteWalRepositoryAdapter;

fn sample_feature(feature_id: &str, milestone_id: &str, seq: usize) -> PlannedFeature {
    PlannedFeature {
        feature_id: feature_id.to_string(),
        milestone_id: milestone_id.to_string(),
        sequence_index: seq,
        title: format!("Feature {feature_id}"),
        slug: format!("feature-{}", feature_id.to_lowercase()),
        spec_dir: format!("docs/features/{milestone_id}/{feature_id}"),
        spec_file_path: format!("docs/features/{milestone_id}/{feature_id}/spec.md"),
        goal: "Validar persistencia de feature no SQLite WAL".to_string(),
        dependencies: vec![],
        capabilities: vec![MilestoneCapability::new(
            "CAP-001",
            "Capacidade Teste",
            "Descricao da capacidade",
            SourceCitation::new("PRD.md", 1, 10, "Escopo"),
        )],
        technical_constraints: vec![TechnicalConstraint::new(
            "ADR-001",
            "Restricao Teste",
            "Descricao da restricao",
            SourceCitation::new("ADR.md", 5, 15, "Decisao"),
        )],
        task_blueprints: vec![FeatureTaskBlueprint {
            task_id: "TASK-001".to_string(),
            layer: ArchitecturalTaskLayer::SchemaAndContract,
            title: "Schema e Contrato".to_string(),
            business_rule_id: "RN-001".to_string(),
            functional_req_id: "RF-001".to_string(),
            mermaid_node: "TASK_001".to_string(),
            target_test_file: "tests/schema_tests.rs".to_string(),
            evidence_artifact: "schema_pass".to_string(),
            depends_on: vec![],
        }],
        status: FeatureStatus::Planned,
        linked_cycle_id: "UNASSIGNED".to_string(),
        created_at: "2026-11-25T00:00:00Z".to_string(),
        updated_at: "2026-11-25T00:00:00Z".to_string(),
    }
}

#[test]
fn test_sqlite_feature_repository_crud_and_status_transition() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open sqlite wal");

    let f1 = sample_feature("M01-F01", "M01", 1);
    let f2 = sample_feature("M01-F02", "M01", 2);
    let f3 = sample_feature("M02-F01", "M02", 1);

    repo.save_feature(&f1).expect("save f1");
    repo.save_feature(&f2).expect("save f2");
    repo.save_feature(&f3).expect("save f3");

    let loaded = repo
        .find_feature_by_id("M01-F01")
        .expect("query f1")
        .expect("f1 exists");
    assert_eq!(loaded.title, "Feature M01-F01");
    assert_eq!(loaded.capabilities.len(), 1);
    assert_eq!(loaded.technical_constraints.len(), 1);
    assert_eq!(loaded.task_blueprints.len(), 1);
    assert_eq!(
        loaded.task_blueprints[0].layer,
        ArchitecturalTaskLayer::SchemaAndContract
    );

    let m01_features = repo.list_features(Some("M01")).expect("list M01 features");
    assert_eq!(m01_features.len(), 2);

    let all_features = repo.list_features(None).expect("list all features");
    assert_eq!(all_features.len(), 3);

    repo.update_feature_status(
        "M01-F01",
        FeatureStatus::Completed,
        "001-m01-f01",
        "2026-11-25T01:00:00Z",
    )
    .expect("update status");

    let updated = repo
        .find_feature_by_id("M01-F01")
        .unwrap()
        .expect("updated f1");
    assert_eq!(updated.status, FeatureStatus::Completed);
    assert_eq!(updated.linked_cycle_id, "001-m01-f01");

    // Re-planning (saving as Planned) must preserve Completed status and linked_cycle_id
    let replanned = sample_feature("M01-F01", "M01", 1);
    repo.save_feature(&replanned)
        .expect("idempotent re-plan save");
    let preserved = repo.find_feature_by_id("M01-F01").unwrap().unwrap();
    assert_eq!(preserved.status, FeatureStatus::Completed);
    assert_eq!(preserved.linked_cycle_id, "001-m01-f01");
}
