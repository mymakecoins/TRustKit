// Command Runner Port and Process Adapter for TRustKit Self-Update (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use trustkit_core::errors::DomainViolation;

pub const OFFICIAL_REPO_URL: &str = "https://github.com/mymakecoins/TRustKit.git";

pub(crate) fn update_fault(details: impl Into<String>) -> DomainViolation {
    DomainViolation::InfrastructureBoundaryFault {
        subsystem: "self_update".to_string(),
        details: details.into(),
    }
}

pub trait UpdateCommandRunnerPort: Send + Sync {
    fn is_source_repository(&self, dir: &Path) -> bool;
    fn is_working_tree_clean(&self, dir: &Path) -> Result<bool, DomainViolation>;
    fn resolve_local_commit(&self, dir: &Path) -> Result<String, DomainViolation>;
    fn resolve_remote_main_commit(
        &self,
        dir: &Path,
        branch: &str,
    ) -> Result<String, DomainViolation>;
    fn pull_and_build_release(
        &self,
        dir: &Path,
        branch: &str,
    ) -> Result<Vec<String>, DomainViolation>;
    fn cargo_install_from_git(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<Vec<String>, DomainViolation>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessUpdateCommandRunner;

impl ProcessUpdateCommandRunner {
    fn run_cmd(dir: Option<&Path>, prog: &str, args: &[&str]) -> Result<String, DomainViolation> {
        let mut cmd = Command::new(prog);
        cmd.args(args);
        if let Some(d) = dir {
            cmd.current_dir(d);
        }
        let out = cmd
            .output()
            .map_err(|e| update_fault(format!("Spawn '{prog}' failed: {e}")))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(update_fault(format!(
                "'{prog} {}' failed: {}",
                args.join(" "),
                err.trim()
            )));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn install_release_binaries(dir: &Path) -> Result<Vec<String>, DomainViolation> {
        let release_dir = dir.join("target").join("release");
        let bin_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .filter(|p| p.file_name().is_some_and(|n| n == "bin"))
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/bin")));
        let mut updated = Vec::new();
        for name in ["trustkit", "tk"] {
            let src = release_dir.join(name);
            if src.exists() {
                if let Some(ref target_dir) = bin_dir {
                    let _ = fs::create_dir_all(target_dir);
                    let dest = target_dir.join(name);
                    let tmp = target_dir.join(format!(".{name}.tmp"));
                    fs::copy(&src, &tmp)
                        .and_then(|_| fs::rename(&tmp, &dest))
                        .map_err(|e| {
                            let _ = fs::remove_file(&tmp);
                            update_fault(format!("Atomic install failed for {name}: {e}"))
                        })?;
                }
                updated.push(name.to_string());
            }
        }
        Ok(if updated.is_empty() {
            vec!["trustkit".into(), "tk".into()]
        } else {
            updated
        })
    }
}

impl UpdateCommandRunnerPort for ProcessUpdateCommandRunner {
    fn is_source_repository(&self, dir: &Path) -> bool {
        dir.join("Cargo.toml").exists()
            && dir.join("crates/trustkit-cli").exists()
            && Self::run_cmd(Some(dir), "git", &["rev-parse", "--is-inside-work-tree"]).is_ok()
    }

    fn is_working_tree_clean(&self, dir: &Path) -> Result<bool, DomainViolation> {
        let out = Self::run_cmd(Some(dir), "git", &["status", "--porcelain"])?;
        Ok(out.lines().all(|l| l.trim().is_empty()))
    }

    fn resolve_local_commit(&self, dir: &Path) -> Result<String, DomainViolation> {
        if self.is_source_repository(dir)
            && let Ok(sha) = Self::run_cmd(Some(dir), "git", &["rev-parse", "--short", "HEAD"])
            && !sha.is_empty()
        {
            return Ok(sha);
        }
        Ok(format!("v{}", env!("CARGO_PKG_VERSION")))
    }

    fn resolve_remote_main_commit(
        &self,
        dir: &Path,
        branch: &str,
    ) -> Result<String, DomainViolation> {
        if self.is_source_repository(dir) {
            let _ = Self::run_cmd(Some(dir), "git", &["fetch", "origin", branch]);
            let rref = format!("origin/{branch}");
            if let Ok(sha) = Self::run_cmd(Some(dir), "git", &["rev-parse", "--short", &rref])
                && !sha.is_empty()
            {
                return Ok(sha);
            }
        }
        let spec = format!("refs/heads/{branch}");
        if let Ok(out) = Self::run_cmd(None, "git", &["ls-remote", OFFICIAL_REPO_URL, &spec])
            && let Some(sha) = out.split_whitespace().next().filter(|s| s.len() >= 7)
        {
            return Ok(sha[..7].to_string());
        }
        self.resolve_local_commit(dir)
    }

    fn pull_and_build_release(
        &self,
        dir: &Path,
        branch: &str,
    ) -> Result<Vec<String>, DomainViolation> {
        if Self::run_cmd(Some(dir), "git", &["rev-parse", "--abbrev-ref", "HEAD"])? != branch {
            Self::run_cmd(Some(dir), "git", &["checkout", branch])?;
        }
        Self::run_cmd(
            Some(dir),
            "git",
            &["merge", "--ff-only", &format!("origin/{branch}")],
        )?;
        Self::run_cmd(Some(dir), "cargo", &["build", "--workspace", "--release"])?;
        Self::install_release_binaries(dir)
    }

    fn cargo_install_from_git(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<Vec<String>, DomainViolation> {
        let args = [
            "install",
            "--git",
            repo_url,
            "--branch",
            branch,
            "--force",
            "trustkit-cli",
        ];
        Self::run_cmd(None, "cargo", &args)?;
        Ok(vec!["trustkit".into(), "tk".into()])
    }
}
