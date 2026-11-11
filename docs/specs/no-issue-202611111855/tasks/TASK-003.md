# TASK-003: Extracao AST de Costuras e Classificacao RealSeam vs HypotheticalSeam

> **Camada:** AST
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/ast_graph_extractor.rs
- crates/trustkit-ast/src/rust_graph_extractor.rs
- crates/trustkit-ast/src/ts_graph_extractor.rs
test_file: crates/trustkit-ast/tests/ast_graph_extractor_tests.rs

---

## Contexto
Costuras (Seams) como `Ports` (traits Rust e interfaces TypeScript) devem ser justificadas por variacao real (dois ou mais adaptadores concretos ou fakes de teste). Abstracoes com apenas um adaptador devem ser classificadas como `HypotheticalSeam`.

## Escopo
- Extrair relacoes de implementacao de traits/interfaces (`impl Trait for Type` em Rust e `implements Interface` em TypeScript) no `AstGraphExtractor`.
- Classificar cada porta/costura em `RealSeam` (2+ adaptadores/fakes) ou `HypotheticalSeam` (0 ou 1 adaptador) no `AstGraphExtractor` e `GraphHealthAuditor`.
- Escrever testes de integracao em `crates/trustkit-ast/tests/ast_graph_extractor_tests.rs`.
