// Merge Danger Door & Blast Radius Calculator (TASK-003, RF-003, RN-003).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{BlastRadiusLevel, MergeDangerAssessment, MergeDangerClassification};
use std::collections::BTreeSet;

pub struct MergeDangerCalculator;

impl MergeDangerCalculator {
    pub fn evaluate<S: AsRef<str>>(affected_files: &[S]) -> MergeDangerAssessment {
        let mut is_one_way = false;
        let mut reasons = Vec::new();
        let mut files = Vec::new();
        let mut crates_affected = BTreeSet::new();

        for file in affected_files {
            let path_str = file.as_ref();
            files.push(path_str.to_string());

            if let Some(rest) = path_str.strip_prefix("crates/") {
                if let Some(crate_name) = rest.split('/').next() {
                    crates_affected.insert(crate_name.to_string());
                }
            }

            if path_str.ends_with("migrations.rs") || path_str.contains("/migrations/") {
                is_one_way = true;
                reasons.push(format!("Database migration altered: {path_str}"));
            } else if path_str.ends_with("schema.rs")
                || path_str.contains("/schemas/")
                || path_str.contains("state.db")
            {
                is_one_way = true;
                reasons.push(format!("Persisted schema altered: {path_str}"));
            } else if path_str.ends_with("ports.rs") || path_str.contains("/ports/") {
                is_one_way = true;
                reasons.push(format!("Public port contract altered: {path_str}"));
            }
        }

        let classification = if is_one_way {
            MergeDangerClassification::OneWayDoor
        } else {
            MergeDangerClassification::TwoWayDoor
        };

        let blast_radius = if is_one_way {
            BlastRadiusLevel::SystemWide
        } else if crates_affected.len() >= 2 {
            BlastRadiusLevel::CrossCrate
        } else if crates_affected.len() == 1 {
            let all_tests = files
                .iter()
                .all(|f| f.contains("/tests/") || f.contains("_test") || f.ends_with(".md"));
            if all_tests {
                BlastRadiusLevel::Isolated
            } else {
                BlastRadiusLevel::ModuleLocal
            }
        } else {
            BlastRadiusLevel::Isolated
        };

        let reason = if reasons.is_empty() {
            "Changes restricted to internal adapters or tests".to_string()
        } else {
            reasons.join("; ")
        };

        MergeDangerAssessment {
            classification,
            blast_radius,
            reason,
            affected_files: files,
        }
    }

    pub fn inject_into_pr_body(body: &str, assessment: &MergeDangerAssessment) -> String {
        if body.contains("## Merge Danger") {
            body.to_string()
        } else {
            let section = assessment.to_markdown_section();
            format!("{body}\n\n{section}")
        }
    }
}
