// Dispatcher for MCP `trustkit_graph_*` tools (TASK-008, RF-032).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::mcp_server::{McpTool, tool};
use serde_json::{Map, Value, json};
use std::sync::Arc;
use trustkit_core::entities::{GraphTraversalOutcome, StructuralEdge, TraversalMode};
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeRebuildSummary, KnowledgeSearchOutcome,
    KnowledgeSearchQuery, KnowledgeWriteOutcome,
};
use trustkit_core::ports::{
    KnowledgeRepositoryPort, StructuralGraphAnalyzerPort, StructuralGraphRepositoryPort,
};
use trustkit_core::use_cases::{
    ExplainGraphNodeRequest, ExplainGraphNodeUseCase, ExplainNodeOutcome,
    FindShortestGraphPathRequest, FindShortestGraphPathUseCase,
};

pub fn graph_tools() -> Vec<McpTool> {
    vec![
        tool(
            "trustkit_graph_query",
            "Consulta o grafo estrutural com fatiamento BFS/DFS ou busca de nos",
            json!({
                "type":"object","properties":{
                    "file":{"type":"string","description":"Arquivo raiz para fatiamento"},
                    "query":{"type":"string","description":"Termo de busca para filtrar nos"},
                    "mode":{"type":"string","enum":["bfs","dfs"],"description":"Modo de travessia"},
                    "hops":{"type":"integer","description":"Numero maximo de saltos (padrao: 1)"},
                    "budget":{"type":"integer","description":"Orcamento maximo de linhas (padrao: 100)"}
                }
            }),
        ),
        tool(
            "trustkit_graph_path",
            "Calcula o menor caminho direcionado ou nao direcionado via BFS",
            json!({
                "type":"object","properties":{
                    "source":{"type":"string","description":"ID ou arquivo de origem"},
                    "target":{"type":"string","description":"ID ou arquivo de destino"},
                    "undirected":{"type":"boolean","description":"Considerar relacoes bidirecionais"}
                },"required":["source","target"]
            }),
        ),
        tool(
            "trustkit_graph_explain",
            "Explica conexoes de entrada, saida e notas wiki vinculadas a um no",
            json!({
                "type":"object","properties":{"node_id":{"type":"string","description":"Identificador unico do no"}},"required":["node_id"]
            }),
        ),
        tool(
            "trustkit_graph_health",
            "Audita integridade estrutural, God Nodes, loops e fronteiras",
            json!({
                "type":"object","properties":{"top_god_nodes":{"type":"integer","description":"Quantidade de God Nodes a ranquear"}}
            }),
        ),
    ]
}

pub fn dispatch_graph_tool(
    name: &str,
    args: &Map<String, Value>,
    repo_opt: Option<&Arc<dyn StructuralGraphRepositoryPort>>,
    analyzer_opt: Option<&Arc<dyn StructuralGraphAnalyzerPort>>,
    knowledge_opt: Option<&Arc<dyn KnowledgeRepositoryPort>>,
) -> (String, bool) {
    let Some(repo) = repo_opt else {
        return (
            json!({"verdict": "PASS", "tool": name, "status": "acknowledged"}).to_string(),
            false,
        );
    };
    match name {
        "trustkit_graph_query" => handle_query(name, args, repo, analyzer_opt),
        "trustkit_graph_path" => handle_path(name, args, repo, analyzer_opt),
        "trustkit_graph_explain" => handle_explain(name, args, repo, knowledge_opt),
        "trustkit_graph_health" => handle_health(name, args, repo, analyzer_opt),
        _ => (
            json!({"verdict": "FAIL", "tool": name, "error": "Unsupported graph tool"}).to_string(),
            true,
        ),
    }
}

