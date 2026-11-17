// Deterministic AST/Markdown extractor for GitHub Issue body (TASK-004, RN-004, RF-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{AtomicTaskDraft, AtomizationSource};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::TaskAtomizerPort;

#[derive(Debug, Default, Clone)]
pub struct IssueBodyAtomizer;

struct TableTaskRow {
    rn: String,
    rf: String,
    task_id: String,
    node: String,
    test_file: String,
    evidence: String,
    title: String,
}

impl IssueBodyAtomizer {
    pub fn new() -> Self {
        Self
    }

    pub fn atomize_issue(
        issue_number: u64,
        issue_title: &str,
        body_markdown: &str,
    ) -> Result<Vec<AtomicTaskDraft>, DomainViolation> {
        let (given, when, then) = parse_gherkin_scenario(body_markdown);
        let table_rows = parse_table_tasks(body_markdown);

        if !table_rows.is_empty() {
            let mut drafts = Vec::with_capacity(table_rows.len());
            for (idx, row) in table_rows.into_iter().enumerate() {
                let task_num = idx + 1;
                let layer = infer_layer(&row.test_file);
                let expected = infer_expected_files(&row.test_file);
                let rn = if row.rn.is_empty() {
                    format!("RN-{issue_number}-{task_num:02}")
                } else {
                    row.rn
                };
                let rf = if row.rf.is_empty() {
                    format!("RF-{issue_number}-{task_num:02}")
                } else {
                    row.rf
                };
                let ev = if row.evidence.is_empty() {
                    format!("EV-{task_num:03}-issue-{issue_number}.json")
                } else {
                    row.evidence
                };

                drafts.push(AtomicTaskDraft {
                    task_id: row.task_id.clone(),
                    title: row.title,
                    architectural_layer: layer,
                    depends_on: vec![],
                    expected_files: expected,
                    target_test_file: row.test_file,
                    context_summary: format!(
                        "{} (GitHub Issue #{issue_number}) atendendo {rn} e {rf}.",
                        row.node
                    ),
                    given_clause: given.clone(),
                    when_clause: when.clone(),
                    then_clause: then.clone(),
                    business_rule_id: rn,
                    functional_req_id: rf,
                    mermaid_node: row.node,
                    evidence_artifact: ev,
                });
            }
            drafts.sort_by(|a, b| a.task_id.cmp(&b.task_id));
            return Ok(drafts);
        }

        let checklist = parse_checklist_tasks(body_markdown);
        if !checklist.is_empty() {
            let mut drafts = Vec::with_capacity(checklist.len());
            for (idx, (task_id, title)) in checklist.into_iter().enumerate() {
                let task_num = idx + 1;
                let rn = format!("RN-{issue_number}-{task_num:02}");
                let rf = format!("RF-{issue_number}-{task_num:02}");
                let ev = format!("EV-{task_num:03}-issue-{issue_number}.json");
                let test_file = format!(
                    "tests/{}_tests.rs",
                    task_id.to_lowercase().replace('-', "_")
                );
                let layer = infer_layer(&test_file);
                let expected = infer_expected_files(&test_file);

                drafts.push(AtomicTaskDraft {
                    task_id,
                    title: title.clone(),
                    architectural_layer: layer,
                    depends_on: vec![],
                    expected_files: expected,
                    target_test_file: test_file,
                    context_summary: format!("{title} (GitHub Issue #{issue_number})"),
                    given_clause: given.clone(),
                    when_clause: when.clone(),
                    then_clause: then.clone(),
                    business_rule_id: rn,
                    functional_req_id: rf,
                    mermaid_node: "IssueTask".to_string(),
                    evidence_artifact: ev,
                });
            }
            drafts.sort_by(|a, b| a.task_id.cmp(&b.task_id));
            return Ok(drafts);
        }

        let task_id = "TASK-001".to_string();
        let test_file = format!("tests/issue_{issue_number}_tests.rs");
        let layer = infer_layer(&test_file);
        let expected = infer_expected_files(&test_file);
        let draft = AtomicTaskDraft {
            task_id,
            title: issue_title.to_string(),
            architectural_layer: layer,
            depends_on: vec![],
            expected_files: expected,
            target_test_file: test_file,
            context_summary: format!("{issue_title} (GitHub Issue #{issue_number})"),
            given_clause: given,
            when_clause: when,
            then_clause: then,
            business_rule_id: format!("RN-{issue_number}"),
            functional_req_id: format!("RF-{issue_number}"),
            mermaid_node: "GitHubIssue".to_string(),
            evidence_artifact: format!("EV-001-issue-{issue_number}.json"),
        };

        Ok(vec![draft])
    }
}

