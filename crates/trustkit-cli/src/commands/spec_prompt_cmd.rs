// Handler for `trustkit spec prompt` (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::SpecPromptArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonDesignItTwiceBrief, JsonSpecPromptResponse};
use std::path::Path;
use trustkit_ast::TreeSitterAstAuditorAdapter;
use trustkit_core::entities::{SpecPromptRequest, SpecScaffoldTemplate};
use trustkit_core::errors::DomainViolation;
use trustkit_core::use_cases::compose_spec_context::{self, ComposeSpecContextRequest};
use trustkit_db::KnowledgeWikiFsRepository;

pub fn execute(args: &SpecPromptArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    execute_from_root(args, json_mode, Path::new("."))
}

pub fn execute_from_root(
    args: &SpecPromptArgs,
    json_mode: bool,
    root: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let domain_req = SpecPromptRequest::new(&args.idea, args.context_hops, args.budget)?
        .with_design_it_twice(args.design_it_twice);

    let context_summary = if let Ok(repo) = KnowledgeWikiFsRepository::open_from_root(root) {
        let slicer = TreeSitterAstAuditorAdapter::new();
        let compose_req =
            ComposeSpecContextRequest::new(domain_req.idea.clone(), domain_req.context_hops, 5)?;
        match compose_spec_context::execute(compose_req, &repo, &slicer) {
            Ok(outcome) => outcome.aggregated_context,
            Err(_) => "Contexto nao pode ser extraido da memoria.".to_string(),
        }
    } else {
        "Nenhum repositorio de memoria ativo em .trustkit/wiki.".to_string()
    };

    let slug_hint = domain_req
        .idea
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");
    let clean_slug = slug_hint
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    let scaffold_markdown =
        SpecScaffoldTemplate::generate_markdown_skeleton(&clean_slug, &domain_req.idea);
    let system_instructions = SpecScaffoldTemplate::build_system_instructions();

    let full_content = format!(
        "{}\n{}\n{}",
        system_instructions, context_summary, scaffold_markdown
    );
    let token_estimate = SpecScaffoldTemplate::estimate_tokens(&full_content);

    let design_it_twice_briefs = if domain_req.design_it_twice {
        SpecScaffoldTemplate::generate_design_it_twice_briefs(&domain_req.idea)
            .into_iter()
            .map(|b| JsonDesignItTwiceBrief {
                brief_id: b.brief_id,
                strategy_code: b.strategy.code_label().to_string(),
                strategy_name: b.strategy_name,
                design_constraint: b.design_constraint,
                target_seam_focus: b.target_seam_focus,
                subagent_prompt: b.subagent_prompt,
                expected_tradeoffs: b.expected_tradeoffs,
            })
            .collect()
    } else {
        Vec::new()
    };

    let response = JsonSpecPromptResponse {
        verdict: "PASS".to_string(),
        idea: domain_req.idea,
        token_estimate,
        system_instructions,
        scaffold_markdown,
        context_summary,
        design_it_twice: domain_req.design_it_twice,
        design_it_twice_briefs,
        next_steps: Vec::new(),
    };

    let rendered = GatePresenter::render_spec_prompt(&response, json_mode || args.json);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}
