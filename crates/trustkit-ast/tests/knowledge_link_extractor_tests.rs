// Integration/unit tests for deterministic YAML frontmatter and wikilink extraction (TASK-002).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use trustkit_ast::knowledge_link_extractor::{
    extract_frontmatter_and_body, extract_wikilinks, render_markdown_with_frontmatter,
};
use trustkit_core::knowledge_entities::WikiLink;

#[test]
fn test_extract_frontmatter_and_body_parses_title_tags_and_clean_body() {
    let raw = "---\ntitle: \"Estrategia de Armazenamento\"\ntags: [adr, storage, sqlite]\nupdated_at: \"2026-10-29T10:00:00Z\"\n---\n\n# Contexto\nUsamos [[adrs/adr-001]] e [[specs/spec-005|Especificacao 5]].\n";

    let extracted = extract_frontmatter_and_body(raw);
    assert_eq!(extracted.title, "Estrategia de Armazenamento");
    assert_eq!(extracted.tags, vec!["adr", "storage", "sqlite"]);
    assert_eq!(extracted.updated_at, "2026-10-29T10:00:00Z");
    assert!(extracted.body.starts_with("# Contexto"));
    assert!(extracted.body.contains("[[adrs/adr-001]]"));
}

#[test]
fn test_extract_wikilinks_normalizes_aliases_and_deduplicates() {
    let body = "Referencia para [[adrs/adr-001]] e tambem [[specs/spec-005|Especificacao 5]].\nRepeticao de [[adrs/adr-001.md]] e [[specs/spec-005#cenario-1|Link com ancora]].";

    let links = extract_wikilinks("tasks/TASK-002", body);
    assert_eq!(
        links,
        vec![
            WikiLink::new("tasks/TASK-002", "adrs/adr-001"),
            WikiLink::new("tasks/TASK-002", "specs/spec-005"),
        ]
    );
}

#[test]
fn test_extract_wikilinks_ignores_fenced_code_blocks_and_inline_code() {
    let body = r#"
Texto valido com [[adrs/adr-001]].

```rust
fn main() {
    let fake = "[[nao-link-rust]]";
}
```

~~~bash
echo "[[nao-link-bash]]"
~~~

Tambem ignorar inline `[[nao-link-inline]]` mas capturar [[specs/spec-005|Spec 5]].
"#;

    let links = extract_wikilinks("notes/indice", body);
    assert_eq!(
        links,
        vec![
            WikiLink::new("notes/indice", "adrs/adr-001"),
            WikiLink::new("notes/indice", "specs/spec-005"),
        ]
    );
}

#[test]
fn test_render_markdown_with_frontmatter_roundtrip() {
    let rendered = render_markdown_with_frontmatter(
        "Decisao de Arquitetura",
        &["adr".to_string(), "hexagonal".to_string()],
        "2026-10-29T11:30:00Z",
        "Corpo com [[specs/spec-005]].",
    );

    let parsed = extract_frontmatter_and_body(&rendered);
    assert_eq!(parsed.title, "Decisao de Arquitetura");
    assert_eq!(parsed.tags, vec!["adr", "hexagonal"]);
    assert_eq!(parsed.updated_at, "2026-10-29T11:30:00Z");
    assert_eq!(parsed.body, "Corpo com [[specs/spec-005]].");
}
