# TASK-010: Sincronizar Diagramas Interativos Archify

> **Camada:** Diagramas / Archify
> **Estimativa:** P
> **Status:** Concluido

depends_on: [TASK-009]
layer: DIAGRAMS
Arquivos Esperados:
- docs/diagrams/01-trustkit-architecture.html
- docs/diagrams/02-trustkit-7gates-workflow.html
- docs/diagrams/05-trustkit-feature-lifecycle.html
test_file: crates/trustkit-ast/tests/lexical_scanner_tests.rs

---

## Contexto
Os diagramas arquiteturais interativos em formato HTML/SVG autônomo gerenciados pelo Archify em `docs/diagrams/` precisam refletir a nova jornada de especificacao nativa do TRustKit (Gate 1 e Gate 2) e a integracao com o servidor MCP nativo.

## Escopo
Atualizar as definicoes e renderizar os diagramas standalone HTML:
- `docs/diagrams/01-trustkit-architecture.html`: incluir o despachante de ferramentas MCP de especificacao e os componentes do `spec_linter`.
- `docs/diagrams/02-trustkit-7gates-workflow.html`: atualizar o no do Gate 1 destacando o scaffolding de prompt com fatiamento de contexto e linter em Rust.
- `docs/diagrams/05-trustkit-feature-lifecycle.html`: atualizar o fluxo de ciclo de vida da feature integrando a geracao de prompt a partir de linguagem natural e auto-correcao agêntica.

### Fora do escopo
- Alteracao da engine de renderizacao do Archify.

## Dependencias
- TASK-009

## Premissas
- Arquivos gerados como HTML standalone com inline SVG e sem dependencias externas de rede.
- Zero sobreposicao de labels e suporte completo a temas Dark/Light.

## Criterio de Aceite
- [x] **Dado** os arquivos HTML atualizados em `docs/diagrams/`, **quando** inspecionados, **entao** sao documentos HTML standalone validos contendo o SVG renderizado e os novos nos arquiteturais.
- [x] **Dado** os novos arquivos em `docs/diagrams/`, **quando** analisados pelo scanner de invariantes, **entao** nao possuem emojis ou trailers de IA.

## Notas Tecnicas
- Validar conformidade de tags e ausencia de caracteres proibidos.

## Rastreabilidade
- Origem: RF-052, RN-010
