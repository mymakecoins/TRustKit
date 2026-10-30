// SQLite WAL repository for structural graph snapshot, BLAKE3 incremental scan and Shrink-Guard (TASK-003).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, FileGraphManifest, GraphHealthVerdict, GraphNodeKind,
    StructuralEdge, StructuralNode,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::StructuralGraphRepositoryPort;
use crate::migrations::initialize_database;

pub struct GraphSqliteRepository {
    conn: Mutex<Connection>,
}

impl GraphSqliteRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Mutex::new(conn) }
    }

    pub fn open(db_path: &Path) -> Result<Self, DomainViolation> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(db_path).map_err(|e| boundary_err("Failed opening SQLite db", e))?;
        initialize_database(&conn)?;
        Ok(Self::new(conn))
    }

    pub fn open_from_root(repo_root: &Path) -> Result<Self, DomainViolation> {
        Self::open(&repo_root.join(".trustkit").join("state.db"))
    }

    pub(crate) fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainViolation> {
        self.conn.lock().map_err(|_| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "graph_sqlite_repository".into(),
            details: "Poisoned SQLite mutex".into(),
        })
    }
}

impl StructuralGraphRepositoryPort for GraphSqliteRepository {
    fn detect_changed_files(
        &self,
        current_manifests: &[FileGraphManifest],
    ) -> Result<Vec<String>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT file_path, blake3_digest FROM structural_graph_manifests")
            .map_err(|e| boundary_err("Failed preparing manifests statement", e))?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| boundary_err("Failed querying manifests", e))?;

        let mut saved_hashes = HashMap::new();
        for r in rows {
            let (path, hash) = r.map_err(|e| boundary_err("Failed reading manifest row", e))?;
            saved_hashes.insert(path, hash);
        }

        let mut changed = Vec::new();
        for candidate in current_manifests {
            match saved_hashes.get(&candidate.file_path) {
                Some(existing_hash) if existing_hash == &candidate.blake3_digest => {}
                _ => changed.push(candidate.file_path.clone()),
            }
        }

        Ok(changed)
    }

    fn save_graph_snapshot(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        manifests: &[FileGraphManifest],
        force: bool,
    ) -> Result<GraphHealthVerdict, DomainViolation> {
        let mut conn = self.lock_conn()?;

        let existing_count: usize = conn.query_row(
            "SELECT COUNT(*) FROM structural_graph_nodes",
            [],
            |r| r.get(0),
        ).unwrap_or(0);

        if existing_count > 0 && nodes.len() < existing_count && !force {
            return Ok(GraphHealthVerdict::ShrinkBlocked);
        }

        let tx = conn.transaction().map_err(|e| boundary_err("Failed starting graph transaction", e))?;

        tx.execute("DELETE FROM structural_graph_edges;", [])
            .map_err(|e| boundary_err("Failed clearing edges", e))?;
        tx.execute("DELETE FROM structural_graph_nodes;", [])
            .map_err(|e| boundary_err("Failed clearing nodes", e))?;

        {
            let mut insert_node = tx.prepare(
                "INSERT INTO structural_graph_nodes (node_id, label, kind, source_file, architectural_layer) VALUES (?1, ?2, ?3, ?4, ?5)"
            ).map_err(|e| boundary_err("Failed preparing node insert", e))?;

            for n in nodes {
                insert_node.execute(params![n.node_id, n.label, n.kind.as_str(), n.source_file, n.architectural_layer])
                    .map_err(|e| boundary_err("Failed inserting graph node", e))?;
            }
        }

        {
            let mut insert_edge = tx.prepare(
                "INSERT OR IGNORE INTO structural_graph_edges (source_id, target_id, relation, confidence, source_file) VALUES (?1, ?2, ?3, ?4, ?5)"
            ).map_err(|e| boundary_err("Failed preparing edge insert", e))?;

            for e in edges {
                insert_edge.execute(params![e.source_id, e.target_id, e.relation.as_str(), e.confidence.as_str(), e.source_file])
                    .map_err(|err| boundary_err("Failed inserting graph edge", err))?;
            }
        }

        {
            let mut upsert_manifest = tx.prepare(
                "INSERT INTO structural_graph_manifests (file_path, blake3_digest, node_count, updated_at) \
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) \
                 ON CONFLICT(file_path) DO UPDATE SET blake3_digest = ?2, node_count = ?3, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')"
            ).map_err(|e| boundary_err("Failed preparing manifest upsert", e))?;

            for m in manifests {
                upsert_manifest.execute(params![m.file_path, m.blake3_digest, m.node_count as i64])
                    .map_err(|e| boundary_err("Failed upserting file graph manifest", e))?;
            }
        }

        tx.commit().map_err(|e| boundary_err("Failed committing graph snapshot transaction", e))?;

        Ok(GraphHealthVerdict::Healthy)
    }

    fn load_nodes(&self) -> Result<Vec<StructuralNode>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT node_id, label, kind, source_file, architectural_layer FROM structural_graph_nodes ORDER BY node_id ASC"
        ).map_err(|e| boundary_err("Failed preparing load_nodes statement", e))?;

        let rows = stmt.query_map([], |r| {
            let node_id: String = r.get(0)?;
            let label: String = r.get(1)?;
            let kind_label: String = r.get(2)?;
            let source_file: String = r.get(3)?;
            let layer: String = r.get(4)?;
            Ok(StructuralNode::new(node_id, label, GraphNodeKind::from_str_label(&kind_label), source_file, layer))
        }).map_err(|e| boundary_err("Failed querying structural nodes", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| boundary_err("Failed mapping structural node row", e))?);
        }
        Ok(result)
    }

    fn load_edges(&self) -> Result<Vec<StructuralEdge>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id, relation, confidence, source_file FROM structural_graph_edges ORDER BY source_id, target_id ASC"
        ).map_err(|e| boundary_err("Failed preparing load_edges statement", e))?;

        let rows = stmt.query_map([], |r| {
            let source_id: String = r.get(0)?;
            let target_id: String = r.get(1)?;
            let rel_label: String = r.get(2)?;
            let conf_label: String = r.get(3)?;
            let source_file: String = r.get(4)?;
            Ok(StructuralEdge::new(
                source_id, target_id,
                EdgeRelationKind::from_str_label(&rel_label),
                EdgeConfidence::from_str_label(&conf_label),
                source_file,
            ))
        }).map_err(|e| boundary_err("Failed querying structural edges", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| boundary_err("Failed mapping structural edge row", e))?);
        }
        Ok(result)
    }

    fn load_manifests(&self) -> Result<Vec<FileGraphManifest>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT file_path, blake3_digest, node_count, updated_at FROM structural_graph_manifests ORDER BY file_path ASC"
        ).map_err(|e| boundary_err("Failed preparing load_manifests statement", e))?;

        let rows = stmt.query_map([], |r| {
            let file_path: String = r.get(0)?;
            let blake3_digest: String = r.get(1)?;
            let node_count: i64 = r.get(2)?;
            let updated_at: String = r.get(3)?;
            Ok(FileGraphManifest::new(file_path, blake3_digest, node_count as usize, updated_at))
        }).map_err(|e| boundary_err("Failed querying graph manifests", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| boundary_err("Failed mapping manifest row", e))?);
        }
        Ok(result)
    }
}

fn boundary_err(ctx: &str, err: impl std::fmt::Display) -> DomainViolation {
    DomainViolation::InfrastructureBoundaryFault {
        subsystem: "graph_sqlite_repository".to_string(),
        details: format!("{ctx}: {err}"),
    }
}
