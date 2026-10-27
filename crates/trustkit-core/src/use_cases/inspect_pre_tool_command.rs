// Synchronous use case for evaluating shell commands prior to execution.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::invariants_policy::evaluate_command_invariants;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectPreToolCommandRequest {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectPreToolCommandOutcome {
    pub evaluated_command: String,
}

pub fn execute(
    request: InspectPreToolCommandRequest,
) -> Result<InspectPreToolCommandOutcome, DomainViolation> {
    evaluate_command_invariants(&request.command)?;
    Ok(InspectPreToolCommandOutcome {
        evaluated_command: request.command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_approves_safe_command() {
        let req = InspectPreToolCommandRequest {
            command: "cargo test -p trustkit-core".to_string(),
        };
        let res = execute(req);
        assert!(res.is_ok());
        let outcome = res.unwrap();
        assert_eq!(outcome.evaluated_command, "cargo test -p trustkit-core");
    }

    #[test]
    fn test_execute_blocks_forbidden_command() {
        let req = InspectPreToolCommandRequest {
            command: "git stash".to_string(),
        };
        let res = execute(req);
        assert!(res.is_err());
        match res.unwrap_err() {
            DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id,
                intercepted_command,
                ..
            } => {
                assert_eq!(invariant_id, "RN-006");
                assert_eq!(intercepted_command, "git stash");
            }
            other => panic!("Expected AstIntegrityOrInvariantViolation, got {other:?}"),
        }
    }
}
