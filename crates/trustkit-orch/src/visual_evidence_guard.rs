// VisualEvidenceGuard implementing VisualEvidencePort.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{ForeignScreenshotInspection, VisualEvidencePort};

#[derive(Debug, Clone)]
pub struct VisualEvidenceGuard {
    repo_root: PathBuf,
}

impl VisualEvidenceGuard {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn validate_screenshot_html_tag(html_tag: &str) -> Result<(), DomainViolation> {
        let trimmed = html_tag.trim();
        if trimmed.contains("raw.githubusercontent.com") {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-008".to_string(),
                intercepted_command: "validate_screenshot_html_tag".to_string(),
                reason: "Invariant #8 violation: raw.githubusercontent.com links are forbidden for visual evidences.".to_string(),
                remediation: "Use standard repository link with '?raw=true' or '&raw=true' parameter.".to_string(),
            });
        }

        if !trimmed.starts_with("<img") || !trimmed.contains("src=\"https://github.com/") || !trimmed.contains("raw=true") {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-008".to_string(),
                intercepted_command: "validate_screenshot_html_tag".to_string(),
                reason: "Visual evidence tag must follow '<img src=\"https://github.com/.../evidence/....png?raw=true\" ... />' format.".to_string(),
                remediation: "Format as: <img src=\"https://github.com/<org>/<repo>/blob/<branch>/.../evidence/<name>.png?raw=true\" width=\"100%\" />".to_string(),
            });
        }

        Ok(())
    }

    fn is_visual_evidence_file(path_str: &str) -> bool {
        let lower = path_str.to_lowercase();
        let is_image = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".webp")
            || lower.ends_with(".svg");

        is_image && (lower.contains("evidence") || lower.contains("screenshot") || lower.contains("docs/specs"))
    }
}

impl VisualEvidencePort for VisualEvidenceGuard {
    fn restore_foreign_screenshots(
        &self,
        active_issue_prefix: &str,
        _evidence_dir: &str,
    ) -> Result<ForeignScreenshotInspection, DomainViolation> {
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "visual_evidence_guard".to_string(),
                details: format!("Failed to run 'git status --porcelain': {e}"),
            })?;

        if !output.status.success() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "visual_evidence_guard".to_string(),
                details: "Failed to check git status".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut detected = Vec::new();
        let mut restored = Vec::new();

        for line in stdout.lines() {
            if line.len() < 4 {
                continue;
            }
            let status_code = &line[..2];
            let raw_path = line[3..].trim().trim_matches('"');

            if !Self::is_visual_evidence_file(raw_path) {
                continue;
            }

            // If it belongs to another issue and not the active issue
            if !active_issue_prefix.is_empty() && !raw_path.contains(active_issue_prefix) {
                detected.push(raw_path.to_string());

                if status_code.contains('?') {
                    // Untracked foreign file - remove it
                    let full_path = self.repo_root.join(raw_path);
                    if full_path.exists() {
                        let _ = fs::remove_file(&full_path);
                    }
                    restored.push(raw_path.to_string());
                } else {
                    // Modified, staged, or deleted tracked file - restore from git
                    let _ = Command::new("git")
                        .current_dir(&self.repo_root)
                        .args(["restore", "--staged", raw_path])
                        .output();

                    let res = Command::new("git")
                        .current_dir(&self.repo_root)
                        .args(["checkout", "--", raw_path])
                        .output();

                    if let Ok(checkout_out) = res
                        && checkout_out.status.success()
                    {
                        restored.push(raw_path.to_string());
                    }
                }
            }
        }

        Ok(ForeignScreenshotInspection {
            active_issue_prefix: active_issue_prefix.to_string(),
            foreign_screenshots_detected: detected,
            restored_screenshots: restored,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_git_repo_with_foreign_screenshot() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempdir().expect("tempdir");
        let path = dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(path).output().expect("git user");
        Command::new("git").args(["config", "user.email", "test@test.local"]).current_dir(path).output().expect("git email");

        let foreign_dir = path.join("docs/specs/issue-12-202610281200/tests/evidence");
        fs::create_dir_all(&foreign_dir).expect("create foreign dir");
        let foreign_file = foreign_dir.join("01-login.png");
        fs::write(&foreign_file, b"original-screenshot-issue-12").expect("write foreign");

        let active_dir = path.join("docs/specs/issue-45-202610281300/tests/evidence");
        fs::create_dir_all(&active_dir).expect("create active dir");
        let active_file = active_dir.join("01-worktree.png");
        fs::write(&active_file, b"original-screenshot-issue-45").expect("write active");

        Command::new("git").args(["add", "."]).current_dir(path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "init: #0 evidences"]).current_dir(path).output().expect("git commit");

        (dir, foreign_file, active_file)
    }

    #[test]
    fn test_restore_foreign_screenshots_reverts_other_issue_only() {
        let (repo_dir, foreign_file, active_file) = setup_git_repo_with_foreign_screenshot();
        let guard = VisualEvidenceGuard::new(repo_dir.path());

        // Modify both screenshots
        fs::write(&foreign_file, b"corrupted-by-issue-45-tests").expect("corrupt foreign");
        fs::write(&active_file, b"valid-new-evidence-45").expect("update active");

        let inspection = guard
            .restore_foreign_screenshots("issue-45", "docs/specs/issue-45-202610281300/tests/evidence")
            .expect("inspection success");

        assert_eq!(inspection.foreign_screenshots_detected.len(), 1);
        assert!(inspection.foreign_screenshots_detected[0].contains("issue-12"));
        assert_eq!(inspection.restored_screenshots.len(), 1);

        // Verify foreign file was restored to original content
        let restored_bytes = fs::read(&foreign_file).expect("read foreign");
        assert_eq!(restored_bytes, b"original-screenshot-issue-12");

        // Verify active issue file kept its updated content
        let active_bytes = fs::read(&active_file).expect("read active");
        assert_eq!(active_bytes, b"valid-new-evidence-45");
    }

    #[test]
    fn test_validate_screenshot_html_tag_rules() {
        let valid = r#"<img src="https://github.com/org/repo/blob/develop/docs/evidence/shot.png?raw=true" alt="shot" width="100%" />"#;
        assert!(VisualEvidenceGuard::validate_screenshot_html_tag(valid).is_ok());

        let forbidden_raw = r#"<img src="https://raw.githubusercontent.com/org/repo/develop/evidence/shot.png" />"#;
        assert!(VisualEvidenceGuard::validate_screenshot_html_tag(forbidden_raw).is_err());

        let missing_raw_query = r#"<img src="https://github.com/org/repo/blob/develop/evidence/shot.png" />"#;
        assert!(VisualEvidenceGuard::validate_screenshot_html_tag(missing_raw_query).is_err());
    }
}
