# TASK-003: Resolucao e Validacao de Citacoes Exatas de Linhas nos Milestones

> **Camada:** AST
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-001
- TASK-002
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/milestone_citation.rs
- crates/trustkit-ast/src/lib.rs
test_file: crates/trustkit-ast/tests/milestone_citation_tests.rs

---

## Contexto
O requisito constitucional e de negocio exige que cada marco contenha citacoes explicitas e verificaveis dos arquivos PRD e ADR de origem, no formato `caminho/arquivo#Lxx-Lyy`, comprovando quais requisitos funcionais e decisoes tecnicas fundamentam aquele milestone.

## Escopo
- Criar `crates/trustkit-ast/src/milestone_citation.rs`.
- Implementar validacao de intervalos de linha contra os arquivos reais em disco (garantir que as linhas citadas existem e nao ultrapassam o final do arquivo).
- Formatador de blocos de citacao e links relativos padronizados nos arquivos de milestone.
- Escrever testes em `crates/trustkit-ast/tests/milestone_citation_tests.rs`.
