// Handler for `trustkit graph` subcommands (TASK-008, RF-032, RN-008).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    GraphArgs, GraphBuildArgs, GraphExplainArgs, GraphHealthArgs, GraphPathArgs, GraphSubcommand,
};
use crate::error_handler::DomainOutcome;
use serde_json::json;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_ast::GraphHealthAuditor;
use trustkit_core::entities::{
    FileGraphManifest, GraphHealthVerdict,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{ArtifactHasherPort, StructuralGraphAnalyzerPort, StructuralGraphRepositoryPort};
use trustkit_core::use_cases::{
    BuildStructuralGraphRequest, BuildStructuralGraphUseCase, ExplainGraphNodeRequest,
    ExplainGraphNodeUseCase, ExplainNodeOutcome, FindShortestGraphPathRequest,
    FindShortestGraphPathUseCase,
};
use trustkit_db::blake3_adapter::Blake3ParallelSealerAdapter;
use trustkit_db::{GraphSqliteRepository, KnowledgeWikiFsRepository};

pub fn execute(args: &GraphArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    execute_in_root(args, json_mode, Path::new("."))
}

pub fn execute_in_root(
    args: &GraphArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        GraphSubcommand::Build(b) => handle_build(b, json_mode, repo_root),
        GraphSubcommand::Health(h) => handle_health(h, json_mode, repo_root),
        GraphSubcommand::Path(p) => handle_path(p, json_mode, repo_root),
        GraphSubcommand::Explain(e) => handle_explain(e, json_mode, repo_root),
    }
}

