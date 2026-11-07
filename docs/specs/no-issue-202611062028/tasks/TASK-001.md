# TASK-001: Extracao e Analise Combinada de PRD e ADR no AST

> **Camada:** AST
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/milestone_extractor.rs
- crates/trustkit-ast/src/lib.rs
test_file: crates/trustkit-ast/tests/milestone_extractor_tests.rs

---

## Contexto
Para permitir que o TRustKit entenda e decomponha grandes sistemas antes da atomizacao de tarefas, precisamos de um extrator estatico que leia o PRD e o ADR do projeto, identifique as capacidades de negocio, os problemas resolvidos, quem recebe valor, bem como as restricoes arquiteturais e dependencias tecnicas.

## Escopo
- Criar o modulo `crates/trustkit-ast/src/milestone_extractor.rs`.
- Implementar o parser que le os arquivos Markdown de PRD e ADR mapeando titulos, secoes, listas e tabelas.
- Rastrear a linha inicial e final de cada bloco de conteudo extraido para viabilizar citacoes exatas.
- Expor a funcao de extracao e contratos associados no `lib.rs` do `trustkit-ast`.
- Escrever testes automatizados em `crates/trustkit-ast/tests/milestone_extractor_tests.rs`.
