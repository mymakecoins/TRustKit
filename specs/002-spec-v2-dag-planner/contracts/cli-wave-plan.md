# CLI Contract: `trustkit wave plan`

## Command Signature
```bash
trustkit wave plan [--tasks-dir <DIR>] [--json]
```

## Description
Analisa o conjunto de tarefas atomicas (`TASK-NNN.md` e `INDEX.md`), constroi o Grafo Aciclico Direcionado (DAG) via `petgraph`, valida a ausencia de ciclos e agrupa as tarefas em ondas topologicas de execucao concorrente. Tarefas com arquivos modificados em comum sao separadas deterministicamente em ondas distintas subsequentes ordenadas pela menor numeracao (`TASK-NNN`).

## Arguments & Flags
- `--tasks-dir <DIR>` *(Opcional)*: Caminho para o diretorio contendo as tarefas atomicas. Se omitido, busca o caminho associado ao ciclo ativo em `.trustkit/state.db`.
- `--json` *(Opcional)*: Emite o cronograma de ondas em formato JSON.

## Preconditions
- A especificacao do ciclo DEVE estar congelada (`spec_lock_state: "FROZEN"`). Tentativas com estado `DRAFT_UNVERIFIED` serao rejeitadas com orientacao instrutiva.

## Exit Codes
- `0`: Sucesso. Ondas planejadas e registradas.
- `1`: Ciclo de governanca nao congelado (`DRAFT_UNVERIFIED`).
- `4`: `DagValidationError::CycleDetected`. Aborta imediatamente com a lista do ciclo.

## JSON Payload Structure (Success - Stdout)
```json
{
  "status": "WAVES_PLANNED",
  "cycle_id": "CYC-20261027-001",
  "total_tasks": 3,
  "total_waves": 2,
  "waves": [
    {
      "wave_index": 1,
      "tasks": [
        {
          "task_id": "TASK-001",
          "title": "Implementar modelo puro de dados",
          "architectural_layer": "CORE",
          "depends_on": [],
          "expected_files": ["crates/trustkit-core/src/entities.rs"]
        }
      ]
    },
    {
      "wave_index": 2,
      "tasks": [
        {
          "task_id": "TASK-002",
          "title": "Implementar adaptador SQLite",
          "architectural_layer": "DB",
          "depends_on": ["TASK-001"],
          "expected_files": ["crates/trustkit-db/src/sqlite_adapter.rs"]
        },
        {
          "task_id": "TASK-003",
          "title": "Adicionar parser Markdown",
          "architectural_layer": "AST",
          "depends_on": ["TASK-001"],
          "expected_files": ["crates/trustkit-ast/src/parser.rs"]
        }
      ]
    }
  ]
}
```

## JSON Payload Structure (Cycle Detected - Stderr / Exit Code 4)
```json
{
  "status": "CYCLE_DETECTED",
  "cycle_id": "CYC-20261027-001",
  "error_kind": "DagValidationError::CycleDetected",
  "cycle_path": ["TASK-001", "TASK-002", "TASK-001"],
  "message": "Dependencia circular detectada entre tarefas: TASK-001 -> TASK-002 -> TASK-001"
}
```
