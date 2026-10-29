// Integration tests for SQLite FTS5 knowledge indexing, adjacency graph and BM25 search (TASK-003).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use tempfile::TempDir;
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeSearchOutcome, KnowledgeSearchQuery, WikiLink,
};
use trustkit_db::SqliteWalRepositoryAdapter;

#[test]
fn test_fts5_bm25_search_ranks_relevance_and_populates_adjacent_links() {
    let tmp = TempDir::new().expect("create tempdir");
    let db_path = tmp.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open sqlite wal repo");

    let doc1 = KnowledgeDocument::new(
        "adrs/adr-001-architecture",
        "Arquitetura Hexagonal Pura",
        vec!["adr".to_string(), "hexagonal".to_string()],
        "Isolamento estrito do dominio sem dependencias externas.",
        "2026-10-29T10:00:00Z",
        "hash_001",
    );
    let doc2 = KnowledgeDocument::new(
        "adrs/adr-002-storage-strategy",
        "Estrategia de Armazenamento SQLite WAL e FTS5",
        vec!["adr".to_string(), "sqlite".to_string(), "fts5".to_string()],
        "Persistencia deterministica em SQLite WAL com indice FTS5 e ranqueamento BM25. Veja [[adrs/adr-001-architecture]].",
        "2026-10-29T10:05:00Z",
        "hash_002",
    );
    let doc3 = KnowledgeDocument::new(
        "specs/spec-005",
        "Especificacao de Memoria Perene",
        vec!["spec".to_string(), "sqlite".to_string()],
        "Nota que menciona SQLite brevemente e aponta para [[adrs/adr-002-storage-strategy]].",
        "2026-10-29T10:10:00Z",
        "hash_003",
    );

    repo.upsert_knowledge_index(&doc1, &[])
        .expect("index doc1");
    repo.upsert_knowledge_index(
        &doc2,
        &[WikiLink::new(
            "adrs/adr-002-storage-strategy",
            "adrs/adr-001-architecture",
        )],
    )
    .expect("index doc2");
    repo.upsert_knowledge_index(
        &doc3,
        &[WikiLink::new(
            "specs/spec-005",
            "adrs/adr-002-storage-strategy",
        )],
    )
    .expect("index doc3");

    let query = KnowledgeSearchQuery::new("sqlite wal fts5", 5);
    let outcome = repo.search_knowledge_fts5(&query).expect("fts5 search");

    match outcome {
        KnowledgeSearchOutcome::Results(hits) => {
            assert!(!hits.is_empty());
            assert_eq!(hits[0].slug, "adrs/adr-002-storage-strategy");
            assert!(hits[0].bm25_score > 0.0);
            assert!(!hits[0].snippet.is_empty());
            assert!(
                hits[0]
                    .adjacent_links
                    .contains(&"adrs/adr-001-architecture".to_string())
            );
            assert!(
                hits[0]
                    .adjacent_links
                    .contains(&"specs/spec-005".to_string())
            );
        }
        KnowledgeSearchOutcome::EmptyKnowledgeList => {
            panic!("Expected search results for 'sqlite wal fts5'");
        }
    }

    let links = repo
        .query_adjacent_links("adrs/adr-002-storage-strategy")
        .expect("query adjacent links");
    assert_eq!(links.len(), 2);
}

#[test]
fn test_fts5_search_sanitizes_reserved_characters_without_sql_error() {
    let tmp = TempDir::new().expect("create tempdir");
    let db_path = tmp.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open sqlite wal repo");

    let doc = KnowledgeDocument::new(
        "incidents/inc-001",
        "Falha de Sintaxe SQL",
        vec!["incident".to_string()],
        "Diagnostico de erro em expressao complexa.",
        "2026-10-29T10:15:00Z",
        "hash_inc_001",
    );
    repo.upsert_knowledge_index(&doc, &[]).expect("index doc");

    let hostile_query = KnowledgeSearchQuery::new(
        "\"unclosed quote (OR AND NOT * : ^ ~ - + [ ] { } \\ / ???) sintaxe",
        5,
    );
    let outcome = repo
        .search_knowledge_fts5(&hostile_query)
        .expect("sanitized search should not fail");
    assert!(!outcome.is_empty());
    assert_eq!(outcome.as_slice()[0].slug, "incidents/inc-001");

    let symbols_only_query = KnowledgeSearchQuery::new("\"\" (( )) *** ::: ---", 5);
    let empty_outcome = repo
        .search_knowledge_fts5(&symbols_only_query)
        .expect("symbols-only query should return EmptyKnowledgeList");
    assert_eq!(empty_outcome, KnowledgeSearchOutcome::EmptyKnowledgeList);
}

#[test]
fn test_fts5_upsert_and_remove_updates_index_and_links() {
    let tmp = TempDir::new().expect("create tempdir");
    let db_path = tmp.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open sqlite wal repo");

    let doc_v1 = KnowledgeDocument::new(
        "tasks/TASK-003",
        "Versao Inicial",
        vec!["task".to_string()],
        "Conteudo alfa exclusivo.",
        "2026-10-29T10:00:00Z",
        "hash_v1",
    );
    repo.upsert_knowledge_index(&doc_v1, &[WikiLink::new("tasks/TASK-003", "specs/spec-005")])
        .expect("index v1");

    let doc_v2 = KnowledgeDocument::new(
        "tasks/TASK-003",
        "Versao Atualizada",
        vec!["task".to_string()],
        "Conteudo beta substituido.",
        "2026-10-29T10:20:00Z",
        "hash_v2",
    );
    repo.upsert_knowledge_index(&doc_v2, &[])
        .expect("index v2");

    let old_search = repo
        .search_knowledge_fts5(&KnowledgeSearchQuery::new("alfa", 5))
        .expect("search old term");
    assert_eq!(old_search, KnowledgeSearchOutcome::EmptyKnowledgeList);

    let new_search = repo
        .search_knowledge_fts5(&KnowledgeSearchQuery::new("beta", 5))
        .expect("search new term");
    assert_eq!(new_search.as_slice().len(), 1);

    let removed = repo
        .remove_knowledge_index("tasks/TASK-003")
        .expect("remove index");
    assert!(removed);

    let after_remove = repo
        .search_knowledge_fts5(&KnowledgeSearchQuery::new("beta", 5))
        .expect("search after remove");
    assert_eq!(after_remove, KnowledgeSearchOutcome::EmptyKnowledgeList);
}
