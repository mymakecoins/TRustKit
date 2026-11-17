// Production implementation for TASK-001
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::net::TcpListener;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaveBoardMetadata {
    pub pid: u32,
    pub port: u16,
    pub url: String,
    pub started_at: String,
}

impl WaveBoardMetadata {
    pub fn new(pid: u32, port: u16, started_at: String) -> Self {
        Self {
            pid,
            port,
            url: format!("http://127.0.0.1:{port}"),
            started_at,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(p, json)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub struct WaveBoardPortAllocator;

impl WaveBoardPortAllocator {
    pub fn allocate_free_port() -> io::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        Ok(addr.port())
    }

    pub fn allocate_and_record_metadata<P: AsRef<Path>>(
        pid: u32,
        metadata_path: P,
        now_iso: String,
    ) -> io::Result<WaveBoardMetadata> {
        let port = Self::allocate_free_port()?;
        let metadata = WaveBoardMetadata::new(pid, port, now_iso);
        metadata.save_to_file(metadata_path)?;
        Ok(metadata)
    }
}
