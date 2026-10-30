# TASK-006: Implementar Caso de Uso de `security-review` e Filtro de Confianca (`>= 8`)

> **Camada:** Caso de Uso (Dominio)
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-002]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/run_security_review.rs
test_file: crates/trustkit-core/tests/run_security_review_use_case_tests.rs

---

## Contexto
Quando a execucao da `security-review` e confirmada (por flag `--security-review`, resposta `s` no prompt interativo ou politica `"always"`), o motor deve avaliar o diff consolidado da branch contra a branch base, descartar falsos positivos com confianca inferior ao limiar (`min_confidence_threshold`, padrao `8/10`) e emitir o veredito deterministico.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/run_security_review.rs`:
- `RunSecurityReviewRequest` (`cycle_id`, `target_branch`, `min_confidence_threshold: u8`, `decision: SecurityReviewDecision`, `now_iso`).
- `RunSecurityReviewOutcome` (`cycle_id`, `verdict: SecurityReviewVerdict`, `surviving_findings: Vec<SemanticReviewFinding>`, `discarded_low_confidence_count: usize`, `recorded_evidence_id: u64`).
- Se `decision` for `SecurityReviewDecision::Skip { reason }`, gravar imediatamente a evidencia `EvidenceKind::SecurityReview` com stdout `SecurityReviewVerdict::SkippedByDeveloper` e `GateVerdict::Pass`, retornando sem invocar a porta de analise.
- Se `decision` for `SecurityReviewDecision::Execute`:
  - Invocar `SecurityDiffReviewerPort::review_branch_diff(&req.target_branch)`.
  - Filtrar mantendo exclusivamente achados onde `finding.confidence >= req.min_confidence_threshold` e que pertençam as 5 categorias AppSec permitidas.
  - Gravar a evidencia em `CycleEvidenceRepositoryPort` (`GateVerdict::Pass` quando `surviving_findings.is_empty()`, ou `GateVerdict::Fail` caso contrario).

### Fora do escopo
- Renderizacao Markdown final do `gauntlet-report.md` (tratada na TASK-007).

## Dependencias
- TASK-002

## Premissas
- Qualquer achado com confianca `< 8` (ou `< min_confidence_threshold`) deve ser silenciosamente descartado e contabilizado apenas em `discarded_low_confidence_count`.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um conjunto de candidatos contendo 2 achados com confianca `6` e `7` e 1 achado de `SQL Injection` com confianca `9`, **quando** `run_security_review::execute` roda com limiar `8`, **entao** descarta os 2 primeiros e retorna `VulnerabilitiesDetected` contendo apenas o achado de confianca `9`.
- [x] **Dado** uma decisao `SecurityReviewDecision::Skip`, **quando** o caso de uso executa, **entao** nao aciona a porta `SecurityDiffReviewerPort` e registra `SkippedByDeveloper` no repositorio de evidencias.

## Notas Tecnicas
- Seguir o padrao estrutural de `crates/trustkit-core/src/use_cases/audit_ast_layer0.rs`.

## Rastreabilidade
- Origem: RF-038, RN-006
