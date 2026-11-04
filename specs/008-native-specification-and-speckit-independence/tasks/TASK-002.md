# TASK-002: Implementar Compositor de Contexto AST e Memoria Wiki FTS5

> **Camada:** Caso de Uso / Orquestracao
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/compose_spec_context.rs
test_file: crates/trustkit-core/tests/spec_context_composer_tests.rs

---

## Contexto
Para fornecer contexto tecnico rico ao agente sem depender de servicos externos de LLM, o TRustKit precisa compor os resultados de busca FTS5 BM25 da wiki viva (`.trustkit/wiki/`) com o fatiamento de AST (`context_slicer`) das entidades mais acopladas.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/compose_spec_context.rs`:
- Estrutura `ComposeSpecContextRequest`: `idea: String`, `hops: usize`, `max_snippets: usize`.
- Estrutura `ComposeSpecContextOutcome`: `wiki_articles: Vec<WikiSnippet>`, `ast_slices: Vec<AstSliceSnippet>`, `aggregated_context: String`.
- Caso de uso que interage com as portas `KnowledgeMemoryPort` e `AstContextSlicerPort` para recuperar documentos wiki correlacionados e fatias AST de funcoes e structs impactadas, truncando o resultado dentro do orcamento de linhas.

### Fora do escopo
- Conexao direta com banco SQLite ou parse Tree-sitter no use case (tudo mediado via portas).

## Dependencias
- TASK-001

## Premissas
- Composicao 100% deterministica sem alteracoes probabilisticas de texto.
- Ordenacao estavel de snippets por score BM25 decrescente.

## Criterio de Aceite
- [ ] **Dado** termos-chave extraidos de uma ideia de feature, **quando** o caso de uso e executado, **entao** agrega artigos wiki relevantes e nos AST correlacionados dentro do limite de hops informado.
- [ ] **Dado** uma base sem correspondencias exatas, **quando** executado, **entao** retorna contexto agregado vazio sem falhas ou panics.

## Notas Tecnicas
- Seguir isolamento hexagonal estrito de `crates/trustkit-core/src/use_cases/`.

## Rastreabilidade
- Origem: RF-044, RN-002
