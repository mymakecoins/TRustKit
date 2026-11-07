// SQLite WAL pragma initialization and 5-table 100% NOT NULL DDL schema.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use rusqlite::Connection;
use trustkit_core::errors::DomainViolation;

pub fn initialize_database(conn: &Connection) -> Result<(), DomainViolation> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;

         CREATE TABLE IF NOT EXISTS sdd_cycles (
             cycle_id                TEXT PRIMARY KEY NOT NULL,
             issue_linkage_kind      TEXT NOT NULL CHECK (issue_linkage_kind IN ('LINKED_ISSUE', 'UNLINKED_LOCAL_CYCLE')),
             issue_number            INTEGER NOT NULL DEFAULT 0,
             issue_url               TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
             project_board_id        TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
             project_item_id         TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
             board_column            TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
             methodology_profile     TEXT NOT NULL CHECK (methodology_profile IN ('STANDARD_GAUNTLET_V2', 'STRUCTURED_SDD')),
             current_gate            INTEGER NOT NULL DEFAULT 0 CHECK (current_gate BETWEEN 0 AND 6),
             lifecycle_status        TEXT NOT NULL CHECK (lifecycle_status IN ('ACTIVE', 'PAUSED', 'CIRCUIT_BREAKER_TRIPPED', 'CERTIFIED', 'MERGED')),
             gauntlet_attempt        INTEGER NOT NULL DEFAULT 0 CHECK (gauntlet_attempt BETWEEN 0 AND 5),
             spec_dir                TEXT NOT NULL,
             spec_file_path          TEXT NOT NULL,
             spec_lock_state         TEXT NOT NULL DEFAULT 'DRAFT_UNVERIFIED' CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN')),
             spec_blake3_digest      TEXT NOT NULL DEFAULT 'UNSEALED_DRAFT',
             work_branch             TEXT NOT NULL,
             base_commit_sha         TEXT NOT NULL,
             pre_existing_dirty_json TEXT NOT NULL DEFAULT '[]',
             created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
             updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );

         CREATE TABLE IF NOT EXISTS artifact_seals (
             seal_id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
             cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
             file_path               TEXT NOT NULL,
             artifact_category       TEXT NOT NULL CHECK (artifact_category IN ('SPEC', 'CONSTITUTION', 'PRE_EXISTING_TEST', 'FOREIGN_ISSUE_EVIDENCE')),
             blake3_digest           TEXT NOT NULL,
             ast_assertion_count     INTEGER NOT NULL DEFAULT 0,
             contract_unlock_state   TEXT NOT NULL DEFAULT 'SEALED' CHECK (contract_unlock_state IN ('SEALED', 'UNLOCKED_BY_HUMAN')),
             unlock_justification    TEXT NOT NULL DEFAULT 'NONE',
             UNIQUE(cycle_id, file_path)
         );

         CREATE TABLE IF NOT EXISTS traceability_links (
             trace_id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
             cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
             business_rule_id        TEXT NOT NULL,
             functional_req_id       TEXT NOT NULL,
             task_id                 TEXT NOT NULL,
             mermaid_journey_node    TEXT NOT NULL,
             playwright_spec_path    TEXT NOT NULL,
             evidence_screenshot     TEXT NOT NULL,
             verification_state      TEXT NOT NULL DEFAULT 'PENDING_GATE5' CHECK (verification_state IN ('PENDING_GATE5', 'VERIFIED_PASS'))
         );

         CREATE TABLE IF NOT EXISTS atomic_tasks (
             cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
             task_id                 TEXT NOT NULL,
             title                   TEXT NOT NULL,
             architectural_layer     TEXT NOT NULL,
             wave_index              INTEGER NOT NULL CHECK (wave_index >= 1),
             depends_on_json         TEXT NOT NULL DEFAULT '[]',
             expected_files_json     TEXT NOT NULL DEFAULT '[]',
             target_test_file        TEXT NOT NULL,
             tdd_lifecycle_state     TEXT NOT NULL DEFAULT 'AWAITING_RED_PHASE' CHECK (tdd_lifecycle_state IN ('AWAITING_RED_PHASE', 'RED_VERIFIED', 'GREEN_VERIFIED', 'COMMITTED')),
             red_evidence_id         INTEGER NOT NULL DEFAULT 0,
             green_evidence_id       INTEGER NOT NULL DEFAULT 0,
             worktree_allocation     TEXT NOT NULL DEFAULT 'NOT_PROVISIONED',
             commit_sha              TEXT NOT NULL DEFAULT 'UNCOMMITTED',
             PRIMARY KEY (cycle_id, task_id)
         );

         CREATE TABLE IF NOT EXISTS execution_evidences (
             evidence_id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
             cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
             gate_stage              INTEGER NOT NULL,
             gauntlet_attempt        INTEGER NOT NULL DEFAULT 0,
             actor_role              TEXT NOT NULL,
             evidence_kind           TEXT NOT NULL,
             executed_command        TEXT NOT NULL DEFAULT 'INTERNAL_AST_EVALUATION',
             exit_code               INTEGER NOT NULL DEFAULT 0,
             duration_ms             INTEGER NOT NULL,
             sanitized_stdout_tail   TEXT NOT NULL DEFAULT '',
             sanitized_stderr_tail   TEXT NOT NULL DEFAULT '',
             output_blake3_digest    TEXT NOT NULL,
             verdict                 TEXT NOT NULL CHECK (verdict IN ('PASS', 'FAIL', 'INFO', 'BLOCKED')),
             recorded_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );

         CREATE INDEX IF NOT EXISTS idx_evidences_cycle_gate ON execution_evidences(cycle_id, gate_stage, verdict);
         CREATE INDEX IF NOT EXISTS idx_traceability_cycle ON traceability_links(cycle_id, business_rule_id);

         CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_pages_fts USING fts5(
             slug UNINDEXED,
             title,
             tags,
             body_content,
             updated_at UNINDEXED,
             blake3_digest UNINDEXED,
             tokenize = 'porter unicode61'
         );

         CREATE TABLE IF NOT EXISTS knowledge_wiki_links (
             source_slug TEXT NOT NULL,
             target_slug TEXT NOT NULL,
             PRIMARY KEY (source_slug, target_slug)
         );

         CREATE INDEX IF NOT EXISTS idx_wiki_links_target ON knowledge_wiki_links(target_slug);

         CREATE TABLE IF NOT EXISTS structural_graph_manifests (
             file_path           TEXT PRIMARY KEY NOT NULL,
             blake3_digest       TEXT NOT NULL,
             node_count          INTEGER NOT NULL DEFAULT 0,
             updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );

         CREATE TABLE IF NOT EXISTS structural_graph_nodes (
             node_id             TEXT PRIMARY KEY NOT NULL,
             label               TEXT NOT NULL,
             kind                TEXT NOT NULL,
             source_file         TEXT NOT NULL,
             architectural_layer TEXT NOT NULL,
             created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );

         CREATE INDEX IF NOT EXISTS idx_graph_nodes_file ON structural_graph_nodes(source_file);
         CREATE INDEX IF NOT EXISTS idx_graph_nodes_layer ON structural_graph_nodes(architectural_layer);

         CREATE TABLE IF NOT EXISTS structural_graph_edges (
             source_id           TEXT NOT NULL,
             target_id           TEXT NOT NULL,
             relation            TEXT NOT NULL,
             confidence          TEXT NOT NULL CHECK (confidence IN ('EXTRACTED', 'INFERRED', 'AMBIGUOUS')),
             source_file         TEXT NOT NULL,
             created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
             PRIMARY KEY (source_id, target_id, relation)
         );

         CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON structural_graph_edges(source_id);
         CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON structural_graph_edges(target_id);
         CREATE INDEX IF NOT EXISTS idx_graph_edges_file ON structural_graph_edges(source_file);

         CREATE TABLE IF NOT EXISTS milestones (
             milestone_id        TEXT PRIMARY KEY NOT NULL,
             plan_id             TEXT NOT NULL,
             sequence_index      INTEGER NOT NULL,
             title               TEXT NOT NULL,
             slug                TEXT NOT NULL,
             file_path           TEXT NOT NULL,
             goal                TEXT NOT NULL,
             dependencies_json   TEXT NOT NULL DEFAULT '[]',
             capabilities_json   TEXT NOT NULL DEFAULT '[]',
             status              TEXT NOT NULL CHECK (status IN ('Planned', 'InProgress', 'Completed', 'Cancelled')),
             created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
             updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );

         CREATE INDEX IF NOT EXISTS idx_milestones_plan ON milestones(plan_id);
         CREATE INDEX IF NOT EXISTS idx_milestones_status ON milestones(status);"
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite_migrations".into(),
        details: format!("Failed initializing database schema: {e}"),
    })
}
