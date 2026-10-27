// Local Git worktree inspector.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::path::Path;
use std::process::Command;
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitWorktreeContext {
    pub work_branch: String,
    pub base_commit_sha: String,
    pub pre_existing_dirty_files: Vec<String>,
}

pub struct GitInspector;

impl GitInspector {
    pub fn inspect(repo_dir: &Path) -> Result<GitWorktreeContext, DomainViolation> {
        Self::verify_git_repository(repo_dir)?;
        let work_branch = Self::resolve_active_branch(repo_dir)?;
        let base_commit_sha = Self::resolve_head_sha(repo_dir)?;
        let pre_existing_dirty_files = Self::discover_dirty_files(repo_dir)?;

        Ok(GitWorktreeContext {
            work_branch,
            base_commit_sha,
            pre_existing_dirty_files,
        })
    }

    fn verify_git_repository(dir: &Path) -> Result<(), DomainViolation> {
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(dir)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Failed to invoke git binary: {e}"),
            })?;

        if !output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: "Directory is not a valid Git repository".to_string(),
            });
        }
        Ok(())
    }

    fn resolve_active_branch(dir: &Path) -> Result<String, DomainViolation> {
        let sym_output = Command::new("git")
            .args(["symbolic-ref", "--short", "-q", "HEAD"])
            .current_dir(dir)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Git symbolic-ref execution failed: {e}"),
            })?;

        if sym_output.status.success() {
            let branch = String::from_utf8_lossy(&sym_output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Ok(branch);
            }
        }

        // Check if HEAD is detached
        let rev_output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Git rev-parse HEAD failed: {e}"),
            })?;

        let ref_name = String::from_utf8_lossy(&rev_output.stdout).trim().to_string();
        if ref_name == "HEAD" || !rev_output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: "Repository is in detached HEAD state; branch checkout required".to_string(),
            });
        }

        Ok(ref_name)
    }

    fn resolve_head_sha(dir: &Path) -> Result<String, DomainViolation> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Git rev-parse HEAD failed: {e}"),
            })?;

        if !output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: "Failed to determine HEAD commit SHA".to_string(),
            });
        }

        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sha.len() != 40 {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Invalid commit SHA resolved: '{sha}'"),
            });
        }
        Ok(sha)
    }

    fn discover_dirty_files(dir: &Path) -> Result<Vec<String>, DomainViolation> {
        let output = Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(dir)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: format!("Git status failed: {e}"),
            })?;

        if !output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "git".to_string(),
                details: "Failed to inspect git status".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dirty = Vec::new();
        for line in stdout.lines() {
            if line.len() > 3 {
                let path = line[3..].trim().to_string();
                if !path.is_empty() {
                    dirty.push(path);
                }
            }
        }
        Ok(dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn init_repo(p: &Path) {
        let run = |args: &[&str]| {
            let status = Command::new("git").args(args).current_dir(p).status().unwrap();
            assert!(status.success());
        };
        run(&["init", "-b", "feat/45-core"]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Tester"]);
        fs::write(p.join("a.txt"), "hello").unwrap();
        run(&["add", "a.txt"]);
        run(&["commit", "-m", "init"]);
    }

    #[test]
    fn test_inspect_clean_repo() {
        let dir = tempdir().unwrap();
        init_repo(dir.path());
        let ctx = GitInspector::inspect(dir.path()).unwrap();
        assert_eq!(ctx.work_branch, "feat/45-core");
        assert_eq!(ctx.base_commit_sha.len(), 40);
        assert!(ctx.pre_existing_dirty_files.is_empty());
    }

    #[test]
    fn test_inspect_dirty_repo_preserves_files() {
        let dir = tempdir().unwrap();
        init_repo(dir.path());
        fs::write(dir.path().join("dirty.txt"), "content").unwrap();
        let ctx = GitInspector::inspect(dir.path()).unwrap();
        assert_eq!(ctx.pre_existing_dirty_files, vec!["dirty.txt"]);
        assert_eq!(fs::read_to_string(dir.path().join("dirty.txt")).unwrap(), "content");
    }

    #[test]
    fn test_inspect_non_git_repo_fails() {
        let dir = tempdir().unwrap();
        let err = GitInspector::inspect(dir.path()).unwrap_err();
        assert_eq!(err.posix_exit_code(), 1);
    }
}
