// Deterministic AST extractor for Traceability Matrix, Mermaid DAG, and Gherkin scenarios (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::collections::BTreeMap;
use trustkit_core::entities::{AtomicTaskDraft, AtomizationSource};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::TaskAtomizerPort;

#[derive(Debug, Default, Clone)]
pub struct SpecMatrixAtomizer;

#[derive(Debug, Clone, Default)]
struct ParsedGherkinBlock {
    header: String,
    given: String,
    when: String,
    then: String,
}

impl SpecMatrixAtomizer {
    pub fn new() -> Self {
        Self
    }

    pub fn atomize_markdown(markdown: &str) -> Result<Vec<AtomicTaskDraft>, DomainViolation> {
        let (node_labels, mermaid_preds) = parse_mermaid_section(markdown);
        let gherkin_blocks = parse_gherkin_blocks(markdown);
        let rows = parse_traceability_rows(markdown);

        let mut node_to_task: BTreeMap<String, String> = BTreeMap::new();
        for cols in &rows {
            if !cols[3].is_empty() && !cols[2].is_empty() {
                node_to_task
                    .entry(cols[3].clone())
                    .or_insert_with(|| cols[2].clone());
            }
        }

        let mut drafts = Vec::with_capacity(rows.len());
        for c in rows {
            let [rn, rf, task_id, node, test_file, ev] = c;
            let layer = infer_layer(&test_file);
            let expected_files = infer_expected_files(&test_file);
            let title = node_labels
                .get(&node)
                .cloned()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Implementacao de {node} ({rf})"));

            let mut depends_on = Vec::new();
            if let Some(preds) = mermaid_preds.get(&node) {
                for pred_node in preds {
                    if let Some(dep_id) = node_to_task.get(pred_node)
                        && dep_id != &task_id
                        && !depends_on.contains(dep_id)
                    {
                        depends_on.push(dep_id.clone());
                    }
                }
            }

            let (given_clause, when_clause, then_clause) =
                match_gherkin(&gherkin_blocks, &rf, &task_id, &title);
            let context_summary = format!("{title} atendendo {rn} e {rf} no componente {node}.");

            drafts.push(AtomicTaskDraft {
                task_id,
                title,
                architectural_layer: layer,
                depends_on,
                expected_files,
                target_test_file: test_file,
                context_summary,
                given_clause,
                when_clause,
                then_clause,
                business_rule_id: rn,
                functional_req_id: rf,
                mermaid_node: node,
                evidence_artifact: ev,
            });
        }

        drafts.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        Ok(drafts)
    }
}

impl TaskAtomizerPort for SpecMatrixAtomizer {
    fn atomize_source(
        &self,
        source: &AtomizationSource,
    ) -> Result<Vec<AtomicTaskDraft>, DomainViolation> {
        match source {
            AtomizationSource::SpecificationMarkdown {
                markdown_content, ..
            } => Self::atomize_markdown(markdown_content),
            AtomizationSource::GitHubIssueBody { body_markdown, .. } => {
                Self::atomize_markdown(body_markdown)
            }
        }
    }
}

fn parse_mermaid_section(
    markdown: &str,
) -> (BTreeMap<String, String>, BTreeMap<String, Vec<String>>) {
    let (mut labels, mut preds, mut in_mermaid) = (BTreeMap::new(), BTreeMap::new(), false);
    for line in markdown.lines().map(str::trim) {
        if line.starts_with("```mermaid") {
            in_mermaid = true;
        } else if in_mermaid && line.starts_with("```") {
            in_mermaid = false;
        } else if in_mermaid && let Some((left, right)) = line.split_once("-->") {
            let (src_id, src_lbl) = extract_mermaid_node(left.trim());
            let (dst_id, dst_lbl) = extract_mermaid_node(right.trim());
            if !src_id.is_empty() && !src_lbl.is_empty() {
                labels.insert(src_id.clone(), src_lbl);
            }
            if !dst_id.is_empty() && !dst_lbl.is_empty() {
                labels.insert(dst_id.clone(), dst_lbl);
            }
            if !src_id.is_empty() && !dst_id.is_empty() {
                let entry: &mut Vec<String> = preds.entry(dst_id).or_default();
                if !entry.contains(&src_id) {
                    entry.push(src_id);
                }
            }
        }
    }
    (labels, preds)
}

