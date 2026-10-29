# TASK-005: Implementar Caso de Uso de Ingestao Automatica no Gate 2 (Spec Lock)

> **Camada:** Dominio / Casos de Uso
> **Estimativa:** P
> **Status:** Concluida

depends_on: [TASK-004]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/ingest_locked_spec_knowledge.rs
test_file: crates/trustkit-core/tests/sdd_gate2_knowledge_hook_tests.rs

---

## Contexto
Quando uma especificacao e congelada no Gate 2 (`trustkit spec lock`), o conhecimento formal (regras de negocio, requisitos funcionais, tarefas planejadas e selo criptografico BLAKE3) deve ser persistido automaticamente na memoria perene do repositorio para consulta imediata pelos agentes construtores.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/ingest_locked_spec_knowledge.rs`:
- Caso de uso deterministico que recebe os metadados do `LockSpecificationOutcome` e o conteudo da spec congelada.
- Sintetiza e grava o documento `.trustkit/wiki/specs/spec-<cycle_id>.md` com tags `["spec", "frozen", "<cycle_id>"]`, digest BLAKE3 e wikilinks para cada tarefa da matriz (`[[tasks/TASK-NNN]]`).

### Fora do escopo
- Ingestao de tarefas do Gate 4 ou incidentes do Gate 5.

## Dependencias
- TASK-004

## Premissas
- A ingestao ocorre de forma sincrona e deterministica logo apos o congelamento da spec, em `< 5ms`.

## Criterio de Aceite
- [x] **Dado** que `lock_specification` obteve `SPEC_FROZEN`, **quando** o caso de uso de ingestao do Gate 2 e invocado, **entao** a nota `.trustkit/wiki/specs/spec-<cycle_id>.md` e criada e indexada no FTS5 com links para as tasks associadas.

## Notas Tecnicas
- Arquivo alvo: `crates/trustkit-core/src/use_cases/ingest_locked_spec_knowledge.rs`.

## Rastreabilidade
- Origem: RF-021, RN-005
