# Data Model: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

Este modelo define as entidades puras de dominio, representacoes de persistencia SQLite e transicoes de estado (Typestate FSM) para a Fase 2 do TRustKit, em conformidade com o Principio II da Constituicao (Proibicao Absoluta de NULL).

---

## 1. Domain Entities (`trustkit-core::entities`)

### UpstreamDocument
Representa o documento bruto de requisitos de entrada (PRD ou ADR).
- **path**: `String` - Caminho absoluto ou relativo para o arquivo no sistema de arquivos.
- **title**: `String` - Titulo principal extraido do primeiro cabecalho H1.
- **topic_slug**: `String` - Slug kebab-case derivado do titulo para nomenclatura de artefatos.
- **business_rules**: `Vec<BusinessRule>` - Colecao de regras de negocio extraidas de tabelas Markdown.
- **functional_requirements**: `Vec<FunctionalRequirement>` - Colecao de requisitos funcionais extraidos.

### BusinessRule
- **rule_id**: `String` - Identificador da regra (ex: `"RN-001"`).
- **description**: `String` - Descricao semantica da regra de negocio.

### FunctionalRequirement
- **req_id**: `String` - Identificador do requisito funcional (ex: `"RF-001"`).
- **description**: `String` - Descricao comportamental do requisito.

### TraceabilityRow
- **business_rule_id**: `String` - Referencia a `RN-NNN`.
- **functional_req_id**: `String` - Referencia a `RF-NNN`.
- **task_id**: `String` - Referencia a tarefa planejada (`TASK-NNN`).
- **mermaid_node**: `String` - Identificador do no de jornada correspondente no Mermaid.
- **playwright_spec_path**: `String` - Caminho do teste automatizado correspondente.
- **evidence_screenshot**: `String` - Nome do arquivo de screenshot esperado (ex: `"SC-001.png"`).

### SpecificationV2
- **cycle_id**: `String` - Identificador do ciclo SDD ativo.
- **feature_slug**: `String` - Nome da funcionalidade.
- **spec_file_path**: `String` - Caminho do arquivo de especificacao em disco.
- **mermaid_content**: `String` - Bloco do diagrama Mermaid (`flowchart LR`/`TD`).
- **gherkin_scenarios**: `Vec<GherkinScenario>` - Cenarios de aceitacao Given/When/Then.
- **traceability_matrix**: `Vec<TraceabilityRow>` - Matriz fechada.
- **lock_state**: `SpecLockState` - Estado de congelamento da especificacao.

### SpecLockState (Enum)
- `DraftUnverified` - Rascunho inicial em elaboracao; impede execucao de ondas.
- `Frozen { blake3_digest: String, locked_at: String }` - Especificacao criptograficamente selada e imutavel.

### TaskNode
- **task_id**: `String` - Identificador sequencial da tarefa (ex: `"TASK-001"`).
- **title**: `String` - Titulo descritivo da tarefa.
- **architectural_layer**: `String` - Camada arquitetural (ex: `"CORE"`, `"DB"`, `"CLI"`).
- **depends_on**: `Vec<String>` - Lista de `task_id` que devem ser concluidas previamente.
- **expected_files**: `Vec<String>` - Lista de caminhos de arquivos de codigo afetados pela tarefa.
- **target_test_file**: `String` - Arquivo de teste de unidade/integracao vinculado.

### WaveSchedule
- **cycle_id**: `String` - Ciclo associado.
- **total_tasks**: `usize` - Quantidade total de tarefas planejadas.
- **waves**: `Vec<ExecutionWave>` - Lista ordenada de ondas de execucao sequencial.

### ExecutionWave
- **wave_index**: `usize` - Indice numerico da onda (base 1).
- **tasks**: `Vec<TaskNode>` - Lista de tarefas que executam concorrentemente nesta onda sem conflito de arquivo.

---

## 2. Typestate & FSM Transitions

```text
[ Gate 0: Discovered ]
          │
          ▼  (trustkit spec init --from-prd)
[ Gate 1: Context Sliced / Spec Draft ]
  spec_lock_state = "DRAFT_UNVERIFIED"
          │
          ▼  (trustkit spec lock)
          ├──────────────────────────────────────────┐
          │ (Validacao falhou: gap ou sintaxe)        │ (Validacao aprovada)
          ▼                                          ▼
[ Rejeicao: SpecLockRejected ]              [ Gate 2: Spec Frozen ]
  status mantido DRAFT_UNVERIFIED             spec_lock_state = "FROZEN"
                                              blake3 gravado no SQLite
                                                     │
                                                     ▼  (trustkit wave plan)
                                            [ Gate 3: Waves Planned ]
                                              WaveSchedule gravado
```

---

## 3. SQLite Persistence Mapping (`.trustkit/state.db`)

Todos os campos sao obrigatoriamente `NOT NULL`.

### Tabela `sdd_cycles` (Campos Atualizados na Fase 2)
| Coluna | Tipo | Restricao | Descricao |
| :--- | :--- | :--- | :--- |
| `spec_lock_state` | `TEXT` | `NOT NULL CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN'))` | Estado criptografico da especificacao. |
| `spec_blake3_digest` | `TEXT` | `NOT NULL DEFAULT 'UNSEALED_DRAFT'` | Hash BLAKE3 de 64 caracteres hexadecimais quando congelada. |
| `current_gate` | `INTEGER` | `NOT NULL CHECK (current_gate BETWEEN 0 AND 6)` | Avanca para 2 apos `spec lock` e para 3 apos `wave plan`. |

### Tabela `artifact_seals` (Insercao no Gate 2)
| Coluna | Tipo | Restricao | Descricao |
| :--- | :--- | :--- | :--- |
| `cycle_id` | `TEXT` | `NOT NULL REFERENCES sdd_cycles(cycle_id)` | Vinculo com o ciclo ativo. |
| `file_path` | `TEXT` | `NOT NULL` | Caminho do arquivo de especificacao congelado. |
| `artifact_category` | `TEXT` | `NOT NULL CHECK (artifact_category = 'SPEC')` | Categoria do selo. |
| `blake3_digest` | `TEXT` | `NOT NULL` | Assinatura imutavel calculada. |
| `ast_assertion_count` | `INTEGER` | `NOT NULL DEFAULT 0` | Contagem de assercoes AST. |
| `contract_unlock_state` | `TEXT` | `NOT NULL DEFAULT 'SEALED'` | Estado de destravamento (apenas por acao humana explicita). |

### Tabela `atomic_tasks` (Insercao/Sincronizacao no Gate 3)
| Coluna | Tipo | Restricao | Descricao |
| :--- | :--- | :--- | :--- |
| `cycle_id` | `TEXT` | `NOT NULL REFERENCES sdd_cycles(cycle_id)` | Ciclo ativo. |
| `task_id` | `TEXT` | `NOT NULL` | Identificador da tarefa (`TASK-001`). |
| `title` | `TEXT` | `NOT NULL` | Titulo da tarefa. |
| `architectural_layer` | `TEXT` | `NOT NULL` | Camada (CORE, DB, AST, CLI). |
| `wave_index` | `INTEGER` | `NOT NULL CHECK (wave_index >= 1)` | Onda topologica calculada pelo DAG. |
| `depends_on_json` | `TEXT` | `NOT NULL DEFAULT '[]'` | Array JSON de dependencias. |
| `expected_files_json` | `TEXT` | `NOT NULL DEFAULT '[]'` | Array JSON de arquivos afetados. |
