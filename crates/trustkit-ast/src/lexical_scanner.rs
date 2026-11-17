// Invariant lexical scanner for AST Layer 0 (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use trustkit_core::entities::AstViolation;

#[derive(Debug, Default, Clone, Copy)]
pub struct InvariantLexicalScanner;

impl InvariantLexicalScanner {
    pub const fn new() -> Self {
        Self
    }

    pub fn scan_content(&self, file_path: &str, content: &str) -> Vec<AstViolation> {
        let mut violations = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;

            // 1. Emoji scan (RN-007)
            for ch in line.chars() {
                if Self::is_emoji(ch) {
                    violations.push(AstViolation::ForbiddenEmoji {
                        file_path: file_path.to_string(),
                        line: line_num,
                        character: ch.to_string(),
                    });
                }
            }

            // 2. AI trailer scan (RN-013)
            let lower = line.to_lowercase();
            if lower.contains("co-authored-by: claude")
                || lower.contains("co-authored-by: anthropic")
                || lower.contains("claude-session")
                || lower.contains("generated with")
                || lower.contains("claude.ai/code")
            {
                violations.push(AstViolation::AiTrailer {
                    file_path: file_path.to_string(),
                    line: line_num,
                    trailer: line.trim().to_string(),
                });
            }

            // 3. Untyped any check for ts/tsx files
            if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
                let occurrences = line.matches(": any").count() + line.matches("as any").count();
                for _ in 0..occurrences {
                    violations.push(AstViolation::UntypedAny {
                        file_path: file_path.to_string(),
                        line: line_num,
                    });
                }
            }

            // 4. Residual debug probe scan (RN-002, RF-002)
            const DEBUG_PROBE_PREFIX: &str = concat!("[", "DEBUG", "-");
            if line.contains(DEBUG_PROBE_PREFIX) {
                violations.push(AstViolation::ResidualDebugProbe {
                    file_path: file_path.to_string(),
                    line: line_num,
                    probe: line.trim().to_string(),
                });
            }
        }

        violations
    }

    fn is_emoji(ch: char) -> bool {
        matches!(
            ch as u32,
            0x1F600..=0x1F64F
                | 0x1F300..=0x1F5FF
                | 0x1F680..=0x1F6FF
                | 0x1F700..=0x1F77F
                | 0x1F780..=0x1F7FF
                | 0x1F800..=0x1F8FF
                | 0x1F900..=0x1F9FF
                | 0x1FA00..=0x1FA6F
                | 0x1FA70..=0x1FAFF
                | 0x2600..=0x26FF
                | 0x2700..=0x27BF
        )
    }
}
