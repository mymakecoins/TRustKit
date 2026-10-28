# CLI Contract: `trustkit spec lock`

## Command Signature
```bash
trustkit spec lock [--spec <PATH>] [--json]
```

## Description
Valida formalmente a especificacao ativa (Mermaid `flowchart`, Gherkin `Dado/Quando/Entao` e Matriz de Rastreabilidade fechada sem regras orfas). Em caso de aprovacao, calcula o digest `BLAKE3`, grava o `ArtifactSeal` no banco SQLite e transiciona o ciclo para `FROZEN`.

## Arguments & Flags
- `--spec <PATH>` *(Opcional)*: Caminho para o arquivo da especificacao. Se omitido, busca o caminho no ciclo ativo registrado no banco SQLite local (`.trustkit/state.db`).
- `--json` *(Opcional)*: Emite o resultado em JSON puro.

## Exit Codes
- `0`: Sucesso. Especificacao congelada (`spec_lock_state: "FROZEN"`).
- `3`: Falha de Validacao / Lacuna de Rastreabilidade (`TraceabilityGap` ou sintaxe Mermaid/Gherkin invalida). O arquivo permanece como rascunho (`DRAFT_UNVERIFIED`).

## JSON Payload Structure (Success - Stdout)
```json
{
  "status": "SPEC_FROZEN",
  "cycle_id": "CYC-20261027-001",
  "spec_file_path": "/path/to/spec.md",
  "spec_lock_state": "FROZEN",
  "blake3_digest": "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
  "locked_at": "2026-10-27T17:30:00Z",
  "rules_count": 8,
  "requirements_count": 12,
  "tasks_count": 15,
  "e2e_tests_count": 8
}
```

## JSON Payload Structure (Failure - Stderr / Exit Code 3)
```json
{
  "status": "SPEC_LOCK_REJECTED",
  "cycle_id": "CYC-20261027-001",
  "spec_lock_state": "DRAFT_UNVERIFIED",
  "rejection_kind": "TraceabilityGap",
  "orphan_rule": "RN-002",
  "message": "Regra de negocio RN-002 nao possui TASK ou Screenshot correspondente na Matriz de Rastreabilidade"
}
```
