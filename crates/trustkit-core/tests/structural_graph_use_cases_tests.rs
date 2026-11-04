// Tests for Structural Graph Use Cases (TASK-007, RF-031, RN-007).
// Principle I: Strict Hexagonal Isolation. Principle II: Zero NULL.
// Principle III: Test-First (RED to GREEN). Principle VI: < 250 LOC.

use std::sync::Mutex;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, FileGraphManifest, GodNodeMetric, GraphHealthReport,
    GraphHealthVerdict, GraphNodeKind, KnowledgeDocument, KnowledgeLookupOutcome,
    KnowledgeRebuildSummary, KnowledgeSearchOutcome, KnowledgeSearchQuery, KnowledgeWriteOutcome,
    ShortestPathOutcome, StructuralEdge, StructuralNode, SurprisingEdge, TraversalMode,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    KnowledgeRepositoryPort, StructuralGraphAnalyzerPort, StructuralGraphRepositoryPort,
};
use trustkit_core::use_cases::structural_graph_use_cases::{
    BuildStructuralGraphRequest, BuildStructuralGraphUseCase, ExplainGraphNodeRequest,
    ExplainGraphNodeUseCase, ExplainNodeOutcome, FindShortestGraphPathRequest,
    FindShortestGraphPathUseCase,
};

struct InMemoryGraphRepo {
    nodes: Mutex<Vec<StructuralNode>>,
    edges: Mutex<Vec<StructuralEdge>>,
    manifests: Mutex<Vec<FileGraphManifest>>,
}

impl InMemoryGraphRepo {
    fn new() -> Self {
        Self {
            nodes: Mutex::new(Vec::new()),
            edges: Mutex::new(Vec::new()),
            manifests: Mutex::new(Vec::new()),
        }
    }
}

impl StructuralGraphRepositoryPort for InMemoryGraphRepo {
    fn detect_changed_files(
        &self,
        cur: &[FileGraphManifest],
    ) -> Result<Vec<String>, DomainViolation> {
        let saved = self.manifests.lock().unwrap();
        Ok(cur
            .iter()
            .filter(|c| {
                !saved
                    .iter()
                    .any(|s| s.file_path == c.file_path && s.blake3_digest == c.blake3_digest)
            })
            .map(|c| c.file_path.clone())
            .collect())
    }

    fn save_graph_snapshot(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        manifests: &[FileGraphManifest],
        force: bool,
    ) -> Result<GraphHealthVerdict, DomainViolation> {
        let mut n_lock = self.nodes.lock().unwrap();
        if !n_lock.is_empty() && nodes.len() < n_lock.len() && !force {
            return Ok(GraphHealthVerdict::ShrinkBlocked);
        }
        *n_lock = nodes.to_vec();
        *self.edges.lock().unwrap() = edges.to_vec();
        *self.manifests.lock().unwrap() = manifests.to_vec();
        Ok(GraphHealthVerdict::Healthy)
    }

    fn load_nodes(&self) -> Result<Vec<StructuralNode>, DomainViolation> {
        Ok(self.nodes.lock().unwrap().clone())
    }
    fn load_edges(&self) -> Result<Vec<StructuralEdge>, DomainViolation> {
        Ok(self.edges.lock().unwrap().clone())
    }
    fn load_manifests(&self) -> Result<Vec<FileGraphManifest>, DomainViolation> {
        Ok(self.manifests.lock().unwrap().clone())
    }
}

struct InMemoryKnowledgeRepo {
    docs: Mutex<Vec<KnowledgeDocument>>,
}
impl InMemoryKnowledgeRepo {
    fn new() -> Self {
        Self {
            docs: Mutex::new(Vec::new()),
        }
    }
}

impl KnowledgeRepositoryPort for InMemoryKnowledgeRepo {
    fn write_document(
        &self,
        doc: &KnowledgeDocument,
    ) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        self.docs.lock().unwrap().push(doc.clone());
        Ok(KnowledgeWriteOutcome::Success {
            slug: doc.slug.clone(),
            file_path: format!(".trustkit/wiki/{}.md", doc.slug),
            blake3_digest: "blake3".into(),
            indexed_links_count: 0,
        })
    }
    fn read_document(&self, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        Ok(self
            .docs
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.slug == slug)
            .map(|d| KnowledgeLookupOutcome::Found(d.clone()))
            .unwrap_or(KnowledgeLookupOutcome::DocumentNotFound))
    }
    fn search_documents(
        &self,
        _q: &KnowledgeSearchQuery,
    ) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        Ok(KnowledgeSearchOutcome::EmptyKnowledgeList)
    }
    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        Ok(self.docs.lock().unwrap().clone())
    }
    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        Ok(KnowledgeRebuildSummary {
            notes_restored: 0,
            links_restored: 0,
            duration_ms: 0,
        })
    }
}

