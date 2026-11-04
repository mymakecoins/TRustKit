// Pure Hard Invariants policy evaluators.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

/// Evaluates command string against Hard Invariants (RN-001..RN-016).
pub fn evaluate_command_invariants(command: &str) -> Result<(), DomainViolation> {
    if contains_prohibited_emoji(command) {
        return Err(violation(
            "RN-007",
            command,
            "Prohibited Unicode emoji or dingbat detected.",
            "Remove all emojis, use ASCII text.",
        ));
    }
    if contains_ai_trailer(command) {
        return Err(violation(
            "RN-013",
            command,
            "AI attribution trailer detected.",
            "Remove AI co-authorship trailers.",
        ));
    }
    for stmt in split_command_statements(command)
        .into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if is_autonomous_gh_pr_create(stmt) {
            return Err(violation(
                "RN-012",
                stmt,
                "Autonomous 'gh pr create' is prohibited.",
                "Use 'trustkit pr'.",
            ));
        }
        if let Some(v) = evaluate_git_statement(stmt) {
            return Err(v);
        }
    }
    Ok(())
}

fn violation(inv: &str, cmd: &str, reason: &str, remed: &str) -> DomainViolation {
    DomainViolation::AstIntegrityOrInvariantViolation {
        invariant_id: inv.to_string(),
        intercepted_command: cmd.to_string(),
        reason: reason.to_string(),
        remediation: remed.to_string(),
    }
}

pub fn contains_prohibited_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c,
            '\u{1F600}'..='\u{1F64F}' | '\u{1F300}'..='\u{1F5FF}' | '\u{1F680}'..='\u{1F6FF}' |
            '\u{1F700}'..='\u{1F77F}' | '\u{1F780}'..='\u{1F7FF}' | '\u{1F800}'..='\u{1F8FF}' |
            '\u{1F900}'..='\u{1F9FF}' | '\u{1FA00}'..='\u{1FA6F}' | '\u{1FA70}'..='\u{1FAFF}' |
            '\u{2600}'..='\u{26FF}'   | '\u{2700}'..='\u{27BF}'   | '\u{1F1E6}'..='\u{1F1FF}' |
            '\u{FE00}'..='\u{FE0F}'
        )
    })
}

pub fn contains_ai_trailer(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    l.contains("co-authored-by:")
        || l.contains("claude-session")
        || l.contains("generated with")
        || l.contains("claude.ai/code")
}

fn is_autonomous_gh_pr_create(stmt: &str) -> bool {
    let t: Vec<&str> = stmt.split_whitespace().collect();
    t.windows(3)
        .any(|w| w[0] == "gh" && w[1] == "pr" && w[2] == "create")
}

fn evaluate_git_statement(stmt: &str) -> Option<DomainViolation> {
    let t: Vec<&str> = stmt.split_whitespace().collect();
    if t.is_empty() || t[0] != "git" {
        return None;
    }
    let sub = t.get(1).copied().unwrap_or("");
    if sub == "stash" {
        let non_flags: Vec<&str> = t[2..]
            .iter()
            .copied()
            .filter(|a| !a.starts_with('-'))
            .collect();
        let ok = matches!(non_flags.first(), Some(&"list") | Some(&"show"));
        if !ok {
            return Some(violation(
                "RN-006",
                stmt,
                "Executing 'git stash' disrupts shared tree state.",
                "Keep pre-existing files untouched.",
            ));
        }
    }
    if sub == "add"
        && t[2..]
            .iter()
            .any(|&a| matches!(a, "." | "-A" | "--all" | "-a" | ":/"))
    {
        return Some(violation(
            "RN-006",
            stmt,
            "Sweeping stage operations are prohibited in shared trees.",
            "Stage explicit task files.",
        ));
    }
    if sub == "reset" && t[2..].contains(&"--hard") {
        return Some(violation(
            "RN-006",
            stmt,
            "Destructive 'git reset --hard' risks destroying uncommitted files.",
            "Revert explicit files via checkout.",
        ));
    }
    if sub == "clean"
        && t[2..]
            .iter()
            .any(|&a| a.starts_with("-f") || a == "--force")
    {
        return Some(violation(
            "RN-006",
            stmt,
            "Destructive 'git clean' risks destroying uncommitted files.",
            "Revert explicit files via checkout.",
        ));
    }
    if sub == "push"
        && t[2..].iter().any(|&a| {
            matches!(a, "main" | "staging" | "develop")
                || a.ends_with("/main")
                || a.ends_with("/staging")
                || a.ends_with("/develop")
        })
    {
        return Some(violation(
            "RN-006",
            stmt,
            "Direct pushes to protected branches are prohibited.",
            "Work in a dedicated feature or task branch.",
        ));
    }
    None
}

