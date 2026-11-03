// Terminal Multiplexer Domain Entities.
// Constitutional Principle I: Zero external dependencies (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplexedTerminalRequest {
    pub task_id: String,
    pub working_dir: String,
    pub command_argv: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub timeout_ms: u64,
}

impl MultiplexedTerminalRequest {
    pub fn new(
        task_id: impl Into<String>,
        working_dir: impl Into<String>,
        command_argv: Vec<String>,
        env_vars: Vec<(String, String)>,
        timeout_ms: u64,
    ) -> Result<Self, DomainViolation> {
        let task_id = task_id.into();
        let working_dir = working_dir.into();

        if task_id.trim().is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "MultiplexedTerminalRequest::new".to_string(),
                reason: "task_id cannot be empty".to_string(),
                remediation: "Provide a valid task identifier like TASK-001".to_string(),
            });
        }

        if working_dir.trim().is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "MultiplexedTerminalRequest::new".to_string(),
                reason: "working_dir cannot be empty".to_string(),
                remediation: "Provide the isolated Git worktree path".to_string(),
            });
        }

        if command_argv.is_empty() || command_argv[0].trim().is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "MultiplexedTerminalRequest::new".to_string(),
                reason: "command_argv cannot be empty".to_string(),
                remediation: "Provide at least the agent binary to execute".to_string(),
            });
        }

        Ok(Self {
            task_id,
            working_dir,
            command_argv,
            env_vars,
            timeout_ms: if timeout_ms == 0 { 60_000 } else { timeout_ms },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplexedTerminalHandle {
    pub terminal_id: String,
    pub task_id: String,
    pub pid: u32,
    pub created_at_iso: String,
}

impl MultiplexedTerminalHandle {
    pub fn new(
        terminal_id: impl Into<String>,
        task_id: impl Into<String>,
        pid: u32,
        created_at_iso: impl Into<String>,
    ) -> Self {
        Self {
            terminal_id: terminal_id.into(),
            task_id: task_id.into(),
            pid,
            created_at_iso: created_at_iso.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReasonKind {
    Permission,
    Question,
    Authentication,
    Unknown,
}

impl BlockReasonKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Question => "question",
            Self::Authentication => "auth",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "permission" => Self::Permission,
            "question" => Self::Question,
            "auth" | "authentication" => Self::Authentication,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSemanticState {
    Working,
    Blocked {
        kind: BlockReasonKind,
        detail: String,
    },
    Done {
        exit_code: i32,
    },
    Failed {
        reason: String,
    },
}

impl AgentSemanticState {
    pub fn is_working(&self) -> bool {
        matches!(self, Self::Working)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked { .. })
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done { .. } | Self::Failed { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeBackendMode {
    #[default]
    Auto,
    Herdr,
    Native,
}

impl RuntimeBackendMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Herdr => "herdr",
            Self::Native => "native",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "herdr" => Self::Herdr,
            "native" => Self::Native,
            _ => Self::Auto,
        }
    }
}
