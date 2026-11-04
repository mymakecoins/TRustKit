# Analise: Independencia do Spec Kit na Especificacao a partir de Linguagem Natural

Data da Analise: 2026-10-31  
Status: Aprovado para Pesquisa e Arquitetura  
Escopo: Avaliacao dos requisitos tecnicos e arquiteturais necessarios para que o TRustKit opere na geracao de especificacoes formais a partir de linguagem natural sem depender do framework externo Spec Kit, mantendo intactos os principios constitucionais do nucleo em Rust.

---

## 1. Contexto e Problema

No ecossistema atual de Spec-Driven Development (SDD) do TRustKit, ha uma clara divisao entre:
- **Motor de Governanca Deterministico (Rust Core)**: responsavel pela fiscalizacao mecanica (AST, Git Worktrees, SQLite WAL, selo BLAKE3, invariantes anti-cheat e Gauntlet Loop).
- **Capacidade Cognitiva / Redacao de Especificacao (Skills de IA)**: delegada a um conjunto de skills herdadas do ecossistema Spec Kit (`.agents/skills/speckit-*`), em especial `speckit-specify`.

O principio constitucional basico do nucleo em Rust do TRustKit estabelece:
> Zero LLM calls, zero heavy regex, zero panic.

Portanto, o binario `trustkit` e estatico e deterministico: nao consome APIs probabilisticas de Modelos de Linguagem e nao realiza inferencia semantica direta.

Esta analise detalha como tornar o TRustKit 100% auto-suficiente na fase de especificacao a partir de linguagem natural, preservando os principios do nucleo em Rust.

---

## 2. Divisao Atual: TRustKit vs speckit-specify

| Etapa | speckit-specify (Skill / Agente LLM) | trustkit (Binario Rust Core) |
|---|---|---|
| Ponto de partida | Ideia abstrata ou requisito em linguagem natural. | Documento Markdown estruturado (PRD ou rascunho de spec). |
| Papel | Autor Criativo / Raciocinio: projeta a arquitetura, cria cenarios Gherkin (Given/When/Then) e desenha diagramas de sequencia Mermaid. | Arbitro / Fiscalizador: parsing mecanico via Tree-sitter, validacao de contratos sintaticos e selagem criptografica. |
| Execucao | Executado pelo assistente no chat via prompt/skill. | Executado via CLI (`trustkit spec init`, `trustkit spec lock`). |

Sem o suporte de uma instrucao/skill para o Modelo de Linguagem, o desenvolvedor humano precisaria redigir manualmente o `spec.md` completo para que o `trustkit` pudesse auditar e selar.

---

## 3. Requisitos para Independencia Total do Spec Kit

Para que o TRustKit elimine qualquer dependencia do Spec Kit e assuma a lideranca desse fluxo mantendo seus principios constitucionais, quatro frentes arquiteturais sao necessarias:

### Frente 1: Unificacao na Thin Skill Nativa do TRustKit
Substituir o diretorio de skills externas (`.agents/skills/speckit-*`) por uma definicao nativa no TRustKit:
- Incorporar os prompts de especificacao diretamente na skill nativa `trustkit-sdd` ou em uma skill dedicada `trustkit-specify`.
- Essa skill instrui os agentes (Claude Code, Antigravity, Codex) a produzirem diretamente o formato exigido pelo TRustKit:
  - Regras de Negocio identificadas como `RN-XXX`.
  - Requisitos Funcionais identificados como `RF-XXX`.
  - Diagramas de fluxo e arquitetura em Mermaid validos.
  - Criterios de aceitacao estritamente em Gherkin.
  - Proibicao absoluta de emojis e trailers de autoria IA.

### Frente 2: Prompt Scaffolding e Fatiamento de Contexto no Rust Core
O nucleo em Rust nao executa chamadas a LLM, mas atua como o **provedor deterministico de contexto e prompt estruturado**:
- Implementacao de comando CLI:
  ```bash
  trustkit spec prompt --idea "<texto_da_ideia>" [--context-hops N] [--json]
  ```
  ou extensao do comando `trustkit spec init`.
- O que o Rust executa em milissegundos (< 5ms):
  1. Consulta a memoria viva do projeto (`.trustkit/wiki/` via SQLite FTS5 com BM25) para recuperar regras de negocio correlacionadas.
  2. Executa fatiamento de grafo AST (`trustkit context slice`) para mapear estruturas e funcoes impactadas.
  3. Gera deterministicamente um payload contendo: regras constitucionais, invariantes de projeto, esqueleto formal do `spec.md` e o contexto de codigo relevante.
- O agente LLM apenas recebe esse payload estruturado e preenche as secoes conceituais.

### Frente 3: Exposicao de Ferramentas via Servidor MCP Nativo
O TRustKit ja possui servidor MCP em Rust (`trustkit-orch/src/mcp_server.rs`).
- Adicionar ferramentas MCP voltadas a especificacao e planejamento:
  - `trustkit_spec_template`: entrega o contrato formal e esqueleto estruturado do `spec.md`.
  - `trustkit_spec_context`: entrega o contexto fatiado (AST + documentacao FTS5) para uma feature.
  - `trustkit_spec_validate`: executa verificacao semantica previa do rascunho de especificacao.
- O agente consome ferramentas padronizadas MCP diretamente do TRustKit, sem acoplamento a ferramentas ou extensoes proprietarias externas.

### Frente 4: Validador Sintatico / Linter de Especificacao em Rust
Criar um comando intermediario de verificacao rapida antes do congelamento:
```bash
trustkit spec lint --file specs/<feature>/spec.md --json
```
- O Rust audita a especificacao gerada pelo agente em latencia < 3ms:
  - Todas as regras de negocio e requisitos estao codificados (`RN-*`, `RF-*`)?
  - Ha cobertura de cenarios Gherkin (`Given / When / Then`) para cada requisito?
  - Diagramas Mermaid compilam sem erros de sintaxe?
  - Ha ausencia rigorosa de emojis e trailers de IA?
- Caso encontre inconsistencias, o `trustkit` retorna um payload JSON com violacoes semanticas para que o agente de IA auto-corrija o artefato antes da invocacao final do `trustkit spec lock`.

---

## 4. Resumo Arquitetural

Com essas quatro implementacoes:
1. **O Rust Core permanece intocado em seus principios constitucionais**: zero chamadas a APIs probabilisticas de LLM, tempo de resposta deterministico em milissegundos, isolamento hexagonal estrito e persistencia transacional em SQLite WAL.
2. **A IA externa atua exclusivamente onde seu papel e insubstituivel**: interpretacao de linguagem natural, sintese conceitual e raciocinio criativo sobre a ideia do usuario.
3. **O TRustKit torna-se 100% auto-suficiente**: fornece templates, contexto (AST + FTS5), integracao de agentes via MCP nativo e validacao rigorosa por linter e selo BLAKE3, eliminando completamente a necessidade de qualquer skill ou framework do Spec Kit.
