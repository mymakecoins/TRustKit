# TASK-006: Implementar Caso de Uso de Ingestao Automatica no Gate 4 (Task Complete)

> **Camada:** Dominio / Casos de Uso
> **Estimativa:** P
> **Status:** Concluida

depends_on: [TASK-004]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/ingest_completed_task_knowledge.rs
test_file: crates/trustkit-core/tests/sdd_gate4_knowledge_hook_tests.rs

---

## Contexto
Ao concluir uma tarefa atomica no Gate 4 com comprovacao do ciclo TDD (`RED_VERIFIED` -> `GREEN_VERIFIED`), e essencial registrar na memoria perene o que foi entregue, quais arquivos foram alterados e as evidencias associadas para permitir handoff continuo entre Builders e ondas subsequentes.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/ingest_completed_task_knowledge.rs`:
- Caso de uso que recebe os dados da `AtomicTask` concluida, o SHA do commit e os resumos de evidencia RED e GREEN.
- Gera e indexa `.trustkit/wiki/tasks/<task_id>.md` com tags `["task", "tdd-verified", "<layer>"]`, wikilink para a especificacao do ciclo (`[[specs/spec-<cycle_id>]]`) e lista dos arquivos implementados.

### Fora do escopo
- Execucao de comandos Git ou verificacao de fases TDD (ja realizadas em `complete_task.rs`).

## Dependencias
- TASK-004

## Premissas
- Preservar rastreabilidade cruzada entre a nota da tarefa e a nota da especificacao via wikilink.

## Criterio de Aceite
- [x] **Dado** uma tarefa que transicionou para `GreenVerified`, **quando** o caso de uso de ingestao do Gate 4 e executado, **entao** o documento `.trustkit/wiki/tasks/<task_id>.md` e persistido e torna-se pesquisavel por nome de arquivo ou ID da tarefa.

## Notas Tecnicas
- Arquivo alvo: `crates/trustkit-core/src/use_cases/ingest_completed_task_knowledge.rs`.

## Rastreabilidade
- Origem: RF-022, RN-006