fn extract_mermaid_node(token: &str) -> (String, String) {
    if let Some((id_part, rest)) = token.split_once('[') {
        let lbl = rest.trim_end_matches(']').trim_matches('"').trim();
        (id_part.trim().to_string(), lbl.to_string())
    } else {
        let id = token.split_whitespace().next().unwrap_or("").trim();
        (id.to_string(), String::new())
    }
}

fn parse_gherkin_blocks(markdown: &str) -> Vec<ParsedGherkinBlock> {
    let (mut blocks, mut current) = (Vec::new(), None::<ParsedGherkinBlock>);
    for trimmed in markdown.lines().map(str::trim) {
        if trimmed.starts_with("### ") {
            if let Some(blk) = current.take() {
                blocks.push(blk);
            }
            current = Some(ParsedGherkinBlock {
                header: trimmed.trim_start_matches('#').trim().to_string(),
                ..Default::default()
            });
        } else if let Some(ref mut blk) = current {
            if let Some(rest) = trimmed.strip_prefix("- **Dado**") {
                blk.given = rest.trim().to_string();
            } else if let Some(rest) = trimmed.strip_prefix("- **Quando**") {
                blk.when = rest.trim().to_string();
            } else if let Some(rest) = trimmed
                .strip_prefix("- **Entao**")
                .or_else(|| trimmed.strip_prefix("- **Então**"))
            {
                blk.then = rest.trim().to_string();
            }
        }
    }
    if let Some(blk) = current {
        blocks.push(blk);
    }
    blocks
}

fn parse_traceability_rows(markdown: &str) -> Vec<[String; 6]> {
    markdown
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|') && l.ends_with('|'))
        .filter_map(|l| {
            let c: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
            (c.len() >= 6 && c[0].starts_with("RN-") && c[2].starts_with("TASK-")).then(|| {
                [
                    c[0].to_string(),
                    c[1].to_string(),
                    c[2].to_string(),
                    c[3].to_string(),
                    c[4].to_string(),
                    c[5].to_string(),
                ]
            })
        })
        .collect()
}

fn infer_layer(target_test_file: &str) -> String {
    for (needle, layer) in [
        ("trustkit-ast", "AST"),
        ("trustkit-orch", "ORCH"),
        ("trustkit-gh", "GH"),
        ("trustkit-db", "DB"),
        ("trustkit-cli", "CLI"),
    ] {
        if target_test_file.contains(needle) {
            return layer.to_string();
        }
    }
    "CORE".to_string()
}

fn infer_expected_files(target_test_file: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Some((crate_prefix, test_rel)) = target_test_file.split_once("/tests/") {
        let stem = test_rel
            .strip_suffix("_tests.rs")
            .or_else(|| test_rel.strip_suffix(".rs"))
            .unwrap_or(test_rel);
        files.push(format!("{crate_prefix}/src/{stem}.rs"));
    }
    if !target_test_file.is_empty() {
        files.push(target_test_file.to_string());
    }
    files
}

fn match_gherkin(
    blocks: &[ParsedGherkinBlock],
    rf_id: &str,
    task_id: &str,
    fallback_title: &str,
) -> (String, String, String) {
    if let Some(blk) = blocks.iter().find(|b| {
        (!rf_id.is_empty() && b.header.contains(rf_id))
            || (!task_id.is_empty() && b.header.contains(task_id))
    }) {
        return (blk.given.clone(), blk.when.clone(), blk.then.clone());
    }
    if let Some(first) = blocks.first().filter(|b| !b.given.is_empty()) {
        return (first.given.clone(), first.when.clone(), first.then.clone());
    }
    (
        format!("o contexto estruturado para {task_id}"),
        format!("a execucao de {fallback_title} for acionada"),
        format!("o requisito {rf_id} deve ser comprovado por testes"),
    )
}
