// Deep Modules (John Ousterhout) and Seam Classification (Michael Feathers) Domain Entities.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleDepthClassification {
    Deep,
    Balanced,
    ShallowPassThrough,
}

impl ModuleDepthClassification {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deep => "Deep",
            Self::Balanced => "Balanced",
            Self::ShallowPassThrough => "ShallowPassThrough",
        }
    }

    pub const fn is_shallow(&self) -> bool {
        matches!(self, Self::ShallowPassThrough)
    }

    pub const fn is_deep(&self) -> bool {
        matches!(self, Self::Deep)
    }

    pub fn from_str_label(label: &str) -> Self {
        match label.trim() {
            "Deep" | "DEEP" => Self::Deep,
            "ShallowPassThrough" | "SHALLOW_PASS_THROUGH" => Self::ShallowPassThrough,
            _ => Self::Balanced,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDepthMetric {
    pub file_path: String,
    pub public_surface_units: usize,
    pub internal_implementation_units: usize,
    pub depth_ratio_bps: u32,
    pub classification: ModuleDepthClassification,
    pub passes_deletion_test: bool,
}

impl ModuleDepthMetric {
    pub fn evaluate(
        file_path: impl Into<String>,
        public_surface_units: usize,
        internal_implementation_units: usize,
        single_delegate_pass_through: bool,
    ) -> Self {
        let ratio_bps = internal_implementation_units
            .saturating_mul(1_000)
            .checked_div(public_surface_units)
            .unwrap_or(if internal_implementation_units > 0 {
                10_000
            } else {
                1_000
            }) as u32;

        let (classification, passes_deletion_test) = if single_delegate_pass_through
            || (public_surface_units > 0 && internal_implementation_units <= public_surface_units)
        {
            (ModuleDepthClassification::ShallowPassThrough, false)
        } else if public_surface_units > 0
            && internal_implementation_units >= public_surface_units.saturating_mul(3)
        {
            (ModuleDepthClassification::Deep, true)
        } else {
            (ModuleDepthClassification::Balanced, true)
        };

        Self {
            file_path: file_path.into(),
            public_surface_units,
            internal_implementation_units,
            depth_ratio_bps: ratio_bps,
            classification,
            passes_deletion_test,
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"file_path\":\"{}\",\"public_surface_units\":{},\"internal_implementation_units\":{},\"depth_ratio_bps\":{},\"classification\":\"{}\",\"passes_deletion_test\":{}}}",
            escape_json(&self.file_path),
            self.public_surface_units,
            self.internal_implementation_units,
            self.depth_ratio_bps,
            self.classification.as_str(),
            self.passes_deletion_test
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShallowModuleCandidate {
    pub file_path: String,
    pub affected_seam: String,
    pub public_surface_units: usize,
    pub internal_implementation_units: usize,
    pub deletion_test_justification: String,
    pub deepening_recommendation: String,
}

impl ShallowModuleCandidate {
    pub fn new(
        file_path: impl Into<String>,
        affected_seam: impl Into<String>,
        public_surface_units: usize,
        internal_implementation_units: usize,
        deletion_test_justification: impl Into<String>,
        deepening_recommendation: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            affected_seam: affected_seam.into(),
            public_surface_units,
            internal_implementation_units,
            deletion_test_justification: deletion_test_justification.into(),
            deepening_recommendation: deepening_recommendation.into(),
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"file_path\":\"{}\",\"affected_seam\":\"{}\",\"public_surface_units\":{},\"internal_implementation_units\":{},\"deletion_test_justification\":\"{}\",\"deepening_recommendation\":\"{}\"}}",
            escape_json(&self.file_path),
            escape_json(&self.affected_seam),
            self.public_surface_units,
            self.internal_implementation_units,
            escape_json(&self.deletion_test_justification),
            escape_json(&self.deepening_recommendation)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeamKind {
    RealSeam,
    HypotheticalSeam,
}

impl SeamKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RealSeam => "RealSeam",
            Self::HypotheticalSeam => "HypotheticalSeam",
        }
    }

    pub const fn is_real(&self) -> bool {
        matches!(self, Self::RealSeam)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeamClassification {
    pub seam_id: String,
    pub source_file: String,
    pub kind: SeamKind,
    pub connected_adapters: Vec<String>,
    pub adapter_count: usize,
    pub rationale: String,
}

impl SeamClassification {
    pub fn evaluate(
        seam_id: impl Into<String>,
        source_file: impl Into<String>,
        mut connected_adapters: Vec<String>,
    ) -> Self {
        connected_adapters.sort();
        connected_adapters.dedup();
        let adapter_count = connected_adapters.len();
        let (kind, rationale) = if adapter_count >= 2 {
            (
                SeamKind::RealSeam,
                format!(
                    "Validated RealSeam with {adapter_count} concrete adapters or test fakes connected"
                ),
            )
        } else {
            (
                SeamKind::HypotheticalSeam,
                format!(
                    "Speculative HypotheticalSeam with {adapter_count} adapter(s); consider collapsing into concrete module until second variation exists"
                ),
            )
        };

        Self {
            seam_id: seam_id.into(),
            source_file: source_file.into(),
            kind,
            connected_adapters,
            adapter_count,
            rationale,
        }
    }

    pub const fn classification_label(&self) -> &'static str {
        self.kind.as_str()
    }

    pub fn to_json(&self) -> String {
        let adapters_json: Vec<String> = self
            .connected_adapters
            .iter()
            .map(|a| format!("\"{}\"", escape_json(a)))
            .collect();
        format!(
            "{{\"seam_id\":\"{}\",\"source_file\":\"{}\",\"classification\":\"{}\",\"adapter_count\":{},\"connected_adapters\":[{}],\"rationale\":\"{}\"}}",
            escape_json(&self.seam_id),
            escape_json(&self.source_file),
            self.kind.as_str(),
            self.adapter_count,
            adapters_json.join(","),
            escape_json(&self.rationale)
        )
    }
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
