# TASK-001: Implementar Entidades de Dominio de Revisao Semantica e Contratos de Porta

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Concluido

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/semantic_review_entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/src/semantic_review_entities_tests.rs

---

## Contexto
Para integrar as capacidades de revisao semantica multi-angulo (`code-review`) e inspecao opcional de vulnerabilidades AppSec em diffs (`security-review`) ao nucleo do TRustKit mantendo o Principio Constitucional I (Isolamento Hexagonal com `std` only) e II (Proibicao absoluta de `NULL`), e necessario modelar os tipos algebricos, decisoes de execucao, vereditos e portas abstratas em `trustkit-core`.

## Escopo
Modelar as estruturas de dominio e portas:
- `CodeReviewEffortLevel`: `Low`, `Medium`, `High`, `Max` com conversao case-insensitive e limite padrao de achados (`4`, `8`, `10`, `15`).
- `ReviewAngle`: `LogicCorrectness`, `RemovedBehavior`, `CrossFileContract`, `Reuse`, `Simplification`, `Efficiency`, `Altitude`, `ProjectConventions`.
- `AppSecCategory`: `InputValidationInjection`, `AuthAndAuthorization`, `CryptoAndSecrets`, `RemoteCodeExecutionXss`, `DataAndPiiExposure`.
- `SecurityReviewMode`: `Prompt`, `Always`, `Disabled`.
- `SecuritySkipReason`: `CliFlagSkip`, `DeveloperPromptOptOut`, `HeadlessNonInteractiveDefault`, `PolicyDisabled`.
- `SecurityReviewDecision`: `Execute`, `Skip { reason: SecuritySkipReason }`, `PromptUser`.
- `SemanticReviewFinding`: `file`, `line`, `severity`, `category_or_angle`, `confidence` (`1..=10`), `summary`, `scenario`, `recommendation`.
- `SecurityReviewVerdict`: `CleanPass`, `SkippedByDeveloper { reason: SecuritySkipReason }`, `VulnerabilitiesDetected { findings: Vec<SemanticReviewFinding> }`.
- Variantes adicionais em `EvidenceKind`: `CodeReview` e `SecurityReview`.
- Traits `SemanticCodeReviewerPort`, `SecurityDiffReviewerPort` e `InteractivePromptPort` em `crates/trustkit-core/src/ports.rs`.

### Fora do escopo
- Parsing de arquivos TOML/JSON de configuracao, prompts de terminal TTY ou persistencia SQLite.

## Dependencias
- Nenhuma

## Premissas
- Zero dependencias externas em `trustkit-core` (`std` apenas).
- Proibicao estrita de valores `null` em todas as structs e enums.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** uma instancia de `SecurityReviewVerdict::SkippedByDeveloper`, **quando** consultada sobre o motivo da dispensa, **entao** retorna `SecuritySkipReason` tipado sem uso de `Option` nulo.
- [x] **Dado** um `SemanticReviewFinding` com confianca `9`, **quando** validado pelo metodo de corte com limiar `8`, **entao** e classificado como vulnerabilidade acionavel de alta confianca.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-core/src/gauntlet_entities.rs` e expor os novos tipos em `crates/trustkit-core/src/entities.rs`.

## Rastreabilidade
- Origem: RF-033, RN-001
