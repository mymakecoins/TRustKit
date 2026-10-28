# TASK-015: Integrar Adaptadores no Composition Root em bootstrap.rs

> **Camada:** Infra/Setup
> **Estimativa:** P
> **Status:** Pendente

depends_on:
- TASK-009
- TASK-014
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/bootstrap.rs
test_file: crates/trustkit-cli/tests/bootstrap_ast_tests.rs

---

## Contexto
Toda a fiação de dependências do TRustKit ocorre exclusivamente no Composition Root (`bootstrap.rs`), respeitando o Princípio V (Separação entre Construção e Execução).

## Escopo
- Instanciar `TreeSitterAstAuditorAdapter` e injetá-lo no caso de uso correspondente.
- Atualizar a função despachante principal do CLI.

### Fora do escopo
- Testes E2E com processos externos (cobertos por TASK-016).

## Dependências
- TASK-009 — Facade da crate `trustkit-ast`.
- TASK-014 — Handlers da CLI.

## Premissas
- Sem singletons globais ou mutabilidade compartilhada.

## Critério de Aceite
- [ ] **Dado** invocação de `trustkit gauntlet ast-audit`, **quando** o bootstrap executa, **então** resolve as dependências e despacha para o handler.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-cli/src/bootstrap.rs`.

## Rastreabilidade
- Origem: RF-007, RN-015
