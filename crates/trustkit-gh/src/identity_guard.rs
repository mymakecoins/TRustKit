// GitHub active identity verification guard (RN-014).
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::process::Command;
use trustkit_core::errors::DomainViolation;
use trustkit_core::special_cases::IdentityVerification;

pub struct GitHubIdentityGuard;

impl GitHubIdentityGuard {
    pub fn verify(expected_handle: &str) -> Result<IdentityVerification, DomainViolation> {
        let detected = match Self::query_current_login() {
            Ok(handle) => handle,
            Err(details) => return Ok(IdentityVerification::GhUnavailable { details }),
        };

        if expected_handle.trim().is_empty() || detected.eq_ignore_ascii_case(expected_handle) {
            return Ok(IdentityVerification::Verified {
                operator_handle: detected,
            });
        }

        // Attempt automatic switch per RN-014
        if Self::attempt_switch(expected_handle).is_ok()
            && let Ok(switched) = Self::query_current_login()
            && switched.eq_ignore_ascii_case(expected_handle)
        {
            return Ok(IdentityVerification::Verified {
                operator_handle: switched,
            });
        }

        Ok(IdentityVerification::Rejected {
            detected_handle: detected,
            expected_handle: expected_handle.to_string(),
        })
    }

    fn query_current_login() -> Result<String, String> {
        let output = Command::new("gh")
            .args(["api", "user", "-q", ".login"])
            .output()
            .map_err(|e| format!("Failed to spawn 'gh': {e}"))?;

        if !output.status.success() {
            // Fallback to parsing gh auth status
            return Self::query_from_auth_status();
        }

        let login = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if login.is_empty() {
            Self::query_from_auth_status()
        } else {
            Ok(login)
        }
    }

    fn query_from_auth_status() -> Result<String, String> {
        let output = Command::new("gh")
            .args(["auth", "status"])
            .output()
            .map_err(|e| format!("Failed to execute 'gh auth status': {e}"))?;

        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        Self::parse_active_handle(&combined)
            .ok_or_else(|| "No active logged in account found in 'gh auth status'".to_string())
    }

    pub fn parse_active_handle(text: &str) -> Option<String> {
        let mut current_account: Option<String> = None;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Logged in to github.com account") {
                if let Some(pos) = trimmed.find("account ") {
                    let after = &trimmed[(pos + 8)..];
                    let handle = after.split_whitespace().next()?.to_string();
                    current_account = Some(handle);
                }
            } else if trimmed.contains("Active account: true")
                && let Some(acc) = current_account.take()
            {
                return Some(acc);
            }
        }
        None
    }

    fn attempt_switch(handle: &str) -> Result<(), String> {
        let output = Command::new("gh")
            .args(["auth", "switch", "-u", handle])
            .output()
            .map_err(|e| format!("Failed to run gh auth switch: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_active_handle_from_multiline_status() {
        let status = r#"
github.com
  Logged in to github.com account other_user (keyring)
  - Active account: false
  Logged in to github.com account operator (keyring)
  - Active account: true
  - Token scopes: 'gist', 'project'
"#;
        let parsed = GitHubIdentityGuard::parse_active_handle(status);
        assert_eq!(parsed, Some("operator".to_string()));
    }

    #[test]
    fn test_parse_active_handle_none_when_empty() {
        let parsed = GitHubIdentityGuard::parse_active_handle("no login here");
        assert_eq!(parsed, None);
    }
}
