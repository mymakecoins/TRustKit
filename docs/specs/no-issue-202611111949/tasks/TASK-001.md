# TASK-001: Entidades GlossaryEntry e AdrTriageAssessment no Dominio Core

> **Camada:** CORE
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/knowledge_entities.rs
test_file: crates/trustkit-core/src/knowledge_entities_tests.rs

---

## Contexto
O TRustKit deve gerenciar um Glossario de Linguagem Ubiqua ativo na memoria perene (.trustkit/wiki/glossary.md) e realizar a triagem deterministica de 3 criterios para criacao de ADRs. Esta tarefa define os modelos de dominio centrais.

## Escopo
- Modelar `GlossaryEntry`, `DomainGlossary` e `AdrTriageAssessment` em `crates/trustkit-core/src/knowledge_entities.rs`.
- Avaliar os 3 criterios booleanos obrigatorios de ADR (`is_hard_to_reverse`, `is_surprising_without_context`, `has_genuine_alternatives_tradeoff`), gerando o veredito `SkipAdr` ou `ApproveAdr`.
- Validar que termos canonicos rejeitam detalhes de implementacao (snippets de codigo ou caminhos de arquivo) na definicao.
- Implementar suite de testes em `crates/trustkit-core/src/knowledge_entities_tests.rs`.
