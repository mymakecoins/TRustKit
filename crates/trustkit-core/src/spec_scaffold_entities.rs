// Spec Scaffold domain entities and Design It Twice Briefings (Ousterhout).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DesignItTwiceStrategy {
    MinimizeInterface,
    MaximizeFlexibility,
    OptimizeCommonCaller,
}

impl DesignItTwiceStrategy {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MinimizeInterface => "Minimizacao de Interface",
            Self::MaximizeFlexibility => "Maxima Flexibilidade",
            Self::OptimizeCommonCaller => "Otimizacao para o Chamador Comum",
        }
    }

    pub const fn code_label(&self) -> &'static str {
        match self {
            Self::MinimizeInterface => "MINIMIZE_INTERFACE",
            Self::MaximizeFlexibility => "MAXIMIZE_FLEXIBILITY",
            Self::OptimizeCommonCaller => "OPTIMIZE_COMMON_CALLER",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignItTwiceBrief {
    pub brief_id: String,
    pub strategy: DesignItTwiceStrategy,
    pub strategy_name: String,
    pub design_constraint: String,
    pub target_seam_focus: String,
    pub subagent_prompt: String,
    pub expected_tradeoffs: String,
}

impl DesignItTwiceBrief {
    pub fn new(
        brief_id: impl Into<String>,
        strategy: DesignItTwiceStrategy,
        design_constraint: impl Into<String>,
        target_seam_focus: impl Into<String>,
        subagent_prompt: impl Into<String>,
        expected_tradeoffs: impl Into<String>,
    ) -> Self {
        Self {
            brief_id: brief_id.into(),
            strategy,
            strategy_name: strategy.as_str().to_string(),
            design_constraint: design_constraint.into(),
            target_seam_focus: target_seam_focus.into(),
            subagent_prompt: subagent_prompt.into(),
            expected_tradeoffs: expected_tradeoffs.into(),
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"brief_id\":\"{}\",\"strategy_code\":\"{}\",\"strategy_name\":\"{}\",\"design_constraint\":\"{}\",\"target_seam_focus\":\"{}\",\"subagent_prompt\":\"{}\",\"expected_tradeoffs\":\"{}\"}}",
            escape_json(&self.brief_id),
            self.strategy.code_label(),
            escape_json(&self.strategy_name),
            escape_json(&self.design_constraint),
            escape_json(&self.target_seam_focus),
            escape_json(&self.subagent_prompt),
            escape_json(&self.expected_tradeoffs)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecPromptRequest {
    pub idea: String,
    pub context_hops: usize,
    pub token_budget: usize,
    pub design_it_twice: bool,
}

impl SpecPromptRequest {
    pub fn new(
        idea: impl Into<String>,
        context_hops: usize,
        token_budget: usize,
    ) -> Result<Self, DomainViolation> {
        let idea = idea.into();
        let trimmed = idea.trim();
        if trimmed.is_empty() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "spec prompt".to_string(),
                reason: "SpecPromptRequest idea cannot be empty".to_string(),
                remediation: "Provide a descriptive feature idea in natural language".to_string(),
            });
        }
        if token_budget == 0 {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-001".to_string(),
                intercepted_command: "spec prompt".to_string(),
                reason: "SpecPromptRequest token_budget must be greater than zero".to_string(),
                remediation: "Specify a positive token budget (e.g. 4000)".to_string(),
            });
        }
        Ok(Self {
            idea: trimmed.to_string(),
            context_hops,
            token_budget,
            design_it_twice: false,
        })
    }

    pub fn with_design_it_twice(mut self, enabled: bool) -> Self {
        self.design_it_twice = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecPromptPayload {
    pub system_instructions: String,
    pub scaffold_markdown: String,
    pub context_summary: String,
    pub token_estimate: usize,
    pub design_it_twice_briefs: Vec<DesignItTwiceBrief>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecScaffoldTemplate;

impl SpecScaffoldTemplate {
    pub fn generate_markdown_skeleton(feature_slug: &str, topic_idea: &str) -> String {
        let clean_slug = if feature_slug.trim().is_empty() {
            "unnamed-feature"
        } else {
            feature_slug.trim()
        };
        let clean_idea = if topic_idea.trim().is_empty() {
            "Descrever a ideia de funcionalidade aqui."
        } else {
            topic_idea.trim()
        };

        format!(
            "# Especificacao: {}\n\n\
            ## 1. Product Trio\n\n\
            - **PM**: Defina as regras de negocio e valor para o usuario para: {}.\n\
            - **Tech Lead**: Defina as fronteiras arquiteturais, crates impactados e contratos livres de valores nulos.\n\
            - **Design / QA**: Defina os criterios BDD/Gherkin e metas de latencia e integridade visual.\n\n\
            ## 2. Diagrama de Fluxo (Mermaid)\n\n\
            ```mermaid\n\
            flowchart LR\n\
                Node1[Entrada / Gatilho] --> Node2[Processamento Hexagonal]\n\
                Node2 --> Node3[Verificacao Deterministica]\n\
            ```\n\n\
            ## 3. Criterios de Aceitacao (Gherkin)\n\n\
            ### Cenario 1: Execucao Nominal (Atendimento de RF-001)\n\n\
            - **Dado** o ambiente e configuracao inicial prontos\n\
            - **Quando** o gatilho da funcionalidade for acionado\n\
            - **Entao** a operacao conclui com sucesso sem violar invariantes constitucionais.\n\n\
            ## 4. Matriz de Rastreabilidade\n\n\
            | Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |\n\
            | :--- | :--- | :--- | :--- | :--- | :--- |\n\
            | RN-001 | RF-001 | TASK-001 | Node1 | crates/trustkit-core/tests/example_tests.rs | EV-001-example.json |\n",
            clean_slug, clean_idea
        )
    }

    pub fn generate_design_it_twice_briefs(topic_idea: &str) -> Vec<DesignItTwiceBrief> {
        let clean_idea = if topic_idea.trim().is_empty() {
            "funcionalidade solicitada"
        } else {
            topic_idea.trim()
        };

        vec![
            DesignItTwiceBrief::new(
                "DIT-01",
                DesignItTwiceStrategy::MinimizeInterface,
                "Reduzir drasticamente a superficie publica: maximo de 1 a 2 metodos por trait/modulo, ocultando toda a maquina de estados e invariantes internamente.",
                "Portas de dominio em trustkit-core/src/ports.rs com tipos algebricos auto-contidos",
                format!(
                    "Projete a alternativa 1 (Minimizacao de Interface) para '{}': proponha a assinatura publica minima possivel que esconda toda a complexidade interna em um Deep Module.",
                    clean_idea
                ),
                "Menor acoplamento cognitivo para chamadores e alta localidade de mudanca; menor parametrizacao fina para casos extremos.",
            ),
            DesignItTwiceBrief::new(
                "DIT-02",
                DesignItTwiceStrategy::MaximizeFlexibility,
                "Priorizar extensibilidade e composicao de adaptadores: permitir multiplas implementacoes intercambiaveis na costura (RealSeam).",
                "Fronteira entre trustkit-core e adaptadores trustkit-ast, trustkit-db e trustkit-orch",
                format!(
                    "Projete a alternativa 2 (Maxima Flexibilidade) para '{}': estruture portas componiveis que suportem multiplos backends e fakes de teste sem alterar o nucleo.",
                    clean_idea
                ),
                "Facilita evolucao futura e injecao de multiplos adaptadores; exige cuidado contra abstracoes especulativas (HypotheticalSeam).",
            ),
            DesignItTwiceBrief::new(
                "DIT-03",
                DesignItTwiceStrategy::OptimizeCommonCaller,
                "Tornar o caso de uso de 90% dos chamadores trivial (zero configuracao manual), absorvendo defaults deterministicos dentro do modulo.",
                "Casos de uso em trustkit-core/src/use_cases/ e fachada CLI/MCP",
                format!(
                    "Projete a alternativa 3 (Otimizacao para o Chamador Comum) para '{}': otimize a ergonomia da chamada principal no CLI e MCP para executar com zero boilerplate.",
                    clean_idea
                ),
                "Ergonomia imediata para agentes e operadores no caminho feliz; transfere orquestracao padrao para dentro do caso de uso.",
            ),
        ]
    }

    pub fn build_system_instructions() -> String {
        "Instrucoes Constitucionais TRustKit SDD:\n\
        1. Proibicao absoluta de emojis ou icones Unicode em qualquer secao ou commit.\n\
        2. Proibicao absoluta de trailers de autoria IA (ex: Co-Authored-By, Generated by).\n\
        3. A especificacao DEVE conter Product Trio, Mermaid flowchart LR, Criterios Gherkin e Matriz de Rastreabilidade fechada.\n\
        4. Toda regra de negocio deve ser identificada por RN-XXX e requisito por RF-XXX.\n\
        5. Todo codigo gerado a partir desta especificacao deve seguir TDD estrito com RED comprovado antes de GREEN.".to_string()
    }

    pub fn estimate_tokens(content: &str) -> usize {
        content.len().div_ceil(4)
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

#[cfg(test)]
#[path = "spec_scaffold_entities_tests.rs"]
mod spec_scaffold_entities_tests;
