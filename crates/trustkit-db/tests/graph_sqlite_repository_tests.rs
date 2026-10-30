// Integration tests for GraphSqliteRepository, BLAKE3 incremental manifest and Shrink-Guard (TASK-003, RF-027, RN-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.

use tempfile::NamedTempFile;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, FileGraphManifest, GraphHealthVerdict, GraphNodeKind,
    StructuralEdge, StructuralNode,
};
use trustkit_core::ports::StructuralGraphRepositoryPort;
use trustkit_db::GraphSqliteRepository;

#[test]
fn test_shrink_guard_blocks_accidental_node_reduction_without_force() {
    let tmp = NamedTempFile::new().expect("Failed creating temp db file");
    let repo = GraphSqliteRepository::open(tmp.path()).expect("Failed opening GraphSqliteRepository");

    // 1. Initial snapshot with 50 nodes
    let mut initial_nodes = Vec::new();
    let mut initial_edges = Vec::new();
    for i in 0..50 {
        let node_id = format!("crates/core/src/file_{i}.rs");
        initial_nodes.push(StructuralNode::new(
            &node_id,
            format!("file_{i}.rs"),
            GraphNodeKind::CodeFile,
            &node_id,
            "CORE",
        ));
        if i > 0 {
            initial_edges.push(StructuralEdge::new(
                &node_id,
                "crates/core/src/file_0.rs",
                EdgeRelationKind::Imports,
                EdgeConfidence::Extracted,
                &node_id,
            ));
        }
    }

    let manifests = vec![FileGraphManifest::new(
        "crates/core/src/file_0.rs",
        "hash_file_0",
        50,
        "2026-10-29T22:00:00Z",
    )];

    let verdict = repo.save_graph_snapshot(&initial_nodes, &initial_edges, &manifests, false)
        .expect("Initial snapshot save must succeed");
    assert_eq!(verdict, GraphHealthVerdict::Healthy);

    let loaded = repo.load_nodes().expect("Must load nodes");
    assert_eq!(loaded.len(), 50);

    // 2. Attempt to save reduced graph of 10 nodes without force -> ShrinkBlocked!
    let reduced_nodes: Vec<StructuralNode> = initial_nodes[0..10].to_vec();
    let reduced_edges: Vec<StructuralEdge> = initial_edges[0..9].to_vec();

    let shrink_verdict = repo.save_graph_snapshot(&reduced_nodes, &reduced_edges, &manifests, false)
        .expect("Shrink-Guard must return verdict without failing the call");

    assert_eq!(shrink_verdict, GraphHealthVerdict::ShrinkBlocked, "Shrink-Guard must trigger when reducing 50 -> 10 nodes");

    // Verify database nodes remained intact at 50
    let persisted_nodes = repo.load_nodes().expect("Must load nodes after shrink attempt");
    assert_eq!(persisted_nodes.len(), 50, "Node count in SQLite must remain unchanged at 50");

    // 3. With force=true, the reduction must be permitted
    let force_verdict = repo.save_graph_snapshot(&reduced_nodes, &reduced_edges, &manifests, true)
        .expect("Forced snapshot save must succeed");
    assert_eq!(force_verdict, GraphHealthVerdict::Healthy);

    let forced_nodes = repo.load_nodes().expect("Must load nodes after force");
    assert_eq!(forced_nodes.len(), 10, "Nodes must be updated to 10 after --force");
}

#[test]
fn test_detect_changed_files_with_blake3_digests() {
    let tmp = NamedTempFile::new().expect("Failed creating temp db file");
    let repo = GraphSqliteRepository::open(tmp.path()).expect("Failed opening GraphSqliteRepository");

    // Seed 20 files in manifest
    let mut initial_manifests = Vec::new();
    for i in 0..20 {
        initial_manifests.push(FileGraphManifest::new(
            format!("file_{i}.rs"),
            format!("digest_initial_{i}"),
            1,
            "2026-10-29T22:00:00Z",
        ));
    }

    let initial_nodes = vec![StructuralNode::new(
        "file_0.rs",
        "file_0.rs",
        GraphNodeKind::CodeFile,
        "file_0.rs",
        "CORE",
    )];
    repo.save_graph_snapshot(&initial_nodes, &[], &initial_manifests, false)
        .expect("Must save initial manifests");

    // Candidate scan of 20 files, but exactly 2 files (file_3.rs and file_17.rs) have altered hashes
    let mut current_manifests = Vec::new();
    for i in 0..20 {
        let digest = if i == 3 || i == 17 {
            format!("digest_modified_{i}")
        } else {
            format!("digest_initial_{i}")
        };
        current_manifests.push(FileGraphManifest::new(
            format!("file_{i}.rs"),
            digest,
            1,
            "2026-10-29T22:05:00Z",
        ));
    }

    let changed = repo.detect_changed_files(&current_manifests)
        .expect("detect_changed_files must succeed");

    assert_eq!(changed.len(), 2, "Only 2 files must be detected as changed");
    assert!(changed.contains(&"file_3.rs".to_string()));
    assert!(changed.contains(&"file_17.rs".to_string()));
}

#[test]
fn test_roundtrip_nodes_edges_and_manifests_preserves_not_null_metadata() {
    let tmp = NamedTempFile::new().expect("Failed creating temp db file");
    let repo = GraphSqliteRepository::open(tmp.path()).expect("Failed opening repo");

    let nodes = vec![
        StructuralNode::new("crates/core/lib.rs", "lib.rs", GraphNodeKind::CodeFile, "crates/core/lib.rs", "CORE"),
        StructuralNode::new("crates/core/lib.rs::MyStruct", "MyStruct", GraphNodeKind::PublicSymbol, "crates/core/lib.rs", "CORE"),
    ];

    let edges = vec![
        StructuralEdge::new("crates/core/lib.rs", "crates/core/lib.rs::MyStruct", EdgeRelationKind::DefinesSymbol, EdgeConfidence::Extracted, "crates/core/lib.rs"),
        StructuralEdge::new("crates/core/lib.rs", "std::sync::Arc", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/core/lib.rs"),
    ];

    let manifests = vec![
        FileGraphManifest::new("crates/core/lib.rs", "blake3_lib_digest", 2, "2026-10-29T22:00:00Z"),
    ];

    let verdict = repo.save_graph_snapshot(&nodes, &edges, &manifests, false).expect("Save must succeed");
    assert_eq!(verdict, GraphHealthVerdict::Healthy);

    let loaded_nodes = repo.load_nodes().expect("Load nodes");
    assert_eq!(loaded_nodes.len(), 2);
    assert_eq!(loaded_nodes[1].label, "MyStruct");

    let loaded_edges = repo.load_edges().expect("Load edges");
    assert_eq!(loaded_edges.len(), 2);

    let loaded_manifests = repo.load_manifests().expect("Load manifests");
    assert_eq!(loaded_manifests.len(), 1);
    assert_eq!(loaded_manifests[0].file_path, "crates/core/lib.rs");
    assert_eq!(loaded_manifests[0].blake3_digest, "blake3_lib_digest");
}
