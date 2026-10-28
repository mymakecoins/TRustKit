# TASK-007: Implementar Auditor Anti-Cheat de Integridade de Testes

> **Camada:** Integração
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-002
- TASK-005
- TASK-006
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/test_integrity_auditor.rs
test_file: crates/trustkit-ast/tests/test_integrity_tests.rs

---

## Contexto
O auditor de integridade consolida a comparação entre a contagem de asserções gravada no Gate 2 (`artifact_seals`) e a contagem real após a intervenção dos Builders no Gate 4.

## Escopo
- Implementar `TestIntegrityAuditor` comparando a contagem de nós da AST antes e depois das modificações.
- Retornar `TestIntegrityViolation` caso haja decréscimo injustificado de asserções.

### Fora do escopo
- Persistência em banco (coberta por TASK-010).

## Dependências
- TASK-002 — Porta do domínio.
- TASK-005 — Contador de asserções Rust.
- TASK-006 — Contador de asserções TypeScript.

## Premissas
- Qualquer remoção de asserção é tratada como tentativa de trapaça (Anti-Cheat).

## Critério de Aceite
- [ ] **Dado** um teste com asserção removida, **quando** auditado, **então** retorna `AstAuditVerdict::Rejected` com `Exit Code: 4`.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/test_integrity_auditor.rs`.

## Rastreabilidade
- Origem: RF-007, RN-007, RN-004
