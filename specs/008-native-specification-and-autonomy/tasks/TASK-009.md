# TASK-009: Atualizar Documentacao Principal em Paridade Bilingue

> **Camada:** Documentacao
> **Estimativa:** P
> **Status:** Pendente

depends_on: [TASK-008]
layer: DOCS
Arquivos Esperados:
- README.md
- README.en.md
test_file: crates/trustkit-cli/src/cli_contract_help_tests.rs

---

## Contexto
Com a consolidacao da autonomia nativa e a adicao dos comandos `trustkit spec prompt` e `trustkit spec lint`, os manuais principais do projeto (`README.md` em portugues e `README.en.md` em ingles) devem ser atualizados em perfeita paridade de conteudo e sem emojis.

## Escopo
Atualizar `README.md` e `README.en.md`:
- Descrever o Gate 1 nativo com `trustkit spec prompt` e linter de especificacao.
- Documentar as flags e subcomandos de `spec prompt` e `spec lint`.
- Detalhar as novas ferramentas MCP de especificacao expostas em `trustkit mcp serve`.
- Atualizar a tabela de comandos suportados e codigos de saida semanticos.
- Manter zero emojis e zero trailers de autoria IA em ambos os arquivos.

### Fora do escopo
- Documentacao de arquitetura interna dos crates (ADR).

## Dependencias
- TASK-008

## Premissas
- Estrita paridade bilingue de secoes, tabelas e comandos entre `README.md` e `README.en.md`.

## Criterio de Aceite
- [ ] **Dado** os arquivos `README.md` e `README.en.md`, **quando** auditados, **entao** ambos detalham os comandos `trustkit spec prompt` e `trustkit spec lint` com equivalencia tecnica total.
- [ ] **Dado** os dois documentos, **quando** inspecionados pelo teste de invariantes em `cli_contract_help_tests.rs`, **entao** nao possuem emojis Unicode nem trailers de IA.

## Notas Tecnicas
- Seguir o estilo e tom corporativo dos READMEs vigentes.

## Rastreabilidade
- Origem: RF-051, RN-009
