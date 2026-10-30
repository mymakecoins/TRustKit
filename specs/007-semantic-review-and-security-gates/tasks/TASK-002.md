# TASK-002: Implementar Resolucao de Politica de Revisao de Seguranca (`trustkit.toml`)

> **Camada:** Dominio / Politica
> **Estimativa:** P
> **Status:** Concluido

depends_on: [TASK-001]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/security_review_policy.rs
test_file: crates/trustkit-core/tests/security_review_policy_tests.rs

---

## Contexto
O time precisa parametrizar o comportamento padrao da `security-review` no repositorio via secao `[governance.gauntlet]` em `trustkit.toml` (ou `.trustkit/config.json`) quando nenhuma flag de linha de comando (`--security-review` ou `--skip-security-review`) for informada.

## Escopo
Implementar em `crates/trustkit-core/src/security_review_policy.rs`:
- `SecurityReviewPolicyConfig` contendo `mode: SecurityReviewMode` (padrao `SecurityReviewMode::Prompt`) e `min_confidence_threshold: u8` (padrao `8`, validado no intervalo `8..=10`).
- Parser deterministico `parse_security_review_policy(raw_content: &str) -> Result<SecurityReviewPolicyConfig, DomainViolation>` capaz de extrair `security_review_mode` (`"prompt"`, `"always"`, `"disabled"`) e `min_confidence_threshold` sem depender de crates externas no core.
- Resolvedor de precedencia `resolve_security_review_decision(cli_flag: CliSecurityReviewFlag, policy: &SecurityReviewPolicyConfig, is_interactive_tty: bool, is_json_mode: bool) -> SecurityReviewDecision`.

### Fora do escopo
- Leitura direta de I/O de sistema de arquivos dentro de `trustkit-core` (o conteudo textual e injetado pelo adaptador/CLI).

## Dependencias
- TASK-001

## Premissas
- `min_confidence_threshold` inferior a `8` ou superior a `10` retorna `DomainViolation::AstIntegrityOrInvariantViolation`.
- Precedencia estrita: Flag CLI (`--security-review` / `--skip-security-review`) > Modo `--json` / Non-TTY > Politica `trustkit.toml` (`"prompt"`, `"always"`, `"disabled"`).

## Criterio de Aceite
- [x] **Dado** uma configuracao com `security_review_mode = "prompt"` e `min_confidence_threshold = 8`, **quando** avaliada sem flags CLI em sessao TTY interativa (`is_json_mode = false`), **entao** resolve para `SecurityReviewDecision::PromptUser`.
- [x] **Dado** a mesma configuracao em modo `--json` (`is_json_mode = true`), **quando** avaliada sem flags CLI, **entao** resolve deterministicamente para `SecurityReviewDecision::Skip { reason: SecuritySkipReason::HeadlessNonInteractiveDefault }`.

## Notas Tecnicas
- Manter 100% pure Rust (`std` only) em `trustkit-core`.

## Rastreabilidade
- Origem: RF-034, RN-002
