// Structural Graph Intelligence Domain Entities & Special Cases (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphNodeKind { CodeFile, PublicSymbol, KnowledgeNote, SpecRule, AtomicTaskNode }

impl GraphNodeKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CodeFile => "CODE_FILE",
            Self::PublicSymbol => "PUBLIC_SYMBOL",
            Self::KnowledgeNote => "KNOWLEDGE_NOTE",
            Self::SpecRule => "SPEC_RULE",
            Self::AtomicTaskNode => "ATOMIC_TASK_NODE",
        }
    }
    pub fn from_str_label(label: &str) -> Self {
        match label {
            "PUBLIC_SYMBOL" => Self::PublicSymbol,
            "KNOWLEDGE_NOTE" => Self::KnowledgeNote,
            "SPEC_RULE" => Self::SpecRule,
            "ATOMIC_TASK_NODE" => Self::AtomicTaskNode,
            _ => Self::CodeFile,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeConfidence { Extracted, Inferred, Ambiguous }

impl EdgeConfidence {
    pub const fn as_str(&self) -> &'static str {
        match self { Self::Extracted => "EXTRACTED", Self::Inferred => "INFERRED", Self::Ambiguous => "AMBIGUOUS" }
    }
    pub const fn priority_weight(&self) -> u8 {
        match self { Self::Extracted => 3, Self::Inferred => 2, Self::Ambiguous => 1 }
    }
    pub fn from_str_label(label: &str) -> Self {
        match label { "EXTRACTED" => Self::Extracted, "INFERRED" => Self::Inferred, _ => Self::Ambiguous }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeRelationKind { Imports, DefinesSymbol, ReferencesSymbol, WikiLinksTo, TracesTo }

impl EdgeRelationKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Imports => "IMPORTS",
            Self::DefinesSymbol => "DEFINES_SYMBOL",
            Self::ReferencesSymbol => "REFERENCES_SYMBOL",
            Self::WikiLinksTo => "WIKI_LINKS_TO",
            Self::TracesTo => "TRACES_TO",
        }
    }
    pub fn from_str_label(label: &str) -> Self {
        match label {
            "DEFINES_SYMBOL" => Self::DefinesSymbol,
            "REFERENCES_SYMBOL" => Self::ReferencesSymbol,
            "WIKI_LINKS_TO" => Self::WikiLinksTo,
            "TRACES_TO" => Self::TracesTo,
            _ => Self::Imports,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraversalMode { Bfs, Dfs }

impl TraversalMode {
    pub const fn as_str(&self) -> &'static str { match self { Self::Bfs => "BFS", Self::Dfs => "DFS" } }
    pub fn from_str_label(label: &str) -> Self { if label.eq_ignore_ascii_case("dfs") { Self::Dfs } else { Self::Bfs } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralNode {
    pub node_id: String,
    pub label: String,
    pub kind: GraphNodeKind,
    pub source_file: String,
    pub architectural_layer: String,
}

impl StructuralNode {
    pub fn new(node_id: impl Into<String>, label: impl Into<String>, kind: GraphNodeKind, source_file: impl Into<String>, architectural_layer: impl Into<String>) -> Self {
        Self { node_id: node_id.into(), label: label.into(), kind, source_file: source_file.into(), architectural_layer: architectural_layer.into() }
    }
    pub fn to_json(&self) -> String {
        format!("{{\"node_id\":\"{}\",\"label\":\"{}\",\"kind\":\"{}\",\"source_file\":\"{}\",\"architectural_layer\":\"{}\"}}",
            escape_json(&self.node_id), escape_json(&self.label), self.kind.as_str(), escape_json(&self.source_file), escape_json(&self.architectural_layer))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation: EdgeRelationKind,
    pub confidence: EdgeConfidence,
    pub source_file: String,
}

impl StructuralEdge {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>, relation: EdgeRelationKind, confidence: EdgeConfidence, source_file: impl Into<String>) -> Self {
        Self { source_id: source_id.into(), target_id: target_id.into(), relation, confidence, source_file: source_file.into() }
    }
    pub fn to_json(&self) -> String {
        format!("{{\"source_id\":\"{}\",\"target_id\":\"{}\",\"relation\":\"{}\",\"confidence\":\"{}\",\"source_file\":\"{}\"}}",
            escape_json(&self.source_id), escape_json(&self.target_id), self.relation.as_str(), self.confidence.as_str(), escape_json(&self.source_file))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileGraphManifest {
    pub file_path: String,
    pub blake3_digest: String,
    pub node_count: usize,
    pub updated_at: String,
}

impl FileGraphManifest {
    pub fn new(file_path: impl Into<String>, blake3_digest: impl Into<String>, node_count: usize, updated_at: impl Into<String>) -> Self {
        Self { file_path: file_path.into(), blake3_digest: blake3_digest.into(), node_count, updated_at: updated_at.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GodNodeMetric {
    pub node_id: String,
    pub in_degree: usize,
    pub out_degree: usize,
    pub total_degree: usize,
    pub coupled_neighbors: Vec<String>,
}

impl GodNodeMetric {
    pub fn new(node_id: impl Into<String>, in_degree: usize, out_degree: usize, coupled_neighbors: Vec<String>) -> Self {
        Self { node_id: node_id.into(), in_degree, out_degree, total_degree: in_degree.saturating_add(out_degree), coupled_neighbors }
    }
    pub fn to_json(&self) -> String {
        format!("{{\"node_id\":\"{}\",\"in_degree\":{},\"out_degree\":{},\"total_degree\":{},\"coupled_neighbors\":{}}}",
            escape_json(&self.node_id), self.in_degree, self.out_degree, self.total_degree, json_str_array(&self.coupled_neighbors))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurprisingEdge {
    pub source_id: String,
    pub target_id: String,
    pub source_layer: String,
    pub target_layer: String,
    pub reason: String,
}

impl SurprisingEdge {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>, source_layer: impl Into<String>, target_layer: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { source_id: source_id.into(), target_id: target_id.into(), source_layer: source_layer.into(), target_layer: target_layer.into(), reason: reason.into() }
    }
    pub fn to_json(&self) -> String {
        format!("{{\"source_id\":\"{}\",\"target_id\":\"{}\",\"source_layer\":\"{}\",\"target_layer\":\"{}\",\"reason\":\"{}\"}}",
            escape_json(&self.source_id), escape_json(&self.target_id), escape_json(&self.source_layer), escape_json(&self.target_layer), escape_json(&self.reason))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphHealthVerdict { Healthy, DegradedWarning, ShrinkBlocked }

impl GraphHealthVerdict {
    pub const fn as_str(&self) -> &'static str {
        match self { Self::Healthy => "HEALTHY", Self::DegradedWarning => "DEGRADED_WARNING", Self::ShrinkBlocked => "SHRINK_BLOCKED" }
    }
    pub const fn is_healthy(&self) -> bool { matches!(self, Self::Healthy) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphHealthReport {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub dangling_edges: Vec<StructuralEdge>,
    pub self_loop_edges: Vec<StructuralEdge>,
    pub collapsed_edges_count: usize,
    pub shrink_guard_triggered: bool,
    pub god_nodes: Vec<GodNodeMetric>,
    pub surprising_connections: Vec<SurprisingEdge>,
    pub verdict: GraphHealthVerdict,
}

impl GraphHealthReport {
    pub fn to_json(&self) -> String {
        let dangling: Vec<String> = self.dangling_edges.iter().map(StructuralEdge::to_json).collect();
        let loops: Vec<String> = self.self_loop_edges.iter().map(StructuralEdge::to_json).collect();
        let gods: Vec<String> = self.god_nodes.iter().map(GodNodeMetric::to_json).collect();
        let surp: Vec<String> = self.surprising_connections.iter().map(SurprisingEdge::to_json).collect();
        format!("{{\"total_nodes\":{},\"total_edges\":{},\"dangling_edges\":[{}],\"self_loop_edges\":[{}],\"collapsed_edges_count\":{},\"shrink_guard_triggered\":{},\"god_nodes\":[{}],\"surprising_connections\":[{}],\"verdict\":\"{}\"}}",
            self.total_nodes, self.total_edges, dangling.join(","), loops.join(","), self.collapsed_edges_count, self.shrink_guard_triggered, gods.join(","), surp.join(","), self.verdict.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphContextSlice {
    pub sliced_content: String,
    pub visited_nodes: Vec<String>,
    pub traversed_edges: Vec<StructuralEdge>,
    pub consumed_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphTraversalOutcome { VisitedSlice(GraphContextSlice), EmptyGraphSlice }

impl GraphTraversalOutcome {
    pub const fn is_empty(&self) -> bool { matches!(self, Self::EmptyGraphSlice) }
    pub const fn variant_label(&self) -> &'static str { match self { Self::VisitedSlice(_) => "VISITED_SLICE", Self::EmptyGraphSlice => "EMPTY_GRAPH_SLICE" } }
    pub fn to_json(&self) -> String {
        match self {
            Self::EmptyGraphSlice => "{\"outcome\":\"EMPTY_GRAPH_SLICE\",\"sliced_content\":\"\",\"visited_nodes\":[],\"traversed_edges\":[],\"consumed_lines\":0}".to_string(),
            Self::VisitedSlice(s) => {
                let edges: Vec<String> = s.traversed_edges.iter().map(StructuralEdge::to_json).collect();
                format!("{{\"outcome\":\"VISITED_SLICE\",\"sliced_content\":\"{}\",\"visited_nodes\":{},\"traversed_edges\":[{}],\"consumed_lines\":{}}}",
                    escape_json(&s.sliced_content), json_str_array(&s.visited_nodes), edges.join(","), s.consumed_lines)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortestPathOutcome { PathFound(Vec<String>), NoPathConnected }

impl ShortestPathOutcome {
    pub fn from_path(nodes: Vec<String>) -> Self { if nodes.is_empty() { Self::NoPathConnected } else { Self::PathFound(nodes) } }
    pub const fn is_connected(&self) -> bool { matches!(self, Self::PathFound(_)) }
    pub fn as_slice(&self) -> &[String] { match self { Self::PathFound(nodes) => nodes.as_slice(), Self::NoPathConnected => &[] } }
    pub const fn variant_label(&self) -> &'static str { match self { Self::PathFound(_) => "PATH_FOUND", Self::NoPathConnected => "NO_PATH_CONNECTED" } }
    pub fn as_json_array(&self) -> String { json_str_array(self.as_slice()) }
    pub fn to_json(&self) -> String { format!("{{\"outcome\":\"{}\",\"path\":{}}}", self.variant_label(), self.as_json_array()) }
}

fn json_str_array(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| format!("\"{}\"", escape_json(s))).collect();
    format!("[{}]", quoted.join(","))
}
fn escape_json(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}
#[cfg(test)]
#[path = "graph_entities_tests.rs"]
mod tests;
