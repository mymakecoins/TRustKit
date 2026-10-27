// BLAKE3 cryptographic sealing adapter using blake3 and rayon.
// Constitutional Principle VII: Cryptographic Immutability ("The Event Log Proves").
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::ArtifactHasherPort;

#[derive(Debug, Default, Clone, Copy)]
pub struct Blake3ParallelSealerAdapter;

impl Blake3ParallelSealerAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_files_parallel(
        &self,
        paths: &[&str],
    ) -> Result<Vec<(String, String)>, DomainViolation> {
        use rayon::prelude::*;

        paths
            .par_iter()
            .map(|path| {
                let digest = self.compute_file_blake3(path)?;
                Ok(((*path).to_string(), digest))
            })
            .collect()
    }
}

impl ArtifactHasherPort for Blake3ParallelSealerAdapter {
    fn compute_file_blake3(&self, path: &str) -> Result<String, DomainViolation> {
        let file_path = Path::new(path);
        let mut file = File::open(file_path).map_err(|err| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "blake3_hasher".to_string(),
                details: format!("Failed to open file '{path}': {err}"),
            }
        })?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 16384];

        loop {
            let bytes_read = file.read(&mut buffer).map_err(|err| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "blake3_hasher".to_string(),
                    details: format!("Failed reading file '{path}': {err}"),
                }
            })?;

            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(hash.to_hex().to_string())
    }

    fn compute_bytes_blake3(&self, bytes: &[u8]) -> String {
        let hash = blake3::hash(bytes);
        hash.to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compute_bytes_blake3_64_chars() {
        let adapter = Blake3ParallelSealerAdapter::new();
        let digest = adapter.compute_bytes_blake3(b"TRustKit deterministic test");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_file_blake3_success() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("trustkit_test_seal.txt");
        {
            let mut f = File::create(&file_path).expect("failed to create temp file");
            f.write_all(b"Determinism in Gate 2")
                .expect("failed to write");
        }

        let adapter = Blake3ParallelSealerAdapter::new();
        let digest = adapter
            .compute_file_blake3(file_path.to_str().unwrap())
            .expect("hashing should succeed");
        assert_eq!(digest.len(), 64);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_compute_file_blake3_missing_file_maps_error() {
        let adapter = Blake3ParallelSealerAdapter::new();
        let result = adapter.compute_file_blake3("/non_existent_file_trustkit_12345.md");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.posix_exit_code(), 1);
    }
}
