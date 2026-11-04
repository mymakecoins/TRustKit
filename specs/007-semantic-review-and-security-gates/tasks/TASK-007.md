# TASK-007: Persistir Evidencias de Revisao no SQLite WAL e Renderizar no `gauntlet-report.md`

> **Camada:** Persistencia SQLite & Orquestracao de Relatorio
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-001]
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/evidence_ops.rs
- crates/trustkit-orch/src/certification_reporter.rs
test_file: crates/trustkit-db/tests/sqlite_security_review_evidence_tests.rs

---

## Contexto
Para manter a auditoria imutavel via Event Sourcing no banco `.trustkit/state.db` e garantir que o relatorio oficial `gauntlet-report.md` reflita se a revisao de seguranca opcional foi aprovada (`PASS`), reprovada (`FAIL`) ou dispensada pelo desenvolvedor (`SKIPPED (Opt-out pelo desenvolvedor)`), e necessario atualizar o mapeamento SQLite e o gerador de relatorio de certificacao.

## Escopo
Implementar:
- Em `crates/trustkit-db/src/evidence_ops.rs` e `row_mappers.rs`:
  - Suporte completo a serializacao e desserializacao de `EvidenceKind::CodeReview` (`"CODE_REVIEW"`) e `EvidenceKind::SecurityReview` (`"SECURITY_REVIEW"`) com restricao `NOT NULL` e digest `BLAKE3`.
- Em `crates/trustkit-orch/src/certification_reporter.rs`:
  - Incluir no cabecalho e na secao de resumo do `gauntlet-report.md` a linha:
    - `- **Specialized Security Review:** PASS (Confianca >= 8)` quando executada e aprovada;
    - `- **Specialized Security Review:** SKIPPED (Opt-out pelo desenvolvedor)` quando dispensada;
    - `- **Specialized Security Review:** FAIL (<N> vulnerabilidades detectadas)` quando reprovada;
  - Exibir na tabela de auditoria os eventos de `AUDITOR:CODE_REVIEW` e `CRITIC:SECURITY_REVIEW`.

### Fora do escopo
- Alteracoes nos comandos CLI de terminal (tratados na TASK-005).

## Dependencias
- TASK-001

## Premissas
- Tabelas SQLite permanecem 100% `NOT NULL` (Principio Constitucional II).
- Relatorio gerado 100% livre de emojis e de trailers de autoria IA.

## Criterio de Aceite
- [x] **Dado** um ciclo contendo uma evidencia `EvidenceKind::SecurityReview` com status `SkippedByDeveloper`, **quando** o `CertificationReporter` renderiza o `gauntlet-report.md`, **entao** o documento inclui exatamente `- **Specialized Security Review:** SKIPPED (Opt-out pelo desenvolvedor)`.
- [x] **Dado** uma evidencia `EvidenceKind::CodeReview` gravada no SQLite WAL, **quando** consultada via `list_evidences_for_cycle`, **entao** reconstitui `EvidenceKind::CodeReview` com integridade total.

## Notas Tecnicas
- Preservar compatibilidade com todos os testes existentes em `crates/trustkit-db/tests/sqlite_tdd_and_cert_tests.rs`.

## Rastreabilidade
- Origem: RF-039, RN-007