fn handle_build(
    args: &GraphBuildArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let graph_repo = GraphSqliteRepository::open_from_root(repo_root)?;
    let knowledge_repo = KnowledgeWikiFsRepository::open_from_root(repo_root)?;
    let analyzer = GraphHealthAuditor::new();

    let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let now_iso = format!("{now_ts}");

    let mut manifests = Vec::new();
    scan_repo_files(repo_root, repo_root, &mut manifests, &now_iso)?;
    manifests.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    let req = BuildStructuralGraphRequest::new(manifests, args.force, args.update, now_iso);
    let outcome = BuildStructuralGraphUseCase::execute(req, &graph_repo, &analyzer, &knowledge_repo)?;

    let is_blocked = outcome.verdict == GraphHealthVerdict::ShrinkBlocked;
    if json_mode {
        let payload = json!({
            "verdict": if is_blocked { "FAIL" } else { "PASS" },
            "outcome": outcome.verdict.as_str(),
            "total_nodes": outcome.total_nodes,
            "total_edges": outcome.total_edges,
            "report_path": outcome.report_path,
            "shrink_guard_triggered": outcome.health_report.shrink_guard_triggered,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    } else if is_blocked {
        println!("[FAIL] [SHRINK_BLOCKED] Graph build blocked by Shrink-Guard. Use --force to override.");
    } else {
        println!("[PASS] [{}] Graph built: {} nodes, {} edges. Report: {}", outcome.verdict.as_str(), outcome.total_nodes, outcome.total_edges, outcome.report_path);
    }

    if is_blocked {
        Ok(DomainOutcome::PassWithDetails("SHRINK_BLOCKED".into()))
    } else {
        Ok(DomainOutcome::Success)
    }
}

fn handle_health(
    args: &GraphHealthArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let graph_repo = GraphSqliteRepository::open_from_root(repo_root)?;
    let analyzer = GraphHealthAuditor::new();
    let nodes = graph_repo.load_nodes()?;
    let edges = graph_repo.load_edges()?;
    let report = analyzer.audit_graph_health(&nodes, &edges, args.top_god_nodes)?;

    if json_mode {
        let gods: Vec<serde_json::Value> = report.god_nodes.iter().map(|g| json!({
            "node_id": g.node_id, "in_degree": g.in_degree, "out_degree": g.out_degree,
            "total_degree": g.total_degree, "coupled_neighbors": g.coupled_neighbors,
        })).collect();
        let surp: Vec<serde_json::Value> = report.surprising_connections.iter().map(|s| json!({
            "source_id": s.source_id, "target_id": s.target_id, "source_layer": s.source_layer,
            "target_layer": s.target_layer, "reason": s.reason,
        })).collect();
        let payload = json!({
            "verdict": "PASS", "outcome": report.verdict.as_str(),
            "total_nodes": report.total_nodes, "total_edges": report.total_edges,
            "dangling_edges_count": report.dangling_edges.len(),
            "self_loop_edges_count": report.self_loop_edges.len(),
            "collapsed_edges_count": report.collapsed_edges_count,
            "shrink_guard_triggered": report.shrink_guard_triggered,
            "god_nodes": gods, "surprising_connections": surp,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    } else {
        println!("[PASS] [{}] Graph health: {} nodes, {} edges (dangling: {}, loops: {}, collapsed: {})",
            report.verdict.as_str(), report.total_nodes, report.total_edges,
            report.dangling_edges.len(), report.self_loop_edges.len(), report.collapsed_edges_count);
    }
    Ok(DomainOutcome::Success)
}

fn handle_path(
    args: &GraphPathArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let graph_repo = GraphSqliteRepository::open_from_root(repo_root)?;
    let analyzer = GraphHealthAuditor::new();
    let req = if args.undirected {
        FindShortestGraphPathRequest::undirected(&args.source, &args.target)
    } else {
        FindShortestGraphPathRequest::new(&args.source, &args.target)
    };
    let outcome = FindShortestGraphPathUseCase::execute(req, &graph_repo, &analyzer)?;
    let path = outcome.as_slice();
    let hops = if outcome.is_connected() { path.len().saturating_sub(1) } else { 0 };

    if json_mode {
        let payload = json!({
            "verdict": "PASS", "outcome": outcome.variant_label(),
            "source": args.source, "target": args.target, "path": path, "hops": hops,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    } else if outcome.is_connected() {
        println!("[PASS] [PATH_FOUND] Path from {} to {}: {}", args.source, args.target, path.join(" -> "));
    } else {
        println!("[PASS] [NO_PATH_CONNECTED] No path connected between {} and {}", args.source, args.target);
    }
    Ok(DomainOutcome::Success)
}

fn handle_explain(
    args: &GraphExplainArgs,
    json_mode: bool,
    repo_root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let graph_repo = GraphSqliteRepository::open_from_root(repo_root)?;
    let knowledge_repo = KnowledgeWikiFsRepository::open_from_root(repo_root)?;
    let req = ExplainGraphNodeRequest::new(&args.node_id);
    let outcome = ExplainGraphNodeUseCase::execute(req, &graph_repo, &knowledge_repo)?;

    match outcome {
        ExplainNodeOutcome::Found(exp) => {
            let in_nbrs: Vec<serde_json::Value> = exp.in_neighbors.iter().map(|n| json!({
                "node_id": n.node_id, "relation": n.relation.as_str(), "confidence": n.confidence.as_str(),
            })).collect();
            let out_nbrs: Vec<serde_json::Value> = exp.out_neighbors.iter().map(|n| json!({
                "node_id": n.node_id, "relation": n.relation.as_str(), "confidence": n.confidence.as_str(),
            })).collect();
            if json_mode {
                let payload = json!({
                    "verdict": "PASS", "outcome": "FOUND",
                    "node": {
                        "node_id": exp.node_id, "label": exp.label, "kind": exp.kind.as_str(),
                        "source_file": exp.source_file, "architectural_layer": exp.architectural_layer,
                        "in_degree": exp.in_degree, "out_degree": exp.out_degree,
                        "in_neighbors": in_nbrs, "out_neighbors": out_nbrs,
                        "linked_wiki_notes": exp.linked_wiki_notes,
                    }
                });
                println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
            } else {
                println!("[PASS] Node: {} ({})\nKind: {}\nFile: {}\nLayer: {}\nDegree: in={}, out={}\nWiki links: {}",
                    exp.node_id, exp.label, exp.kind.as_str(), exp.source_file, exp.architectural_layer,
                    exp.in_degree, exp.out_degree, exp.linked_wiki_notes.join(", "));
            }
        }
        ExplainNodeOutcome::NotFound { node_id } => {
            if json_mode {
                let payload = json!({"verdict": "PASS", "outcome": "NODE_NOT_FOUND", "node_id": node_id});
                println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
            } else {
                println!("[PASS] [NODE_NOT_FOUND] Node not found in graph: {node_id}");
            }
        }
    }
    Ok(DomainOutcome::Success)
}

fn scan_repo_files(
    root: &Path,
    current: &Path,
    manifests: &mut Vec<FileGraphManifest>,
    now_iso: &str,
) -> Result<(), DomainViolation> {
    if !current.exists() { return Ok(()); }
    let sealer = Blake3ParallelSealerAdapter::new();
    let entries = std::fs::read_dir(current).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "graph_cmd".into(), details: format!("Failed reading dir {}: {}", current.display(), e),
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if matches!(name_str.as_ref(), ".git" | "target" | "node_modules" | ".cargo" | "dist" | "build") {
                continue;
            }
            if name_str == ".trustkit" {
                let wiki_path = path.join("wiki");
                if wiki_path.exists() {
                    scan_repo_files(root, &wiki_path, manifests, now_iso)?;
                }
                continue;
            }
            scan_repo_files(root, &path, manifests, now_iso)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str())
            && matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "md")
            && let Ok(rel) = path.strip_prefix(root)
        {
            let rel_str = rel.to_string_lossy().to_string();
            let digest = sealer.compute_file_blake3(&path.to_string_lossy()).unwrap_or_default();
            manifests.push(FileGraphManifest::new(rel_str, digest, 0, now_iso));
        }
    }
    Ok(())
}
