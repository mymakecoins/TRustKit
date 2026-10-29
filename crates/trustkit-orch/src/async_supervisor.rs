// AsyncExecutionSupervisor implementing AsyncExecutionSupervisorPort.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use regex::Regex;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, SupervisedExecutionRequest, SupervisedExecutionOutcome,
};

static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"ghp_[A-Za-z0-9_]{10,}").unwrap(), "[REDACTED_SECRET]"),
        (Regex::new(r"gho_[A-Za-z0-9_]{10,}").unwrap(), "[REDACTED_SECRET]"),
        (Regex::new(r"github_pat_[A-Za-z0-9_]{10,}").unwrap(), "[REDACTED_SECRET]"),
        (Regex::new(r"(?i)bearer\s+[A-Za-z0-9_\-\.]{15,}").unwrap(), "Bearer [REDACTED_SECRET]"),
        (Regex::new(r"(?i)(password|token|secret)=([^&\s]+)").unwrap(), "$1=[REDACTED_SECRET]"),
        (
            Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----").unwrap(),
            "[REDACTED_PRIVATE_KEY]",
        ),
    ]
});

pub fn redact_secrets(input: &str) -> String {
    let mut sanitized = input.to_string();
    for (regex, replacement) in SECRET_PATTERNS.iter() {
        sanitized = regex.replace_all(&sanitized, *replacement).to_string();
    }
    sanitized
}

#[derive(Debug, Default, Clone)]
pub struct AsyncExecutionSupervisor;

impl AsyncExecutionSupervisor {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_supervised_async(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        let start = Instant::now();
        let timeout = Duration::from_millis(if request.timeout_ms == 0 { 60_000 } else { request.timeout_ms });

        let mut cmd = Command::new(&request.command);
        cmd.args(&request.args);
        if !request.working_dir.is_empty() {
            cmd.current_dir(Path::new(&request.working_dir));
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "async_supervisor".to_string(),
            details: format!("Failed to spawn command '{}': {e}", request.command),
        })?;

        let mut stdout_handle = child.stdout.take();
        let mut stderr_handle = child.stderr.take();

        let wait_proc = async {
            let mut stdout_bytes = Vec::new();
            let mut stderr_bytes = Vec::new();

            if let Some(mut out) = stdout_handle.take() {
                let _ = out.read_to_end(&mut stdout_bytes).await;
            }
            if let Some(mut err) = stderr_handle.take() {
                let _ = err.read_to_end(&mut stderr_bytes).await;
            }

            let status = child.wait().await;
            (status, stdout_bytes, stderr_bytes)
        };

        let result = tokio::time::timeout(timeout, wait_proc).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((Ok(status), stdout_bytes, stderr_bytes)) => {
                let exit_code = status.code().unwrap_or(1);
                let stdout_raw = String::from_utf8_lossy(&stdout_bytes);
                let stderr_raw = String::from_utf8_lossy(&stderr_bytes);

                let sanitized_stdout = redact_secrets(&stdout_raw);
                let sanitized_stderr = redact_secrets(&stderr_raw);

                let mut hasher = blake3::Hasher::new();
                hasher.update(sanitized_stdout.as_bytes());
                hasher.update(sanitized_stderr.as_bytes());
                let output_blake3_digest = hasher.finalize().to_hex().to_string();

                let stdout_tail = truncate_tail(&sanitized_stdout, 50);
                let stderr_tail = truncate_tail(&sanitized_stderr, 50);

                Ok(SupervisedExecutionOutcome {
                    exit_code,
                    duration_ms,
                    stdout_tail,
                    stderr_tail,
                    output_blake3_digest,
                })
            }
            Ok((Err(e), _, _)) => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "async_supervisor".to_string(),
                details: format!("Process execution error: {e}"),
            }),
            Err(_) => {
                // Timeout occurred
                let _ = child.kill().await;
                Ok(SupervisedExecutionOutcome {
                    exit_code: 124,
                    duration_ms,
                    stdout_tail: "".to_string(),
                    stderr_tail: format!("Execution timed out after {}ms", timeout.as_millis()),
                    output_blake3_digest: "".to_string(),
                })
            }
        }
    }
}

fn truncate_tail(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        text.to_string()
    } else {
        lines[(lines.len() - max_lines)..].join("\n")
    }
}

impl AsyncExecutionSupervisorPort for AsyncExecutionSupervisor {
    fn execute_supervised(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.execute_supervised_async(request)))
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "async_supervisor".to_string(),
                    details: format!("Failed to initialize tokio runtime: {e}"),
                })?;
            rt.block_on(self.execute_supervised_async(request))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_redaction_nominal_and_tokens() {
        let input = "Error with ghp_abcdef123456789012345678 and token=secretvalue12345";
        let redacted = redact_secrets(input);
        assert!(!redacted.contains("ghp_abcdef123456789012345678"));
        assert!(!redacted.contains("secretvalue12345"));
        assert!(redacted.contains("[REDACTED_SECRET]"));
    }

    #[test]
    fn test_execute_supervised_success_and_digest() {
        let supervisor = AsyncExecutionSupervisor::new();
        let req = SupervisedExecutionRequest {
            command: "echo".to_string(),
            args: vec!["hello world ghp_1234567890abcdef1234".to_string()],
            working_dir: "".to_string(),
            timeout_ms: 5000,
        };

        let outcome = supervisor.execute_supervised(&req).expect("execution succeeded");
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout_tail.contains("hello world [REDACTED_SECRET]"));
        assert_eq!(outcome.output_blake3_digest.len(), 64);
    }

    #[test]
    fn test_execute_supervised_timeout() {
        let supervisor = AsyncExecutionSupervisor::new();
        let req = SupervisedExecutionRequest {
            command: "sleep".to_string(),
            args: vec!["1".to_string()],
            working_dir: "".to_string(),
            timeout_ms: 100,
        };

        let outcome = supervisor.execute_supervised(&req).expect("timeout caught");
        assert_eq!(outcome.exit_code, 124);
        assert!(outcome.stderr_tail.contains("timed out"));
    }
}
