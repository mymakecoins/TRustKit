// Deterministic YAML frontmatter and wikilinks lexical extractor (TASK-002).
// Constitutional Principle I & IV: Zero LLM calls, zero heavy regex, zero panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use trustkit_core::knowledge_entities::{KnowledgeDocument, WikiLink};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMarkdownNote {
    pub title: String,
    pub tags: Vec<String>,
    pub updated_at: String,
    pub body: String,
}

pub fn extract_frontmatter_and_body(raw_md: &str) -> ExtractedMarkdownNote {
    let normalized = raw_md.trim_start_matches('\u{feff}');
    if let Some((fm_block, rest_body)) = split_frontmatter_block(normalized) {
        let mut title = String::new();
        let mut tags: Vec<String> = Vec::new();
        let mut updated_at = String::new();
        let mut in_tags_list = false;

        for line in fm_block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if in_tags_list {
                if let Some(item) = trimmed.strip_prefix('-') {
                    let clean_tag = unquote(item.trim());
                    if !clean_tag.is_empty() && !tags.contains(&clean_tag) {
                        tags.push(clean_tag);
                    }
                    continue;
                } else {
                    in_tags_list = false;
                }
            }

            if let Some((key, raw_val)) = trimmed.split_once(':') {
                let key_norm = key.trim().to_ascii_lowercase();
                let val_norm = raw_val.trim();
                match key_norm.as_str() {
                    "title" => {
                        title = unquote(val_norm);
                    }
                    "updated_at" | "updated" | "date" => {
                        updated_at = unquote(val_norm);
                    }
                    "tags" => {
                        if val_norm.is_empty() {
                            in_tags_list = true;
                        } else {
                            for tag in parse_inline_tags(val_norm) {
                                if !tags.contains(&tag) {
                                    tags.push(tag);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let clean_body = rest_body.trim().to_string();
        if title.is_empty() {
            title = infer_heading_title(&clean_body);
        }

        ExtractedMarkdownNote {
            title,
            tags,
            updated_at,
            body: clean_body,
        }
    } else {
        let clean_body = normalized.trim().to_string();
        ExtractedMarkdownNote {
            title: infer_heading_title(&clean_body),
            tags: Vec::new(),
            updated_at: String::new(),
            body: clean_body,
        }
    }
}

pub fn extract_wikilinks(source_slug: &str, body: &str) -> Vec<WikiLink> {
    let source_clean = normalize_slug(source_slug);
    let mut links: Vec<WikiLink> = Vec::new();
    let mut active_fence: Option<&str> = None;

    for line in body.lines() {
        let trimmed_start = line.trim_start();
        if let Some(fence) = active_fence {
            if trimmed_start.starts_with(fence) {
                active_fence = None;
            }
            continue;
        }
        if trimmed_start.starts_with("```") {
            active_fence = Some("```");
            continue;
        }
        if trimmed_start.starts_with("~~~") {
            active_fence = Some("~~~");
            continue;
        }

        extract_wikilinks_from_line(&source_clean, line, &mut links);
    }

    links
}

pub fn render_markdown_with_frontmatter(
    title: &str,
    tags: &[String],
    updated_at: &str,
    body: &str,
) -> String {
    let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let formatted_tags: Vec<String> = tags
        .iter()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(ToString::to_string)
        .collect();
    let tags_str = formatted_tags.join(", ");
    let clean_body = body.trim();

    format!(
        "---\ntitle: \"{escaped_title}\"\ntags: [{tags_str}]\nupdated_at: \"{updated_at}\"\n---\n\n{clean_body}\n"
    )
}

pub fn render_document_markdown(doc: &KnowledgeDocument) -> String {
    render_markdown_with_frontmatter(
        &doc.title,
        &doc.tags,
        &doc.updated_at,
        &doc.body_markdown,
    )
}

fn split_frontmatter_block(input: &str) -> Option<(&str, &str)> {
    let after_open = input
        .strip_prefix("---\n")
        .or_else(|| input.strip_prefix("---\r\n"))?;

    let mut offset = 0usize;
    for line in after_open.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" {
            let fm = &after_open[..offset];
            let body = &after_open[offset + line.len()..];
            return Some((fm, body));
        }
        offset += line.len();
    }
    None
}

fn unquote(val: &str) -> String {
    let t = val.trim();
    if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
    {
        t[1..t.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        t.to_string()
    }
}

fn parse_inline_tags(raw: &str) -> Vec<String> {
    let inner = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(raw);
    inner
        .split(',')
        .map(|item| unquote(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

fn infer_heading_title(body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(h1) = trimmed.strip_prefix("# ") {
            return h1.trim().to_string();
        }
    }
    String::new()
}

fn extract_wikilinks_from_line(source_slug: &str, line: &str, out: &mut Vec<WikiLink>) {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_inline_code = false;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_inline_code = !in_inline_code;
            i += 1;
            continue;
        }
        if !in_inline_code && i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let start = i + 2;
            if let Some(rel_end) = line[start..].find("]]") {
                let raw_inner = &line[start..start + rel_end];
                let before_alias = raw_inner.split('|').next().unwrap_or("");
                let before_anchor = before_alias.split('#').next().unwrap_or("");
                let target_slug = normalize_slug(before_anchor);
                if !target_slug.is_empty() && !target_slug.contains("..") {
                    let candidate = WikiLink::new(source_slug, target_slug);
                    if !out.contains(&candidate) {
                        out.push(candidate);
                    }
                }
                i = start + rel_end + 2;
                continue;
            }
        }
        i += 1;
    }
}

fn normalize_slug(raw: &str) -> String {
    raw.trim()
        .trim_matches('/')
        .trim_end_matches(".md")
        .replace('\\', "/")
}
