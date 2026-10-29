// WorktreeIsolationProvider implementing WorktreeIsolationPort.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{WorktreeIsolationPort, WorktreeStatus};
use trustkit_core::special_cases::WorktreeAllocation;

#[derive(Debug, Clone)]
pub struct WorktreeIsolationProvider {
    repo_root: PathBuf,
}

impl WorktreeIsolationProvider {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub fn detect() -> Result<Self, DomainViolation> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: format!("Failed to spawn 'git rev-parse --show-toplevel': {e}"),
            })?;

        if !output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: "Not inside a valid Git repository".to_string(),
            });
        }

        let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Self::new(PathBuf::from(root_str)))
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    fn run_git_cmd(&self, working_dir: &Path, args: &[&str]) -> Result<String, DomainViolation> {
        let output = Command::new("git")
            .current_dir(working_dir)
            .args(args)
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: format!("Failed to run 'git {}': {e}", args.join(" ")),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: format!(
                    "Command 'git {}' failed with code {:?}: {} {}",
                    args.join(" "),
                    output.status.code(),
                    stdout,
                    stderr
                ),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn link_dependencies(&self, target_dir: &Path) {
        #[cfg(unix)]
        {
            let candidates = ["node_modules", "target"];
            for dir_name in &candidates {
                let source = self.repo_root.join(dir_name);
                let dest = target_dir.join(dir_name);
                if source.exists() && !dest.exists() {
                    let _ = std::os::unix::fs::symlink(&source, &dest);
                }
            }
        }
    }
}

impl WorktreeIsolationPort for WorktreeIsolationProvider {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        base_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        let worktree_dir = self.repo_root.join(".trustkit").join("worktrees");
        fs::create_dir_all(&worktree_dir).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "worktree_provider".to_string(),
            details: format!("Failed to create worktrees dir {:?}: {e}", worktree_dir),
        })?;

        let task_worktree_path = worktree_dir.join(task_id);
        let branch_name = format!("worktree/{}", task_id);

        if task_worktree_path.exists() {
            let _ = self.run_git_cmd(
                &self.repo_root,
                &["worktree", "remove", "--force", &task_worktree_path.to_string_lossy()],
            );
            let _ = fs::remove_dir_all(&task_worktree_path);
        }

        let path_str = task_worktree_path.to_string_lossy().to_string();
        self.run_git_cmd(
            &self.repo_root,
            &["worktree", "add", "-B", &branch_name, &path_str, base_branch],
        )?;

        self.link_dependencies(&task_worktree_path);

        Ok(WorktreeAllocation::Provisioned {
            worktree_path: path_str,
            branch_name,
        })
    }

    fn check_worktree_status(&self, worktree_path: &str) -> Result<WorktreeStatus, DomainViolation> {
        let wt_path = Path::new(worktree_path);
        if !wt_path.exists() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: format!("Worktree path does not exist: {}", worktree_path),
            });
        }

        let status_out = self.run_git_cmd(wt_path, &["status", "--porcelain"])?;
        let is_clean = status_out.trim().is_empty();

        let branch_name = self.run_git_cmd(wt_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let head_commit_sha = self.run_git_cmd(wt_path, &["rev-parse", "HEAD"])?;

        Ok(WorktreeStatus {
            worktree_path: worktree_path.to_string(),
            branch_name,
            is_clean,
            head_commit_sha,
        })
    }

    fn merge_and_prune_worktree(
        &self,
        _task_id: &str,
        worktree_path: &str,
        target_branch: &str,
    ) -> Result<String, DomainViolation> {
        let wt_path = Path::new(worktree_path);
        let worktree_head = if wt_path.exists() {
            self.run_git_cmd(wt_path, &["rev-parse", "HEAD"])?
        } else {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_provider".to_string(),
                details: format!("Worktree path does not exist for merge: {}", worktree_path),
            });
        };

        // Determine current checked out branch on repo_root
        let current_branch = self.run_git_cmd(&self.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if current_branch == target_branch {
            let _ = self.run_git_cmd(&self.repo_root, &["merge", "--no-edit", &worktree_head]);
        } else {
            // Update the target branch reference directly if not currently checked out
            let _ = self.run_git_cmd(
                &self.repo_root,
                &["branch", "-f", target_branch, &worktree_head],
            );
        }

        let merged_sha = self.run_git_cmd(&self.repo_root, &["rev-parse", target_branch])?;

        // Remove worktree
        let _ = self.run_git_cmd(&self.repo_root, &["worktree", "remove", "--force", worktree_path]);
        if wt_path.exists() {
            let _ = fs::remove_dir_all(wt_path);
        }
        let _ = self.run_git_cmd(&self.repo_root, &["worktree", "prune"]);

        Ok(merged_sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_git_repo() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        let path = dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(path).output().expect("git user");
        Command::new("git").args(["config", "user.email", "test@test.local"]).current_dir(path).output().expect("git email");

        fs::write(path.join("init.txt"), "init content").expect("write init");
        Command::new("git").args(["add", "init.txt"]).current_dir(path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "init: #1 initial"]).current_dir(path).output().expect("git commit");

        dir
    }

    #[test]
    fn test_provision_check_and_prune_worktree() {
        let repo_dir = setup_git_repo();
        let provider = WorktreeIsolationProvider::new(repo_dir.path());

        let alloc = provider
            .provision_worktree("cycle-1", "TASK-001", "main")
            .expect("provision worktree");

        let wt_path = match alloc {
            WorktreeAllocation::Provisioned { worktree_path, branch_name } => {
                assert!(worktree_path.contains("TASK-001"));
                assert_eq!(branch_name, "worktree/TASK-001");
                worktree_path
            }
            _ => panic!("Expected Provisioned allocation"),
        };

        let status = provider.check_worktree_status(&wt_path).expect("check status");
        assert!(status.is_clean);
        assert_eq!(status.branch_name, "worktree/TASK-001");

        // Make a commit in the worktree
        fs::write(Path::new(&wt_path).join("task.txt"), "task content").expect("write task.txt");
        Command::new("git").args(["add", "task.txt"]).current_dir(&wt_path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "feat: #1 task"]).current_dir(&wt_path).output().expect("git commit");

        let merged_sha = provider
            .merge_and_prune_worktree("TASK-001", &wt_path, "main")
            .expect("merge and prune");
        assert!(!merged_sha.is_empty());
        assert!(!Path::new(&wt_path).exists());
    }
}
