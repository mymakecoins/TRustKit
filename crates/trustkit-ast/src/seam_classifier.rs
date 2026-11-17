// Graph-based Deep Module and Architectural Seam Classifier (TASK-002 / TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use trustkit_core::graph_entities::{
    EdgeRelationKind, GraphNodeKind, StructuralEdge, StructuralNode,
};
use trustkit_core::module_depth_entities::{
    ModuleDepthClassification, ModuleDepthMetric, SeamClassification, ShallowModuleCandidate,
};

use crate::module_depth_analyzer::ModuleDepthAnalyzer;

impl ModuleDepthAnalyzer {
    pub fn audit_depth_and_seams(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
    ) -> (
        Vec<ModuleDepthMetric>,
        Vec<ShallowModuleCandidate>,
        Vec<SeamClassification>,
    ) {
        let mut pub_syms_per_file: BTreeMap<&str, usize> = BTreeMap::new();
        for n in nodes {
            if n.kind == GraphNodeKind::PublicSymbol {
                *pub_syms_per_file.entry(n.source_file.as_str()).or_default() += 1;
            }
        }

        let mut outgoing_imports: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in edges {
            if matches!(
                e.relation,
                EdgeRelationKind::Imports | EdgeRelationKind::ReferencesSymbol
            ) {
                outgoing_imports
                    .entry(e.source_id.as_str())
                    .or_default()
                    .push(e.target_id.as_str());
            }
        }

        let mut depth_metrics = Vec::new();
        let mut shallow_candidates = Vec::new();
        let allow_disk_io = nodes.len() <= 500;

        for n in nodes {
            if n.kind != GraphNodeKind::CodeFile {
                continue;
            }
            let file_path = n.source_file.as_str();
            let sym_count = pub_syms_per_file.get(file_path).copied().unwrap_or(0);
            if sym_count == 0 && (!allow_disk_io || !Path::new(file_path).is_file()) {
                continue;
            }

            let metric = if allow_disk_io && let Ok(content) = fs::read_to_string(file_path) {
                self.evaluate_source_depth(file_path, &content)
            } else {
                let targets = outgoing_imports
                    .get(file_path)
                    .map(|v| {
                        let mut uniq = v.clone();
                        uniq.sort_unstable();
                        uniq.dedup();
                        uniq
                    })
                    .unwrap_or_default();
                let is_single_delegate = sym_count > 0 && targets.len() == 1;
                let internal = if is_single_delegate {
                    1
                } else {
                    sym_count.saturating_mul(2)
                };
                ModuleDepthMetric::evaluate(file_path, sym_count, internal, is_single_delegate)
            };

            if metric.classification == ModuleDepthClassification::ShallowPassThrough {
                let seam = outgoing_imports
                    .get(file_path)
                    .and_then(|v| v.first().copied())
                    .unwrap_or(file_path);
                shallow_candidates.push(ShallowModuleCandidate::new(
                    file_path,
                    seam,
                    metric.public_surface_units,
                    metric.internal_implementation_units,
                    format!(
                        "Deletion Test failed: public surface ({}) >= internal density ({}); pure pass-through delegation",
                        metric.public_surface_units, metric.internal_implementation_units
                    ),
                    format!(
                        "Collapse pass-through wrapper '{}' into '{}' or encapsulate domain invariants to create a Deep Module",
                        file_path, seam
                    ),
                ));
            }
            depth_metrics.push(metric);
        }

        let seams = self.classify_seams(nodes, edges);
        (depth_metrics, shallow_candidates, seams)
    }

    pub fn classify_seams(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
    ) -> Vec<SeamClassification> {
        let mut results = Vec::new();
        for n in nodes {
            if n.kind != GraphNodeKind::PublicSymbol {
                continue;
            }
            let is_seam = n.label.ends_with("Port")
                || n.label.ends_with("Trait")
                || n.label.ends_with("Gateway")
                || n.label.ends_with("Repository")
                || n.label.ends_with("Interface")
                || n.source_file.ends_with("ports.rs");
            if !is_seam {
                continue;
            }

            let mut adapters: HashSet<String> = HashSet::new();
            for e in edges {
                if e.relation == EdgeRelationKind::DefinesSymbol {
                    continue;
                }
                let matches_target = e.target_id == n.node_id
                    || e.target_id == n.label
                    || e.target_id.ends_with(&format!("::{}", n.label));
                if matches_target && e.source_id != n.source_file && e.source_id != n.node_id {
                    adapters.insert(e.source_id.clone());
                }
            }
            results.push(SeamClassification::evaluate(
                &n.label,
                &n.source_file,
                adapters.into_iter().collect(),
            ));
        }
        results
    }
}
