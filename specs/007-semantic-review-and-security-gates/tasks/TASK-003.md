# TASK-003: Implementar Caso de Uso de Code Review Multi-Angulo no Gate 4 (Refactor TDD)

> **Camada:** Caso de Uso (Dominio)
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-001]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/review_task_code.rs
test_file: crates/trustkit-core/tests/gate4_code_review_use_case_tests.rs

---

## Contexto
Durante o ciclo TDD no Gate 4, apos a comprovacao das fases `RED` e `GREEN`, a fase `Refactor` deve permitir a execucao da inspecao multi-angulo (`code-review`) antes da homologacao definitiva da tarefa atomica (`trustkit task complete`), prevenindo bugs logicos sutis, quebra de contratos cross-file e violacoes de convencoes.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/review_task_code.rs`:
- `ReviewTaskCodeRequest` (`cycle_id`, `task_id`, `effort_level: CodeReviewEffortLevel`, `max_findings_override: usize`, `now_iso`).
- `ReviewTaskCodeOutcome` (`cycle_id`, `task_id`, `effort_level`, `findings: Vec<SemanticReviewFinding>`, `clean_pass: bool`, `recorded_evidence_id: u64`).
- Validacao previa de que a tarefa ja possui prova da fase `GREEN` (ou retorno de `DomainViolation::TddLifecycleViolation`).
- Invocacao da porta `SemanticCodeReviewerPort`, filtragem por confianca (`>= 7/10`) e ordenacao priorizando bugs de corretude (`LogicCorrectness`, `RemovedBehavior`, `CrossFileContract`) sobre simplificacao e estilo.
- Persistencia da evidencia `EvidenceKind::CodeReview` via `CycleEvidenceRepositoryPort`.

### Fora do escopo
- Abertura de Pull Request ou comentario remoto em GitHub/GitLab.

## Dependencias
- TASK-001

## Premissas
- Bugs de corretude sempre precedem apontamentos de simplificacao, reuso, eficiencia e convencoes quando o teto de achados do nivel de esforco for atingido.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** uma tarefa com fases `RED` e `GREEN` registradas no ciclo ativo, **quando** `review_task_code::execute` e chamado no nivel `Medium`, **entao** avalia os 8 angulos, aplica o teto de 8 achados e grava `EvidenceKind::CodeReview` no repositorio.
- [x] **Dado** uma tarefa que ainda nao registrou a fase `GREEN`, **quando** a revisao de codigo e solicitada, **entao** falha com `DomainViolation::TddLifecycleViolation`.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-core/src/use_cases/complete_task.rs`.

## Rastreabilidade
- Origem: RF-035, RN-003
