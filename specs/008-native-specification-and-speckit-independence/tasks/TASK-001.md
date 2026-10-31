# TASK-001: Implementar Entidades de Dominio e Scaffolding de Prompt de Especificacao

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/spec_scaffold_entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/src/spec_scaffold_entities_tests.rs

---

## Contexto
Para emancipar o TRustKit de frameworks externos de especificacao em linguagem natural mantendo os Principios Constitucionais I (Isolamento Hexagonal com std only) e II (Proibicao absoluta de NULL), e necessario introduzir no core os modelos de dados e contratos que governam o scaffolding deterministico de prompts e esqueletos de especificacao.

## Escopo
Implementar em `crates/trustkit-core/src/spec_scaffold_entities.rs` e registrar em `crates/trustkit-core/src/entities.rs` e `crates/trustkit-core/src/ports.rs`:
- `SpecPromptRequest`: `idea: String`, `context_hops: usize`, `token_budget: usize`.
- `SpecPromptPayload`: `system_instructions: String`, `scaffold_markdown: String`, `context_summary: String`, `token_estimate: usize`.
- `SpecScaffoldTemplate`: gerador deterministico do esqueleto padrao `spec.md` com secoes obrigatorias (Product Trio, Diagrama Mermaid flowchart LR, Criterios Gherkin e Matriz de Rastreabilidade).
- Traits de porta abstrata em `crates/trustkit-core/src/ports.rs` para recuperacao de contexto e composicao de prompt sem dependencias externas fora de `std`.

### Fora do escopo
- Chamadas a APIs de Modelos de Linguagem (proibicao estrita no nucleo Rust).
- I/O direto em disco ou manipulacao de sockets.

## Dependencias
- Nenhuma

## Premissas
- Zero dependencias externas em `trustkit-core` (`std` apenas).
- Proibicao absoluta de valores `null` ou campos `Option` desnecessarios.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [ ] **Dado** uma instancia de `SpecPromptRequest` com orcamento de tokens valido, **quando** o template de scaffolding e gerado, **entao** retorna estrutura estrita contendo o esqueleto formal do `spec.md` com marcadores para Product Trio, Mermaid e Gherkin.
- [ ] **Dado** uma requisicao com orcamento de tokens zero ou negativo, **quando** validada, **entao** retorna `DomainViolation::AstIntegrityOrInvariantViolation`.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-core/src/spec_entities.rs`.

## Rastreabilidade
- Origem: RF-043, RN-001
