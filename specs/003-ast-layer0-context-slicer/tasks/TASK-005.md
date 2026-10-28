# TASK-005: Implementar Parser Tree-Sitter de Assercoes para Rust

> **Camada:** Integração
> **Estimativa:** M
> **Status:** Concluída

depends_on:
- TASK-003
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/rust_assertion_counter.rs
test_file: crates/trustkit-ast/tests/rust_assertion_counter_tests.rs

---

## Contexto
Para impedir enfraquecimento de testes (Anti-Cheat, RN-004), o TRustKit precisa inspecionar a AST de arquivos `.rs` e contar nós de macro invocados (`assert!`, `assert_eq!`, `assert_ne!`, etc.) antes e após a execução do Builder.

## Escopo
- Criar analisador Tree-Sitter para Rust que percorre a árvore sintática e identifica invocações de macros de teste.
- Computar total de asserções ativas por função de teste `#[test]`.

### Fora do escopo
- Análise de testes TypeScript (coberta por TASK-006).

## Dependências
- TASK-003 — Gramática Tree-Sitter de Rust configurada.

## Premissas
- Asserções comentadas não devem ser computadas como nós válidos.

## Critério de Aceite
- [x] **Dado** um arquivo de teste Rust com 3 macros `assert_eq!`, **quando** analisado, **então** retorna contagem exata de 3 asserções.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/rust_assertion_counter.rs`.

## Rastreabilidade
- Origem: RF-007, RN-005, RN-004
