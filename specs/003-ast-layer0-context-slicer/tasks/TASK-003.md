# TASK-003: Configurar Gramaticas e Dependencias Tree-Sitter no Workspace

> **Camada:** Infra/Setup
> **Estimativa:** P
> **Status:** Concluída

depends_on: []
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/Cargo.toml
test_file: crates/trustkit-ast/tests/grammar_setup_tests.rs

---

## Contexto
A Camada 0 de auditoria requer parsing em tempo de execução com performance `< 5ms`. Tree-Sitter é o motor escolhido para inspeção de nós sintáticos de Rust e TypeScript.

## Escopo
- Adicionar dependências `tree-sitter`, `tree-sitter-rust` e `tree-sitter-typescript` em `crates/trustkit-ast/Cargo.toml`.
- Configurar flags e compilação incremental das gramáticas.

### Fora do escopo
- Lógica de extração de nós ou queries (coberta pelas tarefas subsequentes).

## Dependências
- Nenhuma

## Premissas
- Versões compatíveis e estáveis das gramáticas no ecossistema Rust.

## Critério de Aceite
- [x] **Dado** o crate `trustkit-ast`, **quando** `cargo check -p trustkit-ast` for executado, **então** compila sem warnings.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/Cargo.toml`.

## Rastreabilidade
- Origem: RF-007, RN-003
