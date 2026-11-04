// Deterministic Markdown parser for atomic tasks (TASK-NNN.md).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use std::path::Path;
use trustkit_core::entities::TaskNode;
use trustkit_core::errors::DomainViolation;

pub fn parse_tasks_dir(dir_path: &Path) -> Result<Vec<TaskNode>, DomainViolation> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_parser".to_string(),
            details: format!(
                "Tasks directory does not exist or is not a directory: {}",
                dir_path.display()
            ),
        });
    }

    let mut entries = Vec::new();
    let read_dir =
        fs::read_dir(dir_path).map_err(|err| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_parser".to_string(),
            details: format!("Failed reading tasks directory: {err}"),
        })?;

    for entry_res in read_dir {
        let entry = entry_res.map_err(|err| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_parser".to_string(),
            details: format!("Failed reading entry in tasks dir: {err}"),
        })?;
        let path = entry.path();
        if path.is_file() {
            let is_task_md = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with("TASK-") && name.ends_with(".md"));
            if is_task_md {
                entries.push(path);
            }
        }
    }

    entries.sort();

    let mut tasks = Vec::with_capacity(entries.len());
    for path in entries {
        let content = fs::read_to_string(&path).map_err(|err| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "task_parser".to_string(),
                details: format!("Failed reading file {}: {err}", path.display()),
            }
        })?;
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("TASK-UNKNOWN");
        let task = parse_task_content(&content, file_stem);
        tasks.push(task);
    }

    Ok(tasks)
}

pub fn parse_task_content(content: &str, fallback_id: &str) -> TaskNode {
    let mut task_id = String::new();
    let mut title = String::new();
    let mut architectural_layer = "CORE".to_string();
    let mut target_test_file = String::new();
    let mut depends_on = Vec::new();
    let mut expected_files = Vec::new();

    let mut section_mode = "";

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            section_mode = "";
            let h_text = trimmed.trim_start_matches('#').trim();
            if h_text.to_uppercase().starts_with("TASK-") {
                if let Some((id_part, title_part)) = h_text.split_once(':') {
                    task_id = id_part.trim().to_uppercase();
                    title = title_part.trim().to_string();
                } else {
                    task_id = h_text.trim().to_uppercase();
                    title = h_text.trim().to_string();
                }
            }
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.starts_with("depends_on:") || lower.starts_with("dependencies:") {
            let val = trimmed.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
            if val.starts_with('[') && val.ends_with(']') {
                let inner = &val[1..val.len() - 1];
                for item in inner.split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() {
                        depends_on.push(cleaned.to_string());
                    }
                }
                section_mode = "";
            } else if val.is_empty() {
                section_mode = "depends_on";
            } else {
                depends_on.push(val.trim_matches(|c| c == '\'' || c == '"').to_string());
                section_mode = "";
            }
            continue;
        }

        if lower.starts_with("arquivos esperados:") || lower.starts_with("expected_files:") {
            let val = trimmed.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
            if val.starts_with('[') && val.ends_with(']') {
                let inner = &val[1..val.len() - 1];
                for item in inner.split(',') {
                    let cleaned = item.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() {
                        expected_files.push(cleaned.to_string());
                    }
                }
                section_mode = "";
            } else {
                section_mode = "expected_files";
            }
            continue;
        }

        if lower.starts_with("layer:") || lower.starts_with("architectural_layer:") {
            let val = trimmed
                .split_once(':')
                .map(|(_, v)| v.trim())
                .unwrap_or("CORE");
            architectural_layer = val.to_uppercase();
            section_mode = "";
            continue;
        }

        if lower.starts_with("test_file:") || lower.starts_with("target_test_file:") {
            let val = trimmed.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
            target_test_file = val.to_string();
            section_mode = "";
            continue;
        }

        if trimmed.starts_with('-') || trimmed.starts_with('*') {
            let item = trimmed[1..].trim().trim_matches(|c| c == '\'' || c == '"');
            if section_mode == "depends_on" && !item.is_empty() {
                depends_on.push(item.to_string());
            } else if section_mode == "expected_files" && !item.is_empty() {
                expected_files.push(item.to_string());
            }
        }
    }

    if task_id.is_empty() {
        task_id = fallback_id.to_uppercase();
    }
    if title.is_empty() {
        title = task_id.clone();
    }

    TaskNode {
        task_id,
        title,
        architectural_layer,
        depends_on,
        expected_files,
        target_test_file,
    }
}