struct MockGraphAnalyzer;
impl StructuralGraphAnalyzerPort for MockGraphAnalyzer {
    fn audit_graph_health(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        _top: usize,
    ) -> Result<GraphHealthReport, DomainViolation> {
        Ok(GraphHealthReport {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            dangling_edges: vec![],
            self_loop_edges: vec![],
            collapsed_edges_count: 0,
            shrink_guard_triggered: false,
            god_nodes: vec![GodNodeMetric::new(
                "crates/core/src/lib.rs",
                2,
                2,
                vec!["a".into()],
            )],
            surprising_connections: vec![SurprisingEdge::new("c", "d", "CORE", "CLI", "violation")],
            verdict: GraphHealthVerdict::Healthy,
        })
    }
    fn find_shortest_path(
        &self,
        _nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        s: &str,
        t: &str,
    ) -> Result<ShortestPathOutcome, DomainViolation> {
        if s == "node_1" && t == "node_4" {
            return Ok(ShortestPathOutcome::PathFound(vec![
                "node_1".into(),
                "node_2".into(),
                "node_3".into(),
                "node_4".into(),
            ]));
        }
        if edges.iter().any(|e| e.source_id == s && e.target_id == t) {
            return Ok(ShortestPathOutcome::PathFound(vec![s.into(), t.into()]));
        }
        Ok(ShortestPathOutcome::NoPathConnected)
    }
    fn slice_graph_context(
        &self,
        _n: &[StructuralNode],
        _e: &[StructuralEdge],
        _r: &str,
        _m: TraversalMode,
        _h: usize,
        _b: usize,
    ) -> Result<trustkit_core::entities::GraphTraversalOutcome, DomainViolation> {
        Ok(trustkit_core::entities::GraphTraversalOutcome::EmptyGraphSlice)
    }
}

#[test]
fn test_build_structural_graph_persists_and_emits_graph_report_sections() {
    let graph_repo = InMemoryGraphRepo::new();
    let analyzer = MockGraphAnalyzer;
    let knowledge_repo = InMemoryKnowledgeRepo::new();

    let nodes = vec![
        StructuralNode::new(
            "crates/core/src/lib.rs",
            "lib.rs",
            GraphNodeKind::CodeFile,
            "crates/core/src/lib.rs",
            "CORE",
        ),
        StructuralNode::new(
            "crates/db/src/lib.rs",
            "lib.rs",
            GraphNodeKind::CodeFile,
            "crates/db/src/lib.rs",
            "DB",
        ),
    ];
    let edges = vec![StructuralEdge::new(
        "crates/core/src/lib.rs",
        "crates/db/src/lib.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/core/src/lib.rs",
    )];
    let manifests = vec![FileGraphManifest::new(
        "crates/core/src/lib.rs",
        "hash1",
        1,
        "2026-10-29T22:00:00Z",
    )];
    let req = BuildStructuralGraphRequest::new(manifests, false, false, "2026-10-29T22:00:00Z")
        .with_pre_extracted(nodes, edges);

    let outcome =
        BuildStructuralGraphUseCase::execute(req, &graph_repo, &analyzer, &knowledge_repo)
            .expect("Build must succeed");
    assert_eq!(outcome.total_nodes, 2);
    assert_eq!(outcome.total_edges, 1);
    assert_eq!(outcome.verdict, GraphHealthVerdict::Healthy);

    let report_doc = knowledge_repo
        .read_document("graph/GRAPH_REPORT")
        .expect("Knowledge read must succeed");
    let doc = match report_doc {
        KnowledgeLookupOutcome::Found(d) => d,
        KnowledgeLookupOutcome::DocumentNotFound => panic!("Doc not found"),
    };
    let body = &doc.body_markdown;
    assert!(
        body.contains("## God Nodes"),
        "Report must contain God Nodes section"
    );
    assert!(
        body.contains("## Surprising Connections"),
        "Report must contain Surprising Connections section"
    );
    assert!(
        body.contains("## Module Cohesion Scores"),
        "Report must contain Module Cohesion Scores section"
    );
    assert!(
        body.contains("## Graph Health Audit"),
        "Report must contain Graph Health Audit section"
    );
    assert!(
        !body
            .chars()
            .any(|c| (c as u32) > 0x1F600 && (c as u32) < 0x1F64F),
        "Report must have zero emojis"
    );
}

#[test]
fn test_build_structural_graph_shrink_guard_triggered() {
    let graph_repo = InMemoryGraphRepo::new();
    let analyzer = MockGraphAnalyzer;
    let knowledge_repo = InMemoryKnowledgeRepo::new();

    let initial = (0..10)
        .map(|i| StructuralNode::new(format!("n{i}"), "n", GraphNodeKind::CodeFile, "f", "CORE"))
        .collect::<Vec<_>>();
    graph_repo
        .save_graph_snapshot(&initial, &[], &[], false)
        .unwrap();

    let reduced = vec![StructuralNode::new(
        "n0",
        "n",
        GraphNodeKind::CodeFile,
        "f",
        "CORE",
    )];
    let req = BuildStructuralGraphRequest::new(vec![], false, false, "2026-10-29T22:00:00Z")
        .with_pre_extracted(reduced, vec![]);
    let outcome =
        BuildStructuralGraphUseCase::execute(req, &graph_repo, &analyzer, &knowledge_repo).unwrap();
    assert_eq!(outcome.verdict, GraphHealthVerdict::ShrinkBlocked);
}

