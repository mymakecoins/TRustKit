// Supervisor Runtime Selector with graceful fallback.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::async_supervisor::AsyncExecutionSupervisor;
use crate::pty_socket_adapter::PtySocketAdapter;
use std::path::PathBuf;
use trustkit_core::entities::{Osc7501Payload, RuntimeBackendMode, encode_osc7501};
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Clone)]
pub struct ResolvedRuntimeBackend {
    pub active_mode: RuntimeBackendMode,
    pub fallback_triggered: bool,
    pub multiplexer: Option<PtySocketAdapter>,
    pub supervisor: AsyncExecutionSupervisor,
}

#[derive(Debug, Default, Clone)]
pub struct SupervisorRuntimeSelector {
    custom_socket_path: Option<PathBuf>,
}

impl SupervisorRuntimeSelector {
    pub fn new() -> Self {
        Self {
            custom_socket_path: None,
        }
    }

    pub fn with_custom_socket(socket_path: PathBuf) -> Self {
        Self {
            custom_socket_path: Some(socket_path),
        }
    }

    fn check_pty_available(&self) -> Option<PtySocketAdapter> {
        if let Some(ref path) = self.custom_socket_path {
            if path.exists() {
                return Some(PtySocketAdapter::new(path.clone()));
            }
            None
        } else {
            PtySocketAdapter::discover_default()
        }
    }

    pub fn resolve(
        &self,
        requested: RuntimeBackendMode,
    ) -> Result<ResolvedRuntimeBackend, DomainViolation> {
        let supervisor = AsyncExecutionSupervisor::new();
        let pty_opt = self.check_pty_available();

        let resolved = match requested {
            RuntimeBackendMode::Native => ResolvedRuntimeBackend {
                active_mode: RuntimeBackendMode::Native,
                fallback_triggered: false,
                multiplexer: None,
                supervisor,
            },
            RuntimeBackendMode::Pty => {
                if let Some(adapter) = pty_opt {
                    ResolvedRuntimeBackend {
                        active_mode: RuntimeBackendMode::Pty,
                        fallback_triggered: false,
                        multiplexer: Some(adapter),
                        supervisor,
                    }
                } else {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "runtime_selector".to_string(),
                        details: "PTY multiplexer daemon socket is unavailable. Start the daemon or run with --runtime native or --runtime auto".to_string(),
                    });
                }
            }
            RuntimeBackendMode::Auto => {
                if let Some(adapter) = pty_opt {
                    ResolvedRuntimeBackend {
                        active_mode: RuntimeBackendMode::Pty,
                        fallback_triggered: false,
                        multiplexer: Some(adapter),
                        supervisor,
                    }
                } else {
                    ResolvedRuntimeBackend {
                        active_mode: RuntimeBackendMode::Native,
                        fallback_triggered: true,
                        multiplexer: None,
                        supervisor,
                    }
                }
            }
        };

        // Emit telemetry
        let payload = Osc7501Payload {
            state: "working".to_string(),
            id: "runtime_selector".to_string(),
            kind: "".to_string(),
            progress: None,
            app: "trustkit".to_string(),
            title: "Runtime resolved".to_string(),
            msg: format!(
                "Active: {}, Fallback: {}",
                resolved.active_mode.as_str(),
                resolved.fallback_triggered
            ),
        };
        let _ = encode_osc7501(&payload);

        Ok(resolved)
    }
}
