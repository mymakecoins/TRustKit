// SQLite FTS5 indexing, adjacency graph and BM25 lexical search operations (TASK-003).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use rusqlite::{Connection, params};
use trustkit_core::errors::DomainViolation;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeSearchHit, KnowledgeSearchOutcome, KnowledgeSearchQuery, WikiLink,
};

pub fn upsert_knowledge_index(
    conn: &Connection,
    doc: &KnowledgeDocument,
    links: &[WikiLink],
) -> Result<(), DomainViolation> {
    let clean_slug = KnowledgeDocument::validate_slug(&doc.slug)?;
    let tags_joined = doc.tags.join(" ");

    conn.execute(
        "DELETE FROM knowledge_pages_fts WHERE slug = ?1",
        params![clean_slug],
    )
    .map_err(|e| db_err("Failed deleting existing FTS5 row", e))?;

    conn.execute(
        "INSERT INTO knowledge_pages_fts (slug, title, tags, body_content, updated_at, blake3_digest)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            clean_slug,
            doc.title,
            tags_joined,
            doc.body_markdown,
            doc.updated_at,
            doc.blake3_digest,
        ],
    )
    .map_err(|e| db_err("Failed inserting FTS5 knowledge row", e))?;

    conn.execute(
        "DELETE FROM knowledge_wiki_links WHERE source_slug = ?1",
        params![clean_slug],
    )
    .map_err(|e| db_err("Failed clearing previous wiki links", e))?;

    for link in links {
        conn.execute(
            "INSERT OR IGNORE INTO knowledge_wiki_links (source_slug, target_slug) VALUES (?1, ?2)",
            params![link.source_slug, link.target_slug],
        )
        .map_err(|e| db_err("Failed inserting wiki link edge", e))?;
    }

    Ok(())
}

pub fn remove_knowledge_index(conn: &Connection, slug: &str) -> Result<bool, DomainViolation> {
    let clean_slug = KnowledgeDocument::validate_slug(slug)?;
    let deleted = conn
        .execute(
            "DELETE FROM knowledge_pages_fts WHERE slug = ?1",
            params![clean_slug],
        )
        .map_err(|e| db_err("Failed deleting FTS5 knowledge row", e))?;

    conn.execute(
        "DELETE FROM knowledge_wiki_links WHERE source_slug = ?1 OR target_slug = ?1",
        params![clean_slug],
    )
    .map_err(|e| db_err("Failed deleting wiki link edges", e))?;

    Ok(deleted > 0)
}

pub fn clear_knowledge_index(conn: &Connection) -> Result<(), DomainViolation> {
    conn.execute_batch(
        "DELETE FROM knowledge_pages_fts;
         DELETE FROM knowledge_wiki_links;",
    )
    .map_err(|e| db_err("Failed clearing knowledge index tables", e))
}

pub fn query_adjacent_links(
    conn: &Connection,
    slug: &str,
) -> Result<Vec<WikiLink>, DomainViolation> {
    let clean_slug = KnowledgeDocument::validate_slug(slug)?;
    let mut stmt = conn
        .prepare(
            "SELECT source_slug, target_slug
             FROM knowledge_wiki_links
             WHERE source_slug = ?1 OR target_slug = ?1
             ORDER BY source_slug ASC, target_slug ASC",
        )
        .map_err(|e| db_err("Failed preparing adjacent links query", e))?;

    let rows = stmt
        .query_map(params![clean_slug], |row| {
            Ok(WikiLink {
                source_slug: row.get(0)?,
                target_slug: row.get(1)?,
            })
        })
        .map_err(|e| db_err("Failed executing adjacent links query", e))?;

    let mut links = Vec::new();
    for row in rows {
        links.push(row.map_err(|e| db_err("Failed reading adjacent link row", e))?);
    }
    Ok(links)
}

