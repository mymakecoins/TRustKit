# TASK-004: Gerador de Briefings Paralelos Design It Twice no Spec Scaffold

> **Camada:** CORE
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/spec_scaffold_entities.rs
test_file: crates/trustkit-core/src/spec_scaffold_entities_tests.rs

---

## Contexto
Durante o Gate 1, agentes e desenvolvedores devem explorar multiplas alternativas arquiteturais antes de congelar uma porta ou modulo profundo, aplicando o padrao Design It Twice de Ousterhout.

## Escopo
- Definir `DesignItTwiceStrategy` e `DesignItTwiceBrief` em `crates/trustkit-core/src/spec_scaffold_entities.rs` sem campos nulos.
- Implementar `SpecScaffoldTemplate::generate_design_it_twice_briefs` produzindo 3 briefings contrastantes (`Minimizacao de Interface`, `Maxima Flexibilidade`, `Otimizacao para o Chamador Comum`).
- Estender `SpecPromptRequest` e `SpecPromptPayload` para suportar `design_it_twice`.
- Escrever testes unitarios em `crates/trustkit-core/src/spec_scaffold_entities_tests.rs`.
