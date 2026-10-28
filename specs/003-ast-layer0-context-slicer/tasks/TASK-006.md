# TASK-006: Implementar Parser Tree-Sitter de Assercoes para TypeScript

> **Camada:** Integração
> **Estimativa:** M
> **Status:** Concluída

depends_on:
- TASK-003
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/ts_assertion_counter.rs
test_file: crates/trustkit-ast/tests/ts_assertion_counter_tests.rs

---

## Contexto
Aplicações web do ecossistema utilizam Jest, Vitest ou Playwright. O auditor deve detectar se chamadas como `expect(...)` foram excluídas, comentadas ou suprimidas com `.skip()`.

## Escopo
- Analisar ASTs `.ts` e `.tsx` via Tree-Sitter TypeScript/TSX.
- Contabilizar nós de chamada `expect(...)` e sinalizar modificadores `.skip` ou `.todo`.

### Fora do escopo
- Parser de Rust (coberto por TASK-005).

## Dependências
- TASK-003 — Gramática Tree-Sitter configurada.

## Premissas
- Detecção precisa de encadeamentos de assertions (`expect().toBe()`, `expect().toEqual()`).

## Critério de Aceite
- [x] **Dado** um teste com `test.skip`, **quando** analisado, **então** acusa supressão de teste.
- [x] **Dado** asserções ativas, **quando** computadas, **então** retorna o total exato de nós.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/ts_assertion_counter.rs`.

## Rastreabilidade
- Origem: RF-007, RN-006, RN-004
