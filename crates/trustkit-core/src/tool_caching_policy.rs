// Tool caching and batch execution policies for harness economy (TASK-001 / RN-001 / RF-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompilationCacheBackend {
    #[default]
    Auto,
    Sccache,
    SharedTargetDir,
    Disabled,
}

impl CompilationCacheBackend {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Sccache => "sccache",
            Self::SharedTargetDir => "shared-target",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_str_case_insensitive(val: &str) -> Option<Self> {
        match val.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "sccache" => Some(Self::Sccache),
            "shared-target" | "shared_target" | "sharedtargetdir" => Some(Self::SharedTargetDir),
            "disabled" | "none" | "false" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCachingPolicy {
    pub compilation_cache: CompilationCacheBackend,
    pub shared_target_dir: String,
    pub ast_analysis_cache_enabled: bool,
    pub structural_coupling_cache_enabled: bool,
}

impl Default for ToolCachingPolicy {
    fn default() -> Self {
        Self {
            compilation_cache: CompilationCacheBackend::Auto,
            shared_target_dir: ".trustkit/cache/target".to_string(),
            ast_analysis_cache_enabled: true,
            structural_coupling_cache_enabled: true,
        }
    }
}

impl ToolCachingPolicy {
    pub fn new(
        compilation_cache: CompilationCacheBackend,
        shared_target_dir: impl Into<String>,
        ast_analysis_cache_enabled: bool,
        structural_coupling_cache_enabled: bool,
    ) -> Result<Self, DomainViolation> {
        let dir = shared_target_dir.into();
        if compilation_cache != CompilationCacheBackend::Disabled && dir.trim().is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "tool_caching_policy".to_string(),
                reason: "shared_target_dir cannot be empty when compilation cache is enabled"
                    .to_string(),
                remediation: "Configure a valid non-empty shared_target_dir in trustkit.toml."
                    .to_string(),
            });
        }
        Ok(Self {
            compilation_cache,
            shared_target_dir: dir,
            ast_analysis_cache_enabled,
            structural_coupling_cache_enabled,
        })
    }

    pub const fn is_compilation_cache_enabled(&self) -> bool {
        !matches!(self.compilation_cache, CompilationCacheBackend::Disabled)
    }

    pub const fn resolve_rustc_wrapper(&self, sccache_available: bool) -> Option<&'static str> {
        match self.compilation_cache {
            CompilationCacheBackend::Auto if sccache_available => Some("sccache"),
            CompilationCacheBackend::Sccache if sccache_available => Some("sccache"),
            _ => None,
        }
    }

    pub fn resolve_cargo_target_dir(&self) -> Option<&str> {
        if self.is_compilation_cache_enabled() && !self.shared_target_dir.trim().is_empty() {
            Some(self.shared_target_dir.as_str())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchGroupingMode {
    #[default]
    PerCrate,
    PerWave,
    Disabled,
}

impl BatchGroupingMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PerCrate => "per-crate",
            Self::PerWave => "per-wave",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_str_case_insensitive(val: &str) -> Option<Self> {
        match val.trim().to_ascii_lowercase().as_str() {
            "per-crate" | "per_crate" | "percrate" => Some(Self::PerCrate),
            "per-wave" | "per_wave" | "perwave" => Some(Self::PerWave),
            "disabled" | "none" | "false" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchExecutionConfig {
    pub enabled: bool,
    pub grouping_mode: BatchGroupingMode,
    pub max_batch_size: usize,
    pub preserve_task_traceability: bool,
}

impl Default for BatchExecutionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            grouping_mode: BatchGroupingMode::PerCrate,
            max_batch_size: 8,
            preserve_task_traceability: true,
        }
    }
}

impl BatchExecutionConfig {
    pub fn new(
        enabled: bool,
        grouping_mode: BatchGroupingMode,
        max_batch_size: usize,
        preserve_task_traceability: bool,
    ) -> Result<Self, DomainViolation> {
        if !(1..=64).contains(&max_batch_size) {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "batch_execution_config".to_string(),
                reason: format!("max_batch_size {max_batch_size} is out of allowed range 1..=64"),
                remediation: "Set max_batch_size between 1 and 64 in trustkit.toml.".to_string(),
            });
        }
        if enabled && !preserve_task_traceability {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "batch_execution_config".to_string(),
                reason: "Batch execution cannot disable per-task traceability".to_string(),
                remediation: "Keep preserve_task_traceability = true for batching.".to_string(),
            });
        }
        Ok(Self {
            enabled,
            grouping_mode,
            max_batch_size,
            preserve_task_traceability,
        })
    }

    pub const fn is_batching_active(&self) -> bool {
        self.enabled && !matches!(self.grouping_mode, BatchGroupingMode::Disabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HarnessExecutionPolicyConfig {
    pub caching: ToolCachingPolicy,
    pub batching: BatchExecutionConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolEconomyMetrics {
    pub total_cmd_executions: usize,
    pub cached_executions: usize,
    pub batched_executions: usize,
    pub cache_reuse_rate_pct: u8,
    pub total_duration_ms: u64,
    pub average_cycle_time_ms: u64,
}

pub fn parse_harness_execution_policy(
    raw_content: &str,
) -> Result<HarnessExecutionPolicyConfig, DomainViolation> {
    let mut cache_backend = CompilationCacheBackend::Auto;
    let mut shared_target_dir = ".trustkit/cache/target".to_string();
    let mut ast_cache = true;
    let mut coupling_cache = true;
    let mut batch_enabled = true;
    let mut grouping_mode = BatchGroupingMode::PerCrate;
    let mut max_batch_size: usize = 8;
    let mut preserve_traceability = true;

    for line in raw_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let clean =
            trimmed.trim_matches(|c: char| c == ',' || c == '{' || c == '}' || c.is_whitespace());
        let Some((raw_key, raw_val)) = clean.split_once('=').or_else(|| clean.split_once(':'))
        else {
            continue;
        };
        let key = raw_key
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'')
            .to_ascii_lowercase();
        let val = raw_val
            .split('#')
            .next()
            .unwrap_or(raw_val)
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c.is_whitespace());

        match key.as_str() {
            "compilation_cache" => {
                cache_backend = CompilationCacheBackend::from_str_case_insensitive(val)
                    .ok_or_else(|| policy_err(&format!("Invalid compilation_cache '{val}'")))?;
            }
            "shared_target_dir" => shared_target_dir = val.to_string(),
            "ast_cache_enabled" | "ast_analysis_cache_enabled" => {
                ast_cache = parse_bool(val, &key)?;
            }
            "coupling_cache_enabled" | "structural_coupling_cache_enabled" => {
                coupling_cache = parse_bool(val, &key)?;
            }
            "batch_execution_enabled" => batch_enabled = parse_bool(val, &key)?,
            "batch_grouping_mode" => {
                grouping_mode = BatchGroupingMode::from_str_case_insensitive(val)
                    .ok_or_else(|| policy_err(&format!("Invalid batch_grouping_mode '{val}'")))?;
            }
            "max_batch_size" => {
                max_batch_size = val
                    .parse::<usize>()
                    .map_err(|_| policy_err(&format!("Invalid max_batch_size '{val}'")))?;
            }
            "preserve_task_traceability" => preserve_traceability = parse_bool(val, &key)?,
            _ => {}
        }
    }

    Ok(HarnessExecutionPolicyConfig {
        caching: ToolCachingPolicy::new(
            cache_backend,
            shared_target_dir,
            ast_cache,
            coupling_cache,
        )?,
        batching: BatchExecutionConfig::new(
            batch_enabled,
            grouping_mode,
            max_batch_size,
            preserve_traceability,
        )?,
    })
}

fn parse_bool(val: &str, key: &str) -> Result<bool, DomainViolation> {
    match val.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(policy_err(&format!("Invalid boolean '{val}' for {key}"))),
    }
}

fn policy_err(reason: &str) -> DomainViolation {
    DomainViolation::AstIntegrityOrInvariantViolation {
        invariant_id: "RN-001".to_string(),
        intercepted_command: "harness_execution_policy".to_string(),
        reason: reason.to_string(),
        remediation: "Check [harness.execution] settings in trustkit.toml.".to_string(),
    }
}