pub fn query_adjacent_slugs(conn: &Connection, slug: &str) -> Result<Vec<String>, DomainViolation> {
    let links = query_adjacent_links(conn, slug)?;
    let mut neighbors = Vec::new();
    for link in links {
        let other = if link.source_slug == slug {
            link.target_slug
        } else {
            link.source_slug
        };
        if !other.is_empty() && other != slug && !neighbors.contains(&other) {
            neighbors.push(other);
        }
    }
    Ok(neighbors)
}

pub fn search_knowledge_fts5(
    conn: &Connection,
    query: &KnowledgeSearchQuery,
) -> Result<KnowledgeSearchOutcome, DomainViolation> {
    let fts_expr = sanitize_fts5_query(&query.query_text);
    if fts_expr.is_empty() {
        return Ok(KnowledgeSearchOutcome::EmptyKnowledgeList);
    }

    let limit = if query.limit == 0 { 10 } else { query.limit };
    let mut stmt = conn
        .prepare(
            "SELECT
                 slug,
                 title,
                 tags,
                 bm25(knowledge_pages_fts, 0.0, 5.0, 3.0, 1.0) AS raw_rank,
                 snippet(knowledge_pages_fts, 3, '[', ']', '...', 18) AS snippet_text
             FROM knowledge_pages_fts
             WHERE knowledge_pages_fts MATCH ?1
             ORDER BY raw_rank ASC, slug ASC
             LIMIT ?2",
        )
        .map_err(|e| db_err("Failed preparing FTS5 BM25 query", e))?;

    let rows = stmt
        .query_map(params![fts_expr, limit as i64], |row| {
            let slug: String = row.get(0)?;
            let title: String = row.get(1)?;
            let tags_str: String = row.get(2)?;
            let raw_rank: f64 = row.get(3)?;
            let snippet: String = row.get(4)?;
            Ok((slug, title, tags_str, raw_rank, snippet))
        })
        .map_err(|e| db_err("Failed executing FTS5 BM25 query", e))?;

    let mut hits = Vec::new();
    for r in rows {
        let (slug, title, tags_str, raw_rank, snippet) =
            r.map_err(|e| db_err("Failed decoding FTS5 row", e))?;

        if !query.required_tags.is_empty() {
            let doc_tags: Vec<&str> = tags_str.split_whitespace().collect();
            let all_present = query
                .required_tags
                .iter()
                .all(|req| doc_tags.iter().any(|t| t.eq_ignore_ascii_case(req)));
            if !all_present {
                continue;
            }
        }

        let adjacent_links = query_adjacent_slugs(conn, &slug)?;
        let positive_score = if raw_rank < 0.0 {
            -raw_rank
        } else {
            raw_rank.max(0.0001)
        };

        hits.push(KnowledgeSearchHit {
            slug,
            title,
            bm25_score: positive_score,
            snippet,
            adjacent_links,
        });
    }

    Ok(KnowledgeSearchOutcome::from_hits(hits))
}

pub fn sanitize_fts5_query(raw_query: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in raw_query.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            current.push(ch);
        } else if !current.is_empty() {
            push_sanitized_token(&mut tokens, &current);
            current.clear();
        }
    }
    if !current.is_empty() {
        push_sanitized_token(&mut tokens, &current);
    }

    if tokens.is_empty() {
        String::new()
    } else {
        tokens.join(" OR ")
    }
}

fn push_sanitized_token(tokens: &mut Vec<String>, raw_token: &str) {
    let trimmed = raw_token.trim_matches('-');
    if trimmed.is_empty() {
        return;
    }
    let upper = trimmed.to_ascii_uppercase();
    if matches!(upper.as_str(), "AND" | "OR" | "NOT" | "NEAR") {
        return;
    }
    let formatted = format!("\"{trimmed}\"");
    if !tokens.contains(&formatted) {
        tokens.push(formatted);
    }
}

fn db_err(context: &str, err: rusqlite::Error) -> DomainViolation {
    DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite_knowledge_fts5".to_string(),
        details: format!("{context}: {err}"),
    }
}
