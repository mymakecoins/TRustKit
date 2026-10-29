// Integration tests for Wiki file repository and FTS5 index rebuild (TASK-004).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use tempfile::TempDir;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeSearchOutcome, KnowledgeSearchQuery,
    KnowledgeWriteOutcome,
};
use trustkit_core::ports::KnowledgeRepositoryPort;
use trustkit_db::KnowledgeWikiFsRepository;

#[test]
fn test_rebuild_from_wiki_directory_indexes_all_subfolders_and_wikilinks() {
    let tmp = TempDir::new().expect("create tempdir");
    let wiki_dir = tmp.path().join(".trustkit").join("wiki");
    let db_path = tmp.path().join(".trustkit").join("state.db");

    for subfolder in ["adrs", "specs", "tasks", "incidents"] {
        fs::create_dir_all(wiki_dir.join(subfolder)).expect("create subfolder");
    }

    fs::write(
        wiki_dir.join("adrs/adr-001-hexagonal.md"),
        "---\ntitle: \"ADR 001 Arquitetura Hexagonal\"\ntags: [adr, architecture]\nupdated_at: \"2026-10-29T09:00:00Z\"\n---\n\nIsolamento de dominio puro no TRustKit.\n",
    )
    .expect("write adr note");

    fs::write(
        wiki_dir.join("specs/spec-005.md"),
        "---\ntitle: \"Spec 005 Memoria Perene\"\ntags: [spec, frozen, memory]\nupdated_at: \"2026-10-29T09:10:00Z\"\n---\n\nEspecificacao congelada baseada em [[adrs/adr-001-hexagonal]] e implementada por [[tasks/TASK-004]].\n",
    )
    .expect("write spec note");

    fs::write(
        wiki_dir.join("tasks/TASK-004.md"),
        "---\ntitle: \"Task 004 Repositorio Wiki\"\ntags: [task, tdd-verified, db]\nupdated_at: \"2026-10-29T09:20:00Z\"\n---\n\nImplementacao da reconstrucao FTS5 conforme [[specs/spec-005]].\n",
    )
    .expect("write task note");

    fs::write(
        wiki_dir.join("incidents/incident-005-attempt-1.md"),
        "---\ntitle: \"Incidente Gauntlet Tentativa 1\"\ntags: [incident, gauntlet, post-mortem]\nupdated_at: \"2026-10-29T09:30:00Z\"\n---\n\nDiagnostico sanitizado de falha referenciando [[specs/spec-005]].\n",
    )
    .expect("write incident note");

    // Simulate freshly created state.db with empty FTS5 index
    let repo = KnowledgeWikiFsRepository::open(&wiki_dir, &db_path).expect("open wiki repo");
    let empty_check = repo
        .search_documents(&KnowledgeSearchQuery::new("reconstrucao", 5))
        .expect("initial search");
    assert_eq!(empty_check, KnowledgeSearchOutcome::EmptyKnowledgeList);

    // Execute full rebuild from .trustkit/wiki/
    let summary = repo.rebuild_index().expect("rebuild index from wiki dir");
    assert_eq!(summary.notes_restored, 4);
    assert_eq!(summary.links_restored, 4);

    // Verify searchability across all subfolders after rebuild
    let search_outcome = repo
        .search_documents(&KnowledgeSearchQuery::new("reconstrucao FTS5", 5))
        .expect("search after rebuild");
    assert!(!search_outcome.is_empty());
    assert_eq!(search_outcome.as_slice()[0].slug, "tasks/TASK-004");
    assert!(
        search_outcome.as_slice()[0]
            .adjacent_links
            .contains(&"specs/spec-005".to_string())
    );

    let spec_links = repo
        .query_adjacent_links("specs/spec-005")
        .expect("query spec links");
    assert_eq!(spec_links.len(), 4);

    let all_docs = repo.list_documents().expect("list all documents");
    assert_eq!(all_docs.len(), 4);
}

#[test]
fn test_write_and_read_document_atomic_with_blake3_and_fts5_sync() {
    let tmp = TempDir::new().expect("create tempdir");
    let wiki_dir = tmp.path().join(".trustkit").join("wiki");
    let db_path = tmp.path().join(".trustkit").join("state.db");
    let repo = KnowledgeWikiFsRepository::open(&wiki_dir, &db_path).expect("open wiki repo");

    let doc = KnowledgeDocument::new(
        "adrs/adr-002-storage-strategy",
        "Estrategia de Armazenamento",
        vec!["adr".to_string(), "storage".to_string(), "sqlite".to_string()],
        "Decisao de persistencia atomica com [[adrs/adr-001-hexagonal]].",
        "2026-10-29T12:00:00Z",
        "",
    );

    let outcome = repo.write_document(&doc).expect("write document");
    let written_digest = match outcome {
        KnowledgeWriteOutcome::Success {
            slug,
            file_path,
            blake3_digest,
            indexed_links_count,
        } => {
            assert_eq!(slug, "adrs/adr-002-storage-strategy");
            assert!(file_path.ends_with("adrs/adr-002-storage-strategy.md"));
            assert_eq!(blake3_digest.len(), 64);
            assert_eq!(indexed_links_count, 1);
            blake3_digest
        }
    };

    let lookup = repo
        .read_document("adrs/adr-002-storage-strategy")
        .expect("read document");
    match lookup {
        KnowledgeLookupOutcome::Found(loaded) => {
            assert_eq!(loaded.slug, "adrs/adr-002-storage-strategy");
            assert_eq!(loaded.title, "Estrategia de Armazenamento");
            assert_eq!(loaded.tags, vec!["adr", "storage", "sqlite"]);
            assert_eq!(loaded.updated_at, "2026-10-29T12:00:00Z");
            assert_eq!(loaded.blake3_digest, written_digest);
            assert!(loaded.body_markdown.contains("[[adrs/adr-001-hexagonal]]"));
        }
        KnowledgeLookupOutcome::DocumentNotFound => panic!("Expected document to be found"),
    }

    let missing = repo
        .read_document("adrs/nao-existe")
        .expect("read missing");
    assert_eq!(missing, KnowledgeLookupOutcome::DocumentNotFound);
}

#[test]
fn test_write_and_read_reject_path_traversal_slugs() {
    let tmp = TempDir::new().expect("create tempdir");
    let wiki_dir = tmp.path().join(".trustkit").join("wiki");
    let db_path = tmp.path().join(".trustkit").join("state.db");
    let repo = KnowledgeWikiFsRepository::open(&wiki_dir, &db_path).expect("open wiki repo");

    for hostile_slug in [
        "../outside",
        "adrs/../../etc/passwd",
        "/etc/passwd",
        "adrs\\..\\secret",
        "",
    ] {
        let bad_doc = KnowledgeDocument::new(
            hostile_slug,
            "Hostile",
            vec![],
            "Tentativa de escape.",
            "2026-10-29T12:00:00Z",
            "",
        );
        let write_err = repo
            .write_document(&bad_doc)
            .expect_err("path traversal write must fail");
        assert_eq!(write_err.posix_exit_code(), 1);

        let read_err = repo
            .read_document(hostile_slug)
            .expect_err("path traversal read must fail");
        assert_eq!(read_err.posix_exit_code(), 1);
    }
}
