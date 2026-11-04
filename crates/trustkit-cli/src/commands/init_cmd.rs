// Handler for `trustkit init`.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::InitArgs;
use crate::error_handler::DomainOutcome;
use crate::presenter::Presenter;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::errors::DomainViolation;
use trustkit_core::use_cases::initialize_cycle::{self, InitializeCycleRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_gh::board_synchronizer::ProjectBoardSynchronizer;
use trustkit_gh::git_inspector::GitInspector;

const DEFAULT_GITIGNORE_ENTRIES: &str = r#"# TRustKit runtime transacional e worktrees efemeras
.trustkit/state.db*
.trustkit/worktrees/
.trustkit/audits/
!.trustkit/wiki/
"#;

const DEFAULT_TRUSTKIT_TOML: &str = r#"[governance.gauntlet]
# Modos: "prompt", "always", "disabled"
security_review_mode = "prompt"
min_confidence_threshold = 8
"#;

pub fn scaffold_gitignore(repo_dir: &Path) -> Result<(), DomainViolation> {
    let gitignore_path = repo_dir.join(".gitignore");
    if gitignore_path.exists() {
        let existing = fs::read_to_string(&gitignore_path).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "filesystem".to_string(),
                details: format!("Failed reading .gitignore: {e}"),
            }
        })?;
        if !existing.contains(".trustkit/state.db") {
            let mut updated = existing;
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(DEFAULT_GITIGNORE_ENTRIES);
            fs::write(&gitignore_path, updated).map_err(|e| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "filesystem".to_string(),
                    details: format!("Failed updating .gitignore: {e}"),
                }
            })?;
        }
    } else {
        fs::write(&gitignore_path, DEFAULT_GITIGNORE_ENTRIES).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "filesystem".to_string(),
                details: format!("Failed creating .gitignore: {e}"),
            }
        })?;
    }
    Ok(())
}

pub fn scaffold_trustkit_toml(repo_dir: &Path) -> Result<(), DomainViolation> {
    let toml_path = repo_dir.join("trustkit.toml");
    if !toml_path.exists() {
        fs::write(&toml_path, DEFAULT_TRUSTKIT_TOML).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "filesystem".to_string(),
                details: format!("Failed creating trustkit.toml: {e}"),
            }
        })?;
    }
    Ok(())
}

pub fn execute(args: &InitArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let repo_dir = Path::new(".");
    let git_ctx = GitInspector::inspect(repo_dir)?;

    let (now_iso, timestamp_slug) = current_timestamps();

    // Auto-scaffold .gitignore and trustkit.toml if absent or missing entries
    scaffold_gitignore(repo_dir)?;
    scaffold_trustkit_toml(repo_dir)?;

    let db_path = repo_dir.join(".trustkit").join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path)?;
    let board = ProjectBoardSynchronizer::new();

    let req = InitializeCycleRequest {
        issue_param: args.issue.clone(),
        from_prd: args.from_prd.clone(),
        work_branch: git_ctx.work_branch,
        base_commit_sha: git_ctx.base_commit_sha,
        pre_existing_dirty_files: git_ctx.pre_existing_dirty_files,
        now_iso,
        timestamp_slug,
    };

    let outcome = initialize_cycle::execute(req, &repo, &board)?;

    // Create cycle directory docs/specs/<cycle_id>
    let spec_dir = Path::new(&outcome.cycle.spec_dir);
    fs::create_dir_all(spec_dir).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "filesystem".to_string(),
        details: format!(
            "Failed creating spec directory '{}': {e}",
            outcome.cycle.spec_dir
        ),
    })?;

    let rendered = Presenter::render_init(&outcome, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

fn current_timestamps() -> (String, String) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sec = now % 60;
    let min = (now / 60) % 60;
    let hour = (now / 3600) % 24;
    let days = now / 86400;

    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (
        format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z"),
        format!("{y:04}{m:02}{d:02}{hour:02}{min:02}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_current_timestamps_format() {
        let (iso, slug) = current_timestamps();
        assert_eq!(iso.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
        assert!(iso.ends_with('Z'));
        assert_eq!(slug.len(), 12); // YYYYMMDDHHMM
    }

    #[test]
    fn test_scaffold_gitignore_creates_when_missing() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        scaffold_gitignore(repo).unwrap();
        let content = fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert!(content.contains(".trustkit/state.db*"));
        assert!(content.contains("!.trustkit/wiki/"));
    }

    #[test]
    fn test_scaffold_gitignore_appends_when_present() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        fs::write(repo.join(".gitignore"), "target/\nnode_modules/\n").unwrap();
        scaffold_gitignore(repo).unwrap();
        let content = fs::read_to_string(repo.join(".gitignore")).unwrap();
        assert!(content.starts_with("target/\nnode_modules/\n"));
        assert!(content.contains(".trustkit/state.db*"));
        assert!(content.contains("!.trustkit/wiki/"));

        // Second run does not duplicate
        scaffold_gitignore(repo).unwrap();
        let content2 = fs::read_to_string(repo.join(".gitignore")).unwrap();
        let count = content2.matches(".trustkit/state.db*").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_scaffold_trustkit_toml() {
        let dir = tempdir().unwrap();
        let repo = dir.path();
        scaffold_trustkit_toml(repo).unwrap();
        let content = fs::read_to_string(repo.join("trustkit.toml")).unwrap();
        assert!(content.contains("[governance.gauntlet]"));
        assert!(content.contains("security_review_mode = \"prompt\""));

        // Second run preserves custom settings
        fs::write(repo.join("trustkit.toml"), "custom = true\n").unwrap();
        scaffold_trustkit_toml(repo).unwrap();
        let content2 = fs::read_to_string(repo.join("trustkit.toml")).unwrap();
        assert_eq!(content2, "custom = true\n");
    }
}
