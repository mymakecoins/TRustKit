# TASK-004: Regra SPEC-008 de Conflito Terminologico no SpecLinter

> **Camada:** AST
> **Estimativa:** M
> **Status:** Pendente

depends_on: ['TASK-001', 'TASK-002']
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/spec_linter.rs
test_file: crates/trustkit-ast/tests/spec_linter_tests.rs

---

## Contexto
O `SpecLinter` deve analisar arquivos `spec.md` contra termos ambiguos ou banidos cadastrados no glossario ativo do projeto, gerando violacoes deterministicas antes do congelamento (Gate 2).

## Escopo
- Adicionar regra `SPEC-008` ao `SpecLinter` em `crates/trustkit-ast/src/spec_linter.rs`.
- Escanear o texto da especificacao mapeando linhas e indicando o termo rejeitado encontrado e o termo canonico substituto.
- Garantir tempo de execucao inferior a 3ms.
- Implementar testes de cobertura em `crates/trustkit-ast/tests/spec_linter_tests.rs`.
