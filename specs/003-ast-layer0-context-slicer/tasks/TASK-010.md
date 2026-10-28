# TASK-010: Implementar Persistencia de Auditorias AST e Evidencias no SQLite

> **Camada:** Banco
> **Estimativa:** M
> **Status:** Concluída

depends_on:
- TASK-001
layer: DB
Arquivos Esperados:
- crates/trustkit-db/src/sqlite_adapter.rs
test_file: crates/trustkit-db/tests/sqlite_ast_audit_tests.rs

---

## Contexto
O Gate 5 exige o princípio "The Event Log Proves": nenhum gate é considerado superado sem o registro determinístico de auditoria gravado no SQLite WAL local (`.trustkit/state.db`).

## Escopo
- Adicionar operações para persistir vereditos da Camada 0 (`save_ast_audit_evidence`).
- Atualizar tabela `execution_evidences` com o tipo `AstAudit` e veredito correspondente (`Pass` ou `Fail`).

### Fora do escopo
- Lógica de orquestração de use case (coberta por TASK-011).

## Dependências
- TASK-001 — Requer tipos de veredito de AST do domínio.

## Premissas
- Escrita atômica em transação SQLite WAL.

## Critério de Aceite
- [x] **Dado** uma evidência de auditoria AST, **quando** persistida no banco, **então** é recuperável com todos os campos intactos e zero valores nulos.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-db/src/sqlite_adapter.rs`.

## Rastreabilidade
- Origem: RF-007, RN-002, RN-010