fn handle_query(
    tool: &str,
    args: &Map<String, Value>,
    repo: &Arc<dyn StructuralGraphRepositoryPort>,
    analyzer_opt: Option<&Arc<dyn StructuralGraphAnalyzerPort>>,
) -> (String, bool) {
    let file = args
        .get("file")
        .or_else(|| args.get("root_file"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let q_str = args.get("query").and_then(Value::as_str).unwrap_or("");
    let mode_str = args.get("mode").and_then(Value::as_str).unwrap_or("bfs");
    let hops = args.get("hops").and_then(Value::as_u64).unwrap_or(1) as usize;
    let budget = args.get("budget").and_then(Value::as_u64).unwrap_or(100) as usize;

    let (nodes, edges) = match (repo.load_nodes(), repo.load_edges()) {
        (Ok(n), Ok(e)) => (n, e),
        (Err(e), _) | (_, Err(e)) => {
            return (
                json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(),
                true,
            );
        }
    };

    if !file.is_empty() {
        let Some(analyzer) = analyzer_opt else {
            return (
                json!({"verdict":"FAIL","tool":tool,"error":"Graph analyzer unavailable"})
                    .to_string(),
                true,
            );
        };
        match analyzer.slice_graph_context(&nodes, &edges, file, TraversalMode::from_str_label(mode_str), hops, budget) {
            Ok(GraphTraversalOutcome::VisitedSlice(s)) => {
                let edges_json: Vec<Value> = s.traversed_edges.iter().map(edge_to_val).collect();
                (json!({"verdict":"PASS","tool":tool,"outcome":"VISITED_SLICE","file":file,"sliced_content":s.sliced_content,"visited_nodes":s.visited_nodes,"traversed_edges":edges_json,"consumed_lines":s.consumed_lines}).to_string(), false)
            }
            Ok(GraphTraversalOutcome::EmptyGraphSlice) => {
                (json!({"verdict":"PASS","tool":tool,"outcome":"EMPTY_GRAPH_SLICE","file":file,"sliced_content":"","visited_nodes":[],"traversed_edges":[],"consumed_lines":0}).to_string(), false)
            }
            Err(e) => (json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(), true),
        }
    } else {
        let hits: Vec<Value> = nodes.into_iter()
            .filter(|n| q_str.is_empty() || n.node_id.contains(q_str) || n.label.contains(q_str))
            .map(|n| json!({"node_id":n.node_id,"label":n.label,"kind":n.kind.as_str(),"source_file":n.source_file,"architectural_layer":n.architectural_layer}))
            .collect();
        let outcome = if hits.is_empty() {
            "EMPTY_GRAPH_SLICE"
        } else {
            "RESULTS"
        };
        (
            json!({"verdict":"PASS","tool":tool,"outcome":outcome,"query":q_str,"hits":hits})
                .to_string(),
            false,
        )
    }
}

fn handle_path(
    tool: &str,
    args: &Map<String, Value>,
    repo: &Arc<dyn StructuralGraphRepositoryPort>,
    analyzer_opt: Option<&Arc<dyn StructuralGraphAnalyzerPort>>,
) -> (String, bool) {
    let source = args.get("source").and_then(Value::as_str).unwrap_or("");
    let target = args.get("target").and_then(Value::as_str).unwrap_or("");
    let undirected = args
        .get("undirected")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Some(analyzer) = analyzer_opt else {
        return (
            json!({"verdict":"FAIL","tool":tool,"error":"Graph analyzer unavailable"}).to_string(),
            true,
        );
    };
    let req = if undirected {
        FindShortestGraphPathRequest::undirected(source, target)
    } else {
        FindShortestGraphPathRequest::new(source, target)
    };
    match FindShortestGraphPathUseCase::execute(req, repo.as_ref(), analyzer.as_ref()) {
        Ok(out) => {
            let path = out.as_slice();
            let hops = if out.is_connected() {
                path.len().saturating_sub(1)
            } else {
                0
            };
            (json!({"verdict":"PASS","tool":tool,"outcome":out.variant_label(),"source":source,"target":target,"path":path,"hops":hops}).to_string(), false)
        }
        Err(e) => (
            json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(),
            true,
        ),
    }
}

fn handle_explain(
    tool: &str,
    args: &Map<String, Value>,
    repo: &Arc<dyn StructuralGraphRepositoryPort>,
    knowledge_opt: Option<&Arc<dyn KnowledgeRepositoryPort>>,
) -> (String, bool) {
    let node_id = args.get("node_id").and_then(Value::as_str).unwrap_or("");
    let fallback = EmptyKnowledgeRepo;
    let k_ref: &dyn KnowledgeRepositoryPort = knowledge_opt.map_or(&fallback, |k| k.as_ref());
    match ExplainGraphNodeUseCase::execute(
        ExplainGraphNodeRequest::new(node_id),
        repo.as_ref(),
        k_ref,
    ) {
        Ok(ExplainNodeOutcome::Found(exp)) => {
            let in_nbrs: Vec<Value> = exp.in_neighbors.iter().map(|n| json!({"node_id":n.node_id,"relation":n.relation.as_str(),"confidence":n.confidence.as_str()})).collect();
            let out_nbrs: Vec<Value> = exp.out_neighbors.iter().map(|n| json!({"node_id":n.node_id,"relation":n.relation.as_str(),"confidence":n.confidence.as_str()})).collect();
            (json!({"verdict":"PASS","tool":tool,"outcome":"FOUND","node":{"node_id":exp.node_id,"label":exp.label,"kind":exp.kind.as_str(),"source_file":exp.source_file,"architectural_layer":exp.architectural_layer,"in_degree":exp.in_degree,"out_degree":exp.out_degree,"in_neighbors":in_nbrs,"out_neighbors":out_nbrs,"linked_wiki_notes":exp.linked_wiki_notes}}).to_string(), false)
        }
        Ok(ExplainNodeOutcome::NotFound { node_id }) => (
            json!({"verdict":"PASS","tool":tool,"outcome":"NODE_NOT_FOUND","node_id":node_id})
                .to_string(),
            false,
        ),
        Err(e) => (
            json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(),
            true,
        ),
    }
}

fn handle_health(
    tool: &str,
    args: &Map<String, Value>,
    repo: &Arc<dyn StructuralGraphRepositoryPort>,
    analyzer_opt: Option<&Arc<dyn StructuralGraphAnalyzerPort>>,
) -> (String, bool) {
    let top_god_nodes = args
        .get("top_god_nodes")
        .and_then(Value::as_u64)
        .unwrap_or(10) as usize;
    let Some(analyzer) = analyzer_opt else {
        return (
            json!({"verdict":"FAIL","tool":tool,"error":"Graph analyzer unavailable"}).to_string(),
            true,
        );
    };
    let (nodes, edges) = match (repo.load_nodes(), repo.load_edges()) {
        (Ok(n), Ok(e)) => (n, e),
        (Err(e), _) | (_, Err(e)) => {
            return (
                json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(),
                true,
            );
        }
    };
    match analyzer.audit_graph_health(&nodes, &edges, top_god_nodes) {
        Ok(report) => {
            let gods: Vec<Value> = report.god_nodes.iter().map(|g| json!({"node_id":g.node_id,"in_degree":g.in_degree,"out_degree":g.out_degree,"total_degree":g.total_degree,"coupled_neighbors":g.coupled_neighbors})).collect();
            let surp: Vec<Value> = report.surprising_connections.iter().map(|s| json!({"source_id":s.source_id,"target_id":s.target_id,"source_layer":s.source_layer,"target_layer":s.target_layer,"reason":s.reason})).collect();
            let dangling: Vec<Value> = report.dangling_edges.iter().map(edge_to_val).collect();
            let loops: Vec<Value> = report.self_loop_edges.iter().map(edge_to_val).collect();
            (json!({"verdict":"PASS","tool":tool,"outcome":report.verdict.as_str(),"report":{"total_nodes":report.total_nodes,"total_edges":report.total_edges,"dangling_edges":dangling,"self_loop_edges":loops,"dangling_edges_count":report.dangling_edges.len(),"self_loop_edges_count":report.self_loop_edges.len(),"collapsed_edges_count":report.collapsed_edges_count,"shrink_guard_triggered":report.shrink_guard_triggered,"god_nodes":gods,"surprising_connections":surp,"verdict":report.verdict.as_str()}}).to_string(), false)
        }
        Err(e) => (
            json!({"verdict":"FAIL","tool":tool,"error":e.to_string()}).to_string(),
            true,
        ),
    }
}

fn edge_to_val(e: &StructuralEdge) -> Value {
    json!({"source_id":e.source_id,"target_id":e.target_id,"relation":e.relation.as_str(),"confidence":e.confidence.as_str(),"source_file":e.source_file})
}

struct EmptyKnowledgeRepo;
impl KnowledgeRepositoryPort for EmptyKnowledgeRepo {
    fn write_document(
        &self,
        _: &KnowledgeDocument,
    ) -> Result<KnowledgeWriteOutcome, DomainViolation> {
        Ok(KnowledgeWriteOutcome::Success {
            slug: String::new(),
            file_path: String::new(),
            blake3_digest: String::new(),
            indexed_links_count: 0,
        })
    }
    fn read_document(&self, _: &str) -> Result<KnowledgeLookupOutcome, DomainViolation> {
        Ok(KnowledgeLookupOutcome::DocumentNotFound)
    }
    fn search_documents(
        &self,
        _: &KnowledgeSearchQuery,
    ) -> Result<KnowledgeSearchOutcome, DomainViolation> {
        Ok(KnowledgeSearchOutcome::EmptyKnowledgeList)
    }
    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation> {
        Ok(vec![])
    }
    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation> {
        Ok(KnowledgeRebuildSummary {
            notes_restored: 0,
            links_restored: 0,
            duration_ms: 0,
        })
    }
}