#[test]
fn test_find_shortest_graph_path_3_hops_returns_exact_4_nodes() {
    let graph_repo = InMemoryGraphRepo::new();
    let analyzer = MockGraphAnalyzer;

    let req = FindShortestGraphPathRequest::new("node_1", "node_4");
    let outcome = FindShortestGraphPathUseCase::execute(req, &graph_repo, &analyzer)
        .expect("Path use case must succeed");
    match outcome {
        ShortestPathOutcome::PathFound(path) => {
            assert_eq!(path.len(), 4, "3 hops must produce sequence of 4 nodes");
            assert_eq!(path, vec!["node_1", "node_2", "node_3", "node_4"]);
        }
        ShortestPathOutcome::NoPathConnected => panic!("Expected PathFound"),
    }
}

#[test]
fn test_find_shortest_graph_path_undirected_fallback() {
    let graph_repo = InMemoryGraphRepo::new();
    let analyzer = MockGraphAnalyzer;
    let edge = StructuralEdge::new(
        "node_b",
        "node_a",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "f",
    );
    graph_repo
        .save_graph_snapshot(&[], &[edge], &[], false)
        .unwrap();

    let req = FindShortestGraphPathRequest::undirected("node_a", "node_b");
    let outcome = FindShortestGraphPathUseCase::execute(req, &graph_repo, &analyzer).unwrap();
    assert_eq!(
        outcome,
        ShortestPathOutcome::PathFound(vec!["node_a".into(), "node_b".into()])
    );
}

#[test]
fn test_explain_graph_node_returns_complete_node_card() {
    let graph_repo = InMemoryGraphRepo::new();
    let knowledge_repo = InMemoryKnowledgeRepo::new();

    let target_node = StructuralNode::new(
        "target_node",
        "target",
        GraphNodeKind::PublicSymbol,
        "crates/core/src/target.rs",
        "CORE",
    );
    let neighbor_in = StructuralNode::new(
        "neighbor_in",
        "in",
        GraphNodeKind::CodeFile,
        "crates/core/src/in.rs",
        "CORE",
    );
    let neighbor_out = StructuralNode::new(
        "neighbor_out",
        "out",
        GraphNodeKind::KnowledgeNote,
        ".trustkit/wiki/specs/spec-1.md",
        "WIKI",
    );
    let edges = vec![
        StructuralEdge::new(
            "neighbor_in",
            "target_node",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "crates/core/src/in.rs",
        ),
        StructuralEdge::new(
            "target_node",
            "neighbor_out",
            EdgeRelationKind::WikiLinksTo,
            EdgeConfidence::Inferred,
            "crates/core/src/target.rs",
        ),
    ];
    graph_repo
        .save_graph_snapshot(
            &[target_node, neighbor_in, neighbor_out],
            &edges,
            &[],
            false,
        )
        .expect("Save snapshot must succeed");

    let req = ExplainGraphNodeRequest::new("target_node");
    let outcome = ExplainGraphNodeUseCase::execute(req, &graph_repo, &knowledge_repo)
        .expect("Explain use case must succeed");
    match outcome {
        ExplainNodeOutcome::Found(card) => {
            assert_eq!(card.node_id, "target_node");
            assert_eq!(card.kind, GraphNodeKind::PublicSymbol);
            assert_eq!(card.source_file, "crates/core/src/target.rs");
            assert_eq!(card.in_degree, 1);
            assert_eq!(card.out_degree, 1);
            assert_eq!(card.in_neighbors[0].node_id, "neighbor_in");
            assert_eq!(card.in_neighbors[0].confidence, EdgeConfidence::Extracted);
            assert_eq!(card.out_neighbors[0].node_id, "neighbor_out");
            assert_eq!(card.out_neighbors[0].confidence, EdgeConfidence::Inferred);
            assert!(card.linked_wiki_notes.contains(&"neighbor_out".to_string()));
        }
        ExplainNodeOutcome::NotFound { node_id } => panic!("Node '{node_id}' should be found"),
    }
}

#[test]
fn test_explain_graph_node_not_found() {
    let graph_repo = InMemoryGraphRepo::new();
    let knowledge_repo = InMemoryKnowledgeRepo::new();
    let req = ExplainGraphNodeRequest::new("non_existent_node");
    let outcome = ExplainGraphNodeUseCase::execute(req, &graph_repo, &knowledge_repo).unwrap();
    assert_eq!(
        outcome,
        ExplainNodeOutcome::NotFound {
            node_id: "non_existent_node".into()
        }
    );
    assert!(!outcome.is_found());
}
