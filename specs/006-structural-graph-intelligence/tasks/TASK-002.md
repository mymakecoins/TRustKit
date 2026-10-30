# TASK-002: Implementar Extrator Sintatico AST de Nos e Arestas de Codigo

> **Camada:** Analise Estatica AST
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-001]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/ast_graph_extractor.rs
- crates/trustkit-ast/src/rust_graph_extractor.rs
- crates/trustkit-ast/src/ts_graph_extractor.rs
test_file: crates/trustkit-ast/tests/ast_graph_extractor_tests.rs

---

## Contexto
O grafo estrutural do repositorio precisa extrair nos de arquivos, simbolos publicos e arestas de acoplamento diretamente da arvore sintatica (`tree-sitter`) para Rust e TypeScript/JavaScript, sem qualquer chamada de LLM e com latencia de poucos milissegundos.

## Escopo
Implementar `AstGraphExtractor` em `crates/trustkit-ast/src/ast_graph_extractor.rs`:
- Extracao de simbolos publicos em Rust (`pub struct`, `pub enum`, `pub trait`, `pub fn`, `pub mod`) como `StructuralNode` com `GraphNodeKind::PublicSymbol`.
- Extracao de simbolos exportados em TypeScript/JS (`export interface`, `export type`, `export class`, `export function`) como `StructuralNode`.
- Extracao de declaracoes de dependencia (`use`, `mod`, `import ... from`) gerando `StructuralEdge` com `EdgeConfidence::Extracted` quando o destino e resolvido deterministicamente ou `EdgeConfidence::Inferred` quando mapeado por sufixo de caminho relativo.
- Unificacao opcional com as arestas `WikiLink` extraidas por `knowledge_link_extractor.rs`.

### Fora do escopo
- Persistencia em banco de dados SQLite ou interface de linha de comando.

## Dependencias
- TASK-001

## Premissas
- Tempo de extracao inferior a `5ms` por arquivo analisado.
- Zero chamadas a provedores de IA ou processos externos.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** um arquivo Rust contendo `use crate::ports::SpecValidatorPort` e `pub struct SpecValidator`, **quando** processado pelo `AstGraphExtractor`, **entao** retorna os nos correspondentes e a aresta de importacao com `EdgeConfidence::Extracted`.
- [x] **Dado** um arquivo TypeScript com imports de modulos locais, **quando** processado pelo extrator, **entao** gera os nos de simbolos exportados e arestas direcionadas sem panics.

## Notas Tecnicas
- Reutilizar os parsers `tree-sitter-rust` e `tree-sitter-typescript` ja configurados no `Cargo.toml` de `trustkit-ast`.

## Rastreabilidade
- Origem: RF-026, RN-002
