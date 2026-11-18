// Pure Domain Entities and Port for TRustKit Self-Update (`trustkit update` / `tk update`).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfUpdateStatus {
    UpToDate,
    UpdateAvailable,
    Updated,
    ForcedReinstalled,
}

impl SelfUpdateStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UpToDate => "UP_TO_DATE",
            Self::UpdateAvailable => "UPDATE_AVAILABLE",
            Self::Updated => "UPDATED",
            Self::ForcedReinstalled => "FORCED_REINSTALLED",
        }
    }

    pub const fn is_update_available(&self) -> bool {
        matches!(self, Self::UpdateAvailable)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateInstallStrategy {
    SourceRepository,
    CargoGitInstall,
}

impl UpdateInstallStrategy {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SourceRepository => "SOURCE_REPOSITORY",
            Self::CargoGitInstall => "CARGO_GIT_INSTALL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfUpdateRequest {
    pub check_only: bool,
    pub force: bool,
    pub target_branch: String,
}

impl SelfUpdateRequest {
    pub fn new(check_only: bool, force: bool, target_branch: impl Into<String>) -> Self {
        let branch = target_branch.into();
        let normalized = if branch.trim().is_empty() {
            "main".to_string()
        } else {
            branch
        };
        Self {
            check_only,
            force,
            target_branch: normalized,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfUpdateReport {
    pub status: SelfUpdateStatus,
    pub strategy: UpdateInstallStrategy,
    pub local_commit: String,
    pub remote_commit: String,
    pub target_branch: String,
    pub binaries_updated: Vec<String>,
    pub summary: String,
}

pub trait SelfUpdatePort: Send + Sync {
    fn execute_update(&self, req: &SelfUpdateRequest) -> Result<SelfUpdateReport, DomainViolation>;
}
