# TASK-003: Injecao Automatica de Vocabulario Ubiquo em trustkit spec prompt

> **Camada:** CORE
> **Estimativa:** M
> **Status:** Pendente

depends_on: ['TASK-001', 'TASK-002']
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/spec_prompt_builder.rs
- crates/trustkit-core/src/knowledge_entities.rs
test_file: crates/trustkit-core/tests/spec_context_composer_tests.rs

---

## Contexto
Ao preparar o contexto para novos agentes especificadores criarem especificacoes formais, o TRustKit deve injetar automaticamente os termos canonicos do glossario e orientar o banimento de termos rejeitados.

## Escopo
- Integrar a secao `ubiquitous_glossary` no `spec_prompt_builder` do `trustkit-core`.
- Assegurar que os termos cadastrados sao formatados de forma clara para o prompt do agente.
- Cobrir a composicao de contexto com testes unitarios em `crates/trustkit-core/tests/spec_context_composer_tests.rs`.
