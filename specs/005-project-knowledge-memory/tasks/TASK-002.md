# TASK-002: Implementar Extrator Lexico de Frontmatter YAML e Wikilinks

> **Camada:** Analise Sintatica / AST
> **Estimativa:** M
> **Status:** Concluida

depends_on: [TASK-001]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/knowledge_link_extractor.rs
test_file: crates/trustkit-ast/tests/knowledge_link_extractor_tests.rs

---

## Contexto
Os documentos da Wiki em `.trustkit/wiki/` possuem cabecalho frontmatter YAML (`title`, `tags`, `updated_at`) e referencias internas no padrao `[[slug-destino]]` ou `[[slug-destino|alias]]`. E necessario um parser deterministico e sem regex pesadas para extrair esses metadados e construir o grafo de adjacencia.

## Escopo
Implementar em `crates/trustkit-ast/src/knowledge_link_extractor.rs`:
- Funcao deterministica `extract_frontmatter_and_body(raw_md: &str)` retornando titulo, lista de tags e corpo limpo.
- Funcao deterministica `extract_wikilinks(source_slug: &str, body: &str) -> Vec<WikiLink>` que identifica todos os blocos `[[...]]` fora de blocos de codigo cercados (fenced code blocks), normalizando slugs e descartando duplicatas.
- Gerador deterministico de frontmatter YAML na gravacao de notas.

### Fora do escopo
- Gravacao em banco de dados SQLite.

## Dependencias
- TASK-001

## Premissas
- Zero chamadas de LLM; execucao puramente lexica em `< 1ms` por documento.
- Ignorar falsos positivos de `[[...]]` dentro de blocos craseados de codigo.

## Criterio de Aceite
- [x] **Dado** um documento Markdown com frontmatter e links `[[adrs/adr-001]]` e `[[specs/spec-005|Especificacao 5]]`, **quando** processado pelo extrator, **entao** retorna as arestas normalizadas `adrs/adr-001` e `specs/spec-005`.
- [x] **Dado** um trecho `[[nao-link]]` dentro de um bloco de codigo Rust/Bash, **quando** processado, **entao** o extrator ignora a ocorrencia.

## Notas Tecnicas
- Arquivo alvo: `crates/trustkit-ast/src/knowledge_link_extractor.rs`.

## Rastreabilidade
- Origem: RF-018, RN-002
