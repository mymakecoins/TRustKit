# TASK-009: Configurar Reexportacoes e Facade na Crate trustkit-ast

> **Camada:** Integração
> **Estimativa:** P
> **Status:** Pendente

depends_on:
- TASK-004
- TASK-007
- TASK-008
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/lib.rs
test_file: crates/trustkit-ast/tests/ast_facade_tests.rs

---

## Contexto
A crate `trustkit-ast` precisa disponibilizar publicamente todos os adaptadores de Camada 0 de forma organizada e modular, mantendo o encapsulamento interno.

## Escopo
- Reexportar `InvariantLexicalScanner`, `TestIntegrityAuditor` e `AstContextSlicer`.
- Implementar a fachada unificada `TreeSitterAstAuditorAdapter` satisfazendo as traits do core.

### Fora do escopo
- Lógica de persistência em banco SQLite (coberta por TASK-010).

## Dependências
- TASK-004 — Scanner léxico de invariantes.
- TASK-007 — Auditor de testes.
- TASK-008 — Fatiador de contexto.

## Premissas
- Limite de 250 LOC por arquivo respeitado.

## Critério de Aceite
- [ ] **Dado** os consumidores da crate `trustkit-ast`, **quando** importarem o módulo raiz, **então** todos os adaptadores estão acessíveis via trait pública.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/lib.rs`.

## Rastreabilidade
- Origem: RF-007, RN-009