impl TaskAtomizerPort for IssueBodyAtomizer {
    fn atomize_source(
        &self,
        source: &AtomizationSource,
    ) -> Result<Vec<AtomicTaskDraft>, DomainViolation> {
        match source {
            AtomizationSource::GitHubIssueBody {
                issue_number,
                title,
                body_markdown,
            } => Self::atomize_issue(*issue_number, title, body_markdown),
            AtomizationSource::SpecificationMarkdown {
                markdown_content,
                spec_path,
            } => Self::atomize_issue(0, spec_path, markdown_content),
        }
    }
}

fn parse_table_tasks(markdown: &str) -> Vec<TableTaskRow> {
    let mut rows = Vec::new();
    for line in markdown.lines().map(str::trim) {
        if !line.contains('|') || !line.contains("TASK-") {
            continue;
        }
        let cols: Vec<&str> = line
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if cols.len() < 3 {
            continue;
        }

        let mut rn = String::new();
        let mut rf = String::new();
        let mut task_id = String::new();
        let mut node = String::new();
        let mut test_file = String::new();
        let mut evidence = String::new();

        for col in &cols {
            if col.starts_with("RN-") {
                rn = col.to_string();
            } else if col.starts_with("RF-") {
                rf = col.to_string();
            } else if col.starts_with("TASK-") {
                task_id = col.to_string();
            } else if col.ends_with(".rs") {
                test_file = col.to_string();
            } else if col.ends_with(".json") {
                evidence = col.to_string();
            } else if node.is_empty() {
                node = col.to_string();
            }
        }

        if !task_id.is_empty() {
            let title = if node.is_empty() {
                format!("Task {task_id}")
            } else {
                node.clone()
            };
            rows.push(TableTaskRow {
                rn,
                rf,
                task_id,
                node,
                test_file,
                evidence,
                title,
            });
        }
    }
    rows
}

fn parse_checklist_tasks(markdown: &str) -> Vec<(String, String)> {
    let mut items = Vec::new();
    for line in markdown.lines().map(str::trim) {
        let is_checklist = line.starts_with("- [ ] TASK-")
            || line.starts_with("- [x] TASK-")
            || line.starts_with("* [ ] TASK-");
        if is_checklist {
            let after_box = if let Some(pos) = line.find("TASK-") {
                &line[pos..]
            } else {
                continue;
            };
            if let Some((id_part, title_part)) = after_box.split_once(':') {
                items.push((id_part.trim().to_string(), title_part.trim().to_string()));
            } else {
                items.push((after_box.trim().to_string(), after_box.trim().to_string()));
            }
        }
    }
    items
}

fn parse_gherkin_scenario(markdown: &str) -> (String, String, String) {
    let mut given = String::new();
    let mut when = String::new();
    let mut then = String::new();

    for line in markdown.lines().map(str::trim) {
        if line.starts_with("- **Dado**")
            || line.starts_with("**Dado**")
            || line.starts_with("Given ")
        {
            given = clean_clause(line);
        } else if line.starts_with("- **Quando**")
            || line.starts_with("**Quando**")
            || line.starts_with("When ")
        {
            when = clean_clause(line);
        } else if line.starts_with("- **Entao**")
            || line.starts_with("**Entao**")
            || line.starts_with("Then ")
        {
            then = clean_clause(line);
        }
    }

    if given.is_empty() {
        given = "a valid GitHub Issue payload".to_string();
    }
    if when.is_empty() {
        when = "issue body is processed by atomizer".to_string();
    }
    if then.is_empty() {
        then = "derived tasks maintain issue traceability".to_string();
    }

    (given, when, then)
}

fn clean_clause(raw: &str) -> String {
    raw.trim_start_matches('-')
        .trim_start_matches('*')
        .trim()
        .trim_start_matches("**Dado**")
        .trim_start_matches("**Quando**")
        .trim_start_matches("**Entao**")
        .trim_start_matches("Given")
        .trim_start_matches("When")
        .trim_start_matches("Then")
        .trim_start_matches(':')
        .trim()
        .to_string()
}

fn infer_layer(path: &str) -> String {
    if path.contains("trustkit-core") {
        "CORE".to_string()
    } else if path.contains("trustkit-db") {
        "DB".to_string()
    } else if path.contains("trustkit-gh") {
        "GH".to_string()
    } else if path.contains("trustkit-ast") {
        "AST".to_string()
    } else if path.contains("trustkit-orch") {
        "ORCH".to_string()
    } else if path.contains("trustkit-cli") {
        "CLI".to_string()
    } else {
        "GH".to_string()
    }
}

fn infer_expected_files(test_file: &str) -> Vec<String> {
    if test_file.contains("/tests/") && test_file.ends_with("_tests.rs") {
        let src_file = test_file
            .replace("/tests/", "/src/")
            .replace("_tests.rs", ".rs");
        vec![src_file, test_file.to_string()]
    } else {
        vec![test_file.to_string()]
    }
}