fn split_command_statements(command: &str) -> Vec<&str> {
    let mut stmts = Vec::new();
    let (mut last, mut sq, mut dq) = (0, false, false);
    let bytes = command.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !dq => sq = !sq,
            b'"' if !sq => dq = !dq,
            b';' if !sq && !dq => {
                if last < i {
                    stmts.push(&command[last..i]);
                }
                last = i + 1;
            }
            b'|' if !sq && !dq => {
                if last < i {
                    stmts.push(&command[last..i]);
                }
                if bytes.get(i + 1) == Some(&b'|') {
                    i += 1;
                }
                last = i + 1;
            }
            b'&' if !sq && !dq && bytes.get(i + 1) == Some(&b'&') => {
                if last < i {
                    stmts.push(&command[last..i]);
                }
                i += 1;
                last = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if last < command.len() {
        stmts.push(&command[last..]);
    }
    stmts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_stash_policy() {
        for cmd in &[
            "git stash",
            "git stash pop",
            "git stash drop",
            "git stash clear",
            "git stash push -m 'wip'",
        ] {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
        assert!(evaluate_command_invariants("git stash list").is_ok());
        assert!(evaluate_command_invariants("git stash show").is_ok());
    }

    #[test]
    fn test_sweeping_git_add_policy() {
        for cmd in &["git add .", "git add -A", "git add --all", "git add -a"] {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
        assert!(evaluate_command_invariants("git add src/lib.rs").is_ok());
        assert!(evaluate_command_invariants("git add Cargo.toml Cargo.lock").is_ok());
    }

    #[test]
    fn test_destructive_git_reset_and_clean_policy() {
        for cmd in &[
            "git reset --hard",
            "git reset --hard HEAD~1",
            "git clean -fd",
            "git clean -f",
            "git clean -fx",
        ] {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
        assert!(evaluate_command_invariants("git reset --soft HEAD~1").is_ok());
        assert!(evaluate_command_invariants("git checkout -- src/file.rs").is_ok());
    }

    #[test]
    fn test_protected_branch_push_policy() {
        for cmd in &[
            "git push origin main",
            "git push origin staging",
            "git push origin develop",
            "git push -u origin main",
        ] {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
        assert!(evaluate_command_invariants("git push origin feat/45-core-db").is_ok());
    }

    #[test]
    fn test_autonomous_gh_pr_create_policy() {
        assert!(evaluate_command_invariants("gh pr create --title 'WIP'").is_err());
        assert!(evaluate_command_invariants("gh pr create").is_err());
        assert!(evaluate_command_invariants("gh pr view 45").is_ok());
        assert!(evaluate_command_invariants("gh issue list").is_ok());
    }

    #[test]
    fn test_ai_trailers_policy() {
        let cases = [
            "git commit -m 'feat: update\n\nCo-Authored-By: Claude <noreply@anthropic.com>'",
            "git commit -m 'feat: done\n\nClaude-Session: 12345'",
            "git commit -m 'feat: done\n\nGenerated with Claude Code'",
            "git commit -m 'feat: ref https://claude.ai/code/session'",
        ];
        for cmd in &cases {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
    }

    #[test]
    fn test_unicode_emojis_policy() {
        for cmd in &[
            "git commit -m 'feat: \u{1F680} rocket'",
            "git commit -m 'fix: \u{1F41B} bug'",
            "echo '\u{1F600}'",
        ] {
            assert!(
                evaluate_command_invariants(cmd).is_err(),
                "Should block: {cmd}"
            );
        }
        assert!(evaluate_command_invariants("git commit -m 'feat: support fast hooks'").is_ok());
    }

    #[test]
    fn test_safe_developer_commands() {
        for cmd in &[
            "cargo test",
            "cargo build --release",
            "git status",
            "git diff",
            "ls -la",
        ] {
            assert!(
                evaluate_command_invariants(cmd).is_ok(),
                "Should pass: {cmd}"
            );
        }
    }

    #[test]
    fn test_split_command_statements_with_pipes_and_or() {
        let stmts = split_command_statements("cmd1 || cmd2 | cmd3 && cmd4; cmd5");
        assert_eq!(stmts.len(), 5);
        assert_eq!(stmts[0].trim(), "cmd1");
        assert_eq!(stmts[1].trim(), "cmd2");
        assert_eq!(stmts[2].trim(), "cmd3");
        assert_eq!(stmts[3].trim(), "cmd4");
        assert_eq!(stmts[4].trim(), "cmd5");
    }
}
