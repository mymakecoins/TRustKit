// Structural Graph Intelligence Use Cases (TASK-007, RF-031, RN-007).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Emojis.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    EdgeConfidence, EdgeRelationKind, FileGraphManifest, GraphHealthReport, GraphHealthVerdict,
    GraphNodeKind, KnowledgeDocument, KnowledgeWriteOutcome, ShortestPathOutcome, StructuralEdge,
    StructuralNode,
};
use crate::errors::DomainViolation;
use crate::ports::{
    KnowledgeRepositoryPort, StructuralGraphAnalyzerPort, StructuralGraphRepositoryPort,
};
use crate::use_cases::structural_graph_report::render_graph_report;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStructuralGraphRequest {
    pub manifests: Vec<FileGraphManifest>,
    pub force: bool,
    pub update_only: bool,
    pub now_iso: String,
    pub pre_extracted_nodes: Vec<StructuralNode>,
    pub pre_extracted_edges: Vec<StructuralEdge>,
}

impl BuildStructuralGraphRequest {
    pub fn new(
        manifests: Vec<FileGraphManifest>,
        force: bool,
        update_only: bool,
        now_iso: impl Into<String>,
    ) -> Self {
        Self {
            manifests,
            force,
            update_only,
            now_iso: now_iso.into(),
            pre_extracted_nodes: Vec::new(),
            pre_extracted_edges: Vec::new(),
        }
    }
    pub fn with_pre_extracted(
        mut self,
        nodes: Vec<StructuralNode>,
        edges: Vec<StructuralEdge>,
    ) -> Self {
        self.pre_extracted_nodes = nodes;
        self.pre_extracted_edges = edges;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStructuralGraphOutcome {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub health_report: GraphHealthReport,
    pub report_markdown: String,
    pub report_path: String,
    pub verdict: GraphHealthVerdict,
}

pub struct BuildStructuralGraphUseCase;

impl BuildStructuralGraphUseCase {
    pub fn execute(
        req: BuildStructuralGraphRequest,
        graph_repo: &dyn StructuralGraphRepositoryPort,
        analyzer: &dyn StructuralGraphAnalyzerPort,
        knowledge_repo: &dyn KnowledgeRepositoryPort,
    ) -> Result<BuildStructuralGraphOutcome, DomainViolation> {
        let (nodes, edges) = if !req.pre_extracted_nodes.is_empty() {
            (req.pre_extracted_nodes, req.pre_extracted_edges)
        } else if req.update_only {
            let changed = graph_repo.detect_changed_files(&req.manifests)?;
            if changed.is_empty() {
                (graph_repo.load_nodes()?, graph_repo.load_edges()?)
            } else {
                let (new_nodes, new_edges) = analyzer.extract_from_files(&changed)?;
                let mut existing_nodes: Vec<StructuralNode> = graph_repo
                    .load_nodes()?
                    .into_iter()
                    .filter(|n| !changed.contains(&n.source_file))
                    .collect();
                let mut existing_edges: Vec<StructuralEdge> = graph_repo
                    .load_edges()?
                    .into_iter()
                    .filter(|e| !changed.contains(&e.source_file))
                    .collect();
                existing_nodes.extend(new_nodes);
                existing_edges.extend(new_edges);
                (existing_nodes, existing_edges)
            }
        } else {
            let paths: Vec<String> = req.manifests.iter().map(|m| m.file_path.clone()).collect();
            analyzer.extract_from_files(&paths)?
        };

        let verdict = graph_repo.save_graph_snapshot(&nodes, &edges, &req.manifests, req.force)?;
        let health_report = analyzer.audit_graph_health(&nodes, &edges, 10)?;
        let report_markdown =
            render_graph_report(&nodes, &edges, &health_report, &req.manifests, &req.now_iso);

        let doc = KnowledgeDocument::new(
            "graph/GRAPH_REPORT",
            "Structural Graph Report",
            vec!["graph".into(), "report".into(), "structural".into()],
            report_markdown.clone(),
            req.now_iso,
            String::new(),
        );

        let report_path = match knowledge_repo.write_document(&doc)? {
            KnowledgeWriteOutcome::Success { file_path, .. } => file_path,
        };

        Ok(BuildStructuralGraphOutcome {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            health_report,
            report_markdown,
            report_path,
            verdict,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindShortestGraphPathRequest {
    pub source_id: String,
    pub target_id: String,
    pub undirected: bool,
}

impl FindShortestGraphPathRequest {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            undirected: false,
        }
    }
    pub fn undirected(source_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            undirected: true,
        }
    }
}

pub struct FindShortestGraphPathUseCase;

impl FindShortestGraphPathUseCase {
    pub fn execute(
        req: FindShortestGraphPathRequest,
        graph_repo: &dyn StructuralGraphRepositoryPort,
        analyzer: &dyn StructuralGraphAnalyzerPort,
    ) -> Result<ShortestPathOutcome, DomainViolation> {
        let nodes = graph_repo.load_nodes()?;
        let mut edges = graph_repo.load_edges()?;
        if req.undirected {
            let rev: Vec<StructuralEdge> = edges
                .iter()
                .map(|e| {
                    StructuralEdge::new(
                        &e.target_id,
                        &e.source_id,
                        e.relation,
                        e.confidence,
                        &e.source_file,
                    )
                })
                .collect();
            edges.extend(rev);
        }
        analyzer.find_shortest_path(&nodes, &edges, &req.source_id, &req.target_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborConnection {
    pub node_id: String,
    pub relation: EdgeRelationKind,
    pub confidence: EdgeConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeExplanation {
    pub node_id: String,
    pub label: String,
    pub kind: GraphNodeKind,
    pub source_file: String,
    pub architectural_layer: String,
    pub in_degree: usize,
    pub out_degree: usize,
    pub in_neighbors: Vec<NeighborConnection>,
    pub out_neighbors: Vec<NeighborConnection>,
    pub linked_wiki_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplainNodeOutcome {
    Found(NodeExplanation),
    NotFound { node_id: String },
}

impl ExplainNodeOutcome {
    pub const fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }
    pub const fn explanation(&self) -> Option<&NodeExplanation> {
        match self {
            Self::Found(e) => Some(e),
            Self::NotFound { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainGraphNodeRequest {
    pub node_id: String,
}

impl ExplainGraphNodeRequest {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
        }
    }
}

pub struct ExplainGraphNodeUseCase;

impl ExplainGraphNodeUseCase {
    pub fn execute(
        req: ExplainGraphNodeRequest,
        graph_repo: &dyn StructuralGraphRepositoryPort,
        knowledge_repo: &dyn KnowledgeRepositoryPort,
    ) -> Result<ExplainNodeOutcome, DomainViolation> {
        let nodes = graph_repo.load_nodes()?;
        let target = match nodes.into_iter().find(|n| n.node_id == req.node_id) {
            Some(t) => t,
            None => {
                return Ok(ExplainNodeOutcome::NotFound {
                    node_id: req.node_id,
                });
            }
        };

        let edges = graph_repo.load_edges()?;
        let mut in_nbrs = Vec::new();
        let mut out_nbrs = Vec::new();
        let mut wiki_notes = Vec::new();

        for e in &edges {
            if e.target_id == req.node_id {
                in_nbrs.push(NeighborConnection {
                    node_id: e.source_id.clone(),
                    relation: e.relation,
                    confidence: e.confidence,
                });
                if e.relation == EdgeRelationKind::WikiLinksTo || is_wiki_slug(&e.source_id) {
                    wiki_notes.push(e.source_id.clone());
                }
            }
            if e.source_id == req.node_id {
                out_nbrs.push(NeighborConnection {
                    node_id: e.target_id.clone(),
                    relation: e.relation,
                    confidence: e.confidence,
                });
                if e.relation == EdgeRelationKind::WikiLinksTo || is_wiki_slug(&e.target_id) {
                    wiki_notes.push(e.target_id.clone());
                }
            }
        }

        if let Ok(links) = knowledge_repo.query_adjacent_links(&req.node_id) {
            for l in links {
                wiki_notes.push(l.target_slug);
            }
        }
        wiki_notes.sort();
        wiki_notes.dedup();

        Ok(ExplainNodeOutcome::Found(NodeExplanation {
            node_id: target.node_id,
            label: target.label,
            kind: target.kind,
            source_file: target.source_file,
            architectural_layer: target.architectural_layer,
            in_degree: in_nbrs.len(),
            out_degree: out_nbrs.len(),
            in_neighbors: in_nbrs,
            out_neighbors: out_nbrs,
            linked_wiki_notes: wiki_notes,
        }))
    }
}

fn is_wiki_slug(id: &str) -> bool {
    id.ends_with(".md")
        || id.starts_with("specs/")
        || id.starts_with("tasks/")
        || id.starts_with("adrs/")
}
