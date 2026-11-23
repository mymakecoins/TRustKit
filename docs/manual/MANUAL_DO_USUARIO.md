# Manual de Uso do TRustKit

**Versão da Documentação:** 1.2.0  
**Motor:** `trustkit` / `tk` (Rust Nativo — 6 Crates)  
**Ambiente:** Spec-Driven Development (SDD), Isolamento em Git Worktrees e Gauntlet Loop V2  
**Data:** Novembro de 2026  

---

## Sumario

1. [Visao Geral & Boas-Vindas](#1-visao-geral--boas-vindas)
   - [1.1 O que e o TRustKit e o Alias tk](#11-o-que-e-o-trustkit-e-o-alias-tk)
   - [1.2 Apresentador Dual-Audience e Orientacao de Proximo Passo](#12-apresentador-dual-audience-e-orientacao-de-proximo-passo)
   - [1.3 Para quem se destina este manual?](#13-para-quem-se-destina-este-manual)
   - [1.4 O Paradigma Hybrid Rust Core + Thin Skill](#14-o-paradigma-hybrid-rust-core--thin-skill)
     - [1.4.1 Catalogo das Thin Skills e Slash Commands do Framework SDD](#141-catalogo-das-thin-skills-e-slash-commands-do-framework-sdd)
   - [1.5 Principios Constitucionais de Governanca](#15-principios-constitucionais-de-governanca)
2. [Primeiros Passos & Instalacao](#2-primeiros-passos--instalacao)
   - [2.1 Requisitos do Sistema](#21-requisitos-do-sistema)
   - [2.2 Compilacao e Instalacao dos Binarios (`trustkit` e `tk`)](#22-compilacao-e-instalacao-dos-binarios-trustkit-e-tk)
   - [2.3 Modos de Operacao: Humano vs Agente](#23-modos-de-operacao-humano-vs-agente)
   - [2.4 Diagnosticando a Sessao Atual e o Proximo Passo (`tk status`)](#24-diagnosticando-a-sessao-atual-e-o-proximo-passo-tk-status)
   - [2.5 Guia de Adocao: Novo Projeto vs Projeto em Andamento](#25-guia-de-adocao-novo-projeto-vs-projeto-em-andamento)
3. [Guia Operacional Passo a Passo (A Jornada dos 7 Gates)](#3-guia-operacional-passo-a-passo-a-jornada-dos-7-gates)
   - [3.1 Tarefa 1: Iniciar um Ciclo de Desenvolvimento (Gate 0)](#31-tarefa-1-iniciar-um-ciclo-de-desenvolvimento-gate-0)
   - [3.2 Tarefa 2: Formular, Validar e Congelar a Especificacao (Gates 1 & 2)](#32-tarefa-2-formular-validar-e-congelar-a-especificacao-gates-1--2)
   - [3.3 Tarefa 3: Decompor Tarefas e Planejar Ondas em DAG Expand-Contract (Gate 3)](#33-tarefa-3-decompor-tarefas-e-planejar-ondas-em-dag-expand-contract-gate-3)
   - [3.4 Tarefa 4: Execucao Paralela, Worktrees, TDD e Dual-Axis Review (Gate 4)](#34-tarefa-4-execucao-paralela-worktrees-tdd-e-dual-axis-review-gate-4)
   - [3.5 Tarefa 5: Submeter ao Gauntlet Loop e Revisao de Seguranca (Gate 5)](#35-tarefa-5-submeter-ao-gauntlet-loop-e-revisao-de-seguranca-gate-5)
   - [3.6 Tarefa 6: Certificacao Oficial e Fechamento com `close` (Gate 6)](#36-tarefa-6-certificacao-oficial-e-fechamento-com-close-gate-6)
   - [3.7 Exemplo Pratico: Ciclo Completo de Feature com 3 Waves](#37-exemplo-pratico-ciclo-completo-de-feature-com-3-waves)
   - [3.8 Orquestracao Autonoma de Ponta a Ponta (`trustkit auto` / `tk auto`)](#38-orquestracao-autonoma-de-ponta-a-ponta-trustkit-auto--tk-auto)
4. [Recursos Avancados e Modulos Especializados](#4-recursos-avancados-e-modulos-especializados)
   - [4.1 Memoria Perene do Projeto, Glossario Ubiquo e Triagem ADR (`tk memory`)](#41-memoria-perene-do-projeto-glossario-ubiquo-e-triagem-adr-tk-memory)
   - [4.2 Inteligencia de Grafo Estrutural (`tk graph`)](#42-inteligencia-de-grafo-estrutural-tk-graph)
   - [4.3 Fatiamento Cirurgico de Contexto (`tk context`)](#43-fatiamento-cirurgico-de-contexto-tk-context)
   - [4.4 Protecao Proativa com Fast Hooks (`tk hook`)](#44-protecao-proativa-com-fast-hooks-tk-hook)
   - [4.5 Servidor Nativo MCP — 23 Ferramentas Expostas (`tk mcp`)](#45-servidor-nativo-mcp--23-ferramentas-expostas-tk-mcp)
   - [4.6 Auditoria Global Pre-Release (`tk audit security`)](#46-auditoria-global-pre-release-tk-audit-security)
5. [Perguntas Frequentes (FAQ)](#5-perguntas-frequentes-faq)
6. [Guia de Solucao de Problemas (Troubleshooting)](#6-guia-de-solucao-de-problemas-troubleshooting)
   - [6.1 Tabela de Codigos de Saida Semanticos (POSIX)](#61-tabela-de-codigos-de-saida-semanticos-posix)
   - [6.2 Mensagens Comuns de Bloqueio e Como Resolver](#62-mensagens-comuns-de-bloqueio-e-como-resolver)
   - [6.3 Diagnostico e Recuperacao com o Assistente Doutor (`tk doctor`)](#63-diagnostico-e-recuperacao-com-o-assistente-doutor-tk-doctor)
7. [Glossario de Termos e Linguagem Ubiqua](#7-glossario-de-termos-e-linguagem-ubiqua)

---

## 1. Visao Geral & Boas-Vindas

### 1.1 O que e o TRustKit e o Alias tk?

O **TRustKit** e um motor de governanca deterministica e execucao de alta velocidade desenvolvido em Rust nativo. Ele foi projetado especificamente para gerenciar fluxos de **Spec-Driven Development (SDD)** em projetos de software modernos, onde atuam equipes multidisciplinares e agentes autonomos de inteligencia artificial (Claude Code, Antigravity, Cursor, Codex).

Diferente de abordagens baseadas exclusivamente em instrucoes textuais que podem sofrer com desvios ou alucinacoes, o TRustKit delega as decisoes criticas a um binario compilado de alta performance (`trustkit`) e seu alias contratual oficial conciso (`tk`). Esse motor valida sintaxes em microssegundos com parsers Tree-Sitter nativos, fiscaliza a integridade dos testes de ponta a ponta sob TDD mecanico, isola tarefas em Git Worktrees efemeras com Copy-on-Write (reflink), impede alteracoes indevidas em regras de negocio congeladas via hashes BLAKE3 e sincroniza o estado das entregas em um banco transacional auditavel (SQLite WAL).

A paridade entre `trustkit` e `tk` e estrita: ambos os binarios sao gerados no mesmo workspace Cargo e contam com resolucao dinamica de `argv0`, adaptando automaticamente o texto de ajuda e as mensagens de erro ao nome invocado pelo operador.

### 1.2 Apresentador Dual-Audience e Orientacao de Proximo Passo

O TRustKit implementa o padrao arquitetural **Dual-Audience Presenter**, concebido para que humanos e agentes agênticos colaborem sem atritos de comunicacao:

- **Modo Humano (Terminal TTY):** Exibe texto legivel com marcadores visuais padronizados (`[PASS]`, `[FAIL]`, `[INFO]`) e finaliza a saida com o bloco formatado `NEXT STEP:`, indicando explicitamente a proxima acao priorizada (`[PRIMARY]`, `[SECONDARY]`, `[OPTIONAL]`) e sua justificativa.
- **Modo Agente (Flag `--json`):** Retorna uma estrutura JSON puramente deterministica, sem sequencias de escape ANSI e **estritamente livre de valores nulos (null-free)**, enriquecida com o array `next_steps` calculado pelo `NextStepResolver`.

### 1.3 Para quem se destina este manual?

Este manual serve como guia pratico para quatro papeis fundamentais:

* **Desenvolvedores de Software e Arquitetos:** Que precisam planejar, codificar e entregar features complexas garantindo que cada teste e regra de negocio permaneca integro.
* **Operadores e Agentes Agênticos:** Que executam comandos de terminal de forma autonoma e demandam respostas sem ambiguidades, com baixo consumo de memoria e sem campos nulos (`--json`).
* **Lideres Tecnicos (Tech Leaders):** Que precisam auditar o cumprimento dos requisitos antes de autorizar a integracao de codigo no repositorio principal.
* **Analistas de Qualidade (QA) e Homologadores:** Que buscam rastreabilidade matematica entre a regra de negocio acordada, os testes executados e as capturas de tela geradas.

### 1.4 O Paradigma Hybrid Rust Core + Thin Skill

Para aliar a flexibilidade do raciocinio humano e agêntico a solidez de regras imutaveis, o TRustKit adota uma arquitetura em duas camadas:

1. **Thin Skill Declarativa (`SKILL.md`):** Camada leve que reside no contexto do desenvolvedor ou agente, fornecendo orientacoes de alto nivel e fluxos conceituais sem sobrecarregar a janela de contexto.
2. **Native Rust Core (`trustkit` / `tk`):** Binario estatico compilado em Rust que executa 100% das mutacoes de estado, validacoes sintaticas de AST, congelamento criptografico BLAKE3, criacao de Git Worktrees e verificacao estrita de TDD.

```
+--------------------------------------------------------------------+
|                         Desenvolvedor / Agente                     |
+--------------------------------------------------------------------+
                                  |
                                  v
+--------------------------------------------------------------------+
|               Camada 1: Thin Skill Declarativa (Prompt)            |
|       (Raciocinio de Produto, Arquitetura e Decisoes de Codigo)     |
+--------------------------------------------------------------------+
                                  |
                        Comandos CLI / OpenRPC MCP
                                  |
                                  v
+--------------------------------------------------------------------+
|               Camada 2: Native Rust Core (trustkit / tk)           |
|   - Persistencia Transacional SQLite WAL (.trustkit/state.db)      |
|   - Validador AST Tree-Sitter (< 3ms) e Anti-Cheat de Testes       |
|   - Isolamento Paralelo de Tarefas em Git Worktrees (CoW)          |
|   - Dual-Axis Review (Fowler Smells + Scope Creep)                 |
|   - Fechamento Formal com 'close' e Transicao no Board             |
|   - Selo Criptografico BLAKE3 para Especificacoes Congeladas       |
|   - Servidor MCP Nativo (23 Tools), Grafo AST e Memoria Wiki FTS5  |
+--------------------------------------------------------------------+
```

#### 1.4.1 Catalogo das Thin Skills e Slash Commands do Framework SDD

As Thin Skills residem em `.agents/skills/` e guiam os assistentes agênticos atraves de slash commands conceituais, delegando a execucao mecanica aos binarios `trustkit` e `tk`:

| Slash Command | Thin Skill (`SKILL.md`) | Gate Principal | Finalidade e Entregaveis |
| :--- | :--- | :--- | :--- |
| `/sdd-define` | `sdd-define` | Gate 1 | Formulacao da especificacao funcional em `spec.md`, personas do Product Trio e criterios BDD. |
| `/sdd-refine` | `sdd-refine` | Gate 1 | Identificacao de ambiguidades na especificacao e rodadas interativas de perguntas/respostas. |
| `/sdd-principles` | `sdd-principles` | Gate 2 | Gestao da constituicao do projeto (`constitution.md`), diretrizes arquiteturais e regras inegociaveis. |
| `/sdd-planning` | `sdd-planning` | Gate 2 | Planejamento de arquitetura, contratos de interfaces e modelagem de dados em `plan.md`. |
| `/sdd-decompose` | `sdd-decompose` | Gate 3 | Decomposicao sequenciada por dependencias em tarefas atomicas isoladas em `tasks.md`. |
| `/sdd-discover` | `sdd-discover` | Gate 3 | Auditoria nao destrutiva de consistencia cruzada entre `spec.md`, `plan.md` e `tasks.md`. |
| `/sdd-execute` | `sdd-execute` | Gate 4 | Execucao orientada das tarefas atomicas sob TDD estrito e isolamento em Git Worktrees. |
| `/sdd-resolve` | `sdd-resolve` | Gate 4 | Avaliacao comparativa do codigo construido contra os requisitos e convergencia de tarefas pendentes. |
| `/sdd-validate` | `sdd-validate` | Gate 5 | Geracao de checklists de qualidade de requisitos e homologacao previa ao Gauntlet Loop. |
| `/sdd-publish-issues` | `sdd-publish-issues` | Gate 6 | Sincronizacao idempotente de tarefas no GitHub Issues e boards de projetos. |
| `/tk` | `trustkit-sdd` | Transversal | Execucao imperativa direta do binario nativo com `$ARGUMENTS --json` e fast-path de alta economia de tokens. |
| N/A | `trustkit-sdd` | Transversal | Thin Skill mestre de governanca SDD, worktrees com CoW e consenso Gauntlet V2 em Rust nativo. |

### 1.5 Principios Constitucionais de Governanca

Todas as rotinas operadas pelo TRustKit sao regidas por regras fundamentais de qualidade e conformidade:

* **Invariante #1: Sem Codigo Sem Spec Congelada.** Nenhum arquivo de producao pode ser comitado ou implementado sem que a especificacao funcional esteja validada e selada com hash criptografico.
* **Invariante #2: Evidencia sobre Assercao.** Nenhuma etapa de teste ou compilacao e aprovada por declaracao verbal. O binario captura a saida literal, a duracao em milissegundos e o codigo de retorno POSIX.
* **Invariante #4: Integridade Imutavel de Testes (Anti-Cheat).** E proibido remover, comentar ou enfraquecer assercoes de testes existentes para forcar aprovacoes.
* **Invariante #5: Disjuntor de Seguranca (Circuit Breaker).** Se uma etapa de verificacao falhar por 5 vezes consecutivas, o sistema bloqueia novos ciclos e exige intervencao humana orientada.
* **Invariante #6: GitFlow Rigoroso.** Ramificacoes baseadas em `develop -> staging -> main`. Bloqueio de comandos destrutivos (`git reset --hard`, `git stash`, `git clean -fd`).
* **Invariante #7: Proibicao Absoluta de Emojis.** Toda comunicacao, log, arquivo de codigo e mensagem de commit deve empregar exclusivamente caracteres alfanumericos e marcadores textuais claros (ex: `[PASS]`, `[FAIL]`).
* **Invariante #12: Pull Request Sob Demanda.** Pull Requests nunca sao abertos automaticamente; dependem de solicitacao expressa do desenvolvedor via `tk pr`.
* **Invariante #13: Proibicao de Trailers de IA.** Vetado o uso de marcas como `Co-Authored-By: Claude` ou mensagens geradas automaticamente em commits e documentos.
* **Invariante #14: Orientacao Contextual Mandatoria de Proximo Passo.** Todo retorno do binario nativo emite recomendacoes contextuais via array `next_steps` (JSON) e rodape `NEXT STEP:` (ASCII), de consumo obrigatorio por agentes e desenvolvedores ao final de cada turno.
* **Invariante #15: Paridade Documental Mandatoria em Releases.** Toda atualizacao de comando, ciclo ou release do TRustKit deve ser publicada com sua base documental integralmente sincronizada (`README.md`, `README.en.md`, manuais em Markdown `docs/manual/MANUAL_DO_USUARIO.md` e `docs/manuals/user-guide.md`, bem como suas versoes HTML standalone `docs/manual/manual.html` e `docs/manuals/user-guide.html`). E terminantemente proibido promover ciclos ou releases para develop/main com documentacao defasada.

---

## 2. Primeiros Passos & Instalacao

### 2.1 Requisitos do Sistema

Antes de iniciar as atividades, confirme se as seguintes ferramentas estao presentes no ambiente operacional:

* **Sistema Operacional:** Linux (x86_64 ou AArch64) ou macOS (Darwin).
* **Compilador e Ferramental Rust:** Versao 1.82 ou superior com Cargo instalado.
* **Git:** Versao 2.38 ou superior (com suporte ativo a worktrees e sparse-checkout).
* **SQLite:** Versao 3.35 ou superior com suporte a modo WAL.
* **GitHub CLI (`gh`):** Autenticado na conta corporativa com acesso de leitura e escrita ao repositorio.

### 2.2 Compilacao e Instalacao dos Binarios (`trustkit` e `tk`)

Para compilar os binarios de producao do TRustKit no workspace atual:

```bash
cargo build --release --workspace
```

Os executaveis ficarao disponiveis em `target/release/trustkit` e `target/release/tk`. Para facilitar o acesso global:

```bash
sudo ln -sf "$(pwd)/target/release/trustkit" /usr/local/bin/trustkit
sudo ln -sf "$(pwd)/target/release/tk" /usr/local/bin/tk

# Verificar as versoes e o funcionamento de ambos
trustkit --version
tk --version
```

#### Auto-Atualizacao Automatizada (`trustkit update` / `tk update`)

Para verificar e promover atualizacoes diretamente a partir da branch `main` remota do repositorio oficial (`mymakecoins/TRustKit`):

```bash
# Apenas verificar se ha commits mais recentes na main remota
tk update --check
trustkit update --check --json

# Aplicar a atualizacao (fast-forward + build release no repo fonte, ou cargo install --git no ambiente global)
tk update

# Forcar rebuild e reinstalacao mesmo quando ja estiver na versao mais recente
tk update --force --json
```

Se a arvore de trabalho local contiver alteracoes pendentes ou se a compilacao falhar, o disjuntor semantico aborta a promocao e mantem os binarios atuais preservados sem corrupcao.

### 2.3 Modos de Operacao: Humano vs Agente

O TRustKit suporta nativamente dois formatos de saida ergonomicos:

- **Modo Humano (Padrao):** Apresenta tabelas limpas, texto legivel, prefixos padronizados como `[PASS]`, `[FAIL]`, `[GATE-0]` a `[GATE-6]` e o rodape contextual `NEXT STEP:` indicando os proximos comandos priorizados (`[PRIMARY]`, `[SECONDARY]`, `[OPTIONAL]`).
- **Modo Agente (Flag `--json` ou `TRUSTKIT_AGENT_MODE=1`):** Retorna dados em JSON minificado, sem formatacao ANSI, **sem valores nulos (null)** e enriquecidos com o array `next_steps` calculado pelo `NextStepResolver` (`priority`, `target_gate`, `command` e `description`).

### 2.4 Diagnosticando a Sessao Atual e o Proximo Passo (`tk status`)

```bash
tk status
```

**Saida Esperada (Modo Humano):**

```text
[STATUS] Ciclo Atual: issue-45-202611041030
[STATUS] Gate Atual: GATE_1_SPECIFICATION
[STATUS] Estado da Spec: DRAFT_UNVERIFIED
[STATUS] Invariantes: 16 de 16 Ativas
[STATUS] Banco SQLite WAL: .trustkit/state.db (Ok)

NEXT STEP:
  [PRIMARY]   (GATE_1_SPEC) tk spec lint --file docs/specs/issue-45-202611041030/spec.md
              -> Validate the specification structure, Mermaid diagram, Gherkin scenarios, and traceability matrix.
  [SECONDARY] (GATE_2_LOCK) tk spec lock --spec docs/specs/issue-45-202611041030/spec.md
              -> Freeze the specification with a BLAKE3 seal once linting passes.
```

### 2.5 Guia de Adocao: Novo Projeto vs Projeto em Andamento

O comando `tk init` inspeciona o ambiente, gerencia o `.gitignore` sem quebrar exclusoes existentes, provisiona o template `trustkit.toml`, inicializa o banco SQLite WAL e monta a infraestrutura do ciclo de forma transparente.

#### Cenario A: Como Configurar o TRustKit em um Projeto Novo (Sem Codigo)

```bash
# Passo 1: Preparar o repositorio Git
mkdir meu-novo-sistema && cd meu-novo-sistema
git init -b develop
git remote add origin git@github.com:organizacao/meu-novo-sistema.git
git commit --allow-empty -m "chore: initial repository bootstrap"

# Passo 2: Inicializar o SDD via tk
tk init --issue 1   # Vinculado a uma issue do GitHub
# ou:
tk init             # Ciclo puramente local
```

O comando `init` realiza automaticamente:
1. **Gerencia o `.gitignore`:** Adiciona `.trustkit/state.db*`, `.trustkit/worktrees/`, `.trustkit/audits/` e `!.trustkit/wiki/` sem alterar entradas pre-existentes.
2. **Gera o `trustkit.toml`:** Cria o arquivo de governanca do Gauntlet caso ausente.
3. **Provisiona a infraestrutura:** Cria `.trustkit/`, inicializa o banco SQLite WAL e monta `docs/specs/<ciclo>/`.

---

## 3. Guia Operacional Passo a Passo (A Jornada dos 7 Gates)

### 3.0 Planejamento Previo de Marcos de Entregas (Pre-Gate 0: `tk milestone`)

Antes de iniciar ciclos sobre especificacoes extensas, o TRustKit permite analisar conjuntamente o PRD e o ADR, mapear capacidades de negocio e restricoes tecnicas, gerar os marcos aciclicos em `docs/milestones/` com citacao exata de intervalos de linhas (`#LXX-LYY`) e persisti-los no SQLite WAL (suporta `tk milestones` e o alias singular `tk milestone`):

```bash
# Analisa PRD e ADR, gera M01..MNN.md com citacoes de linha e persiste no banco
tk milestone plan --prd docs/PRD_TRustKit.md --adr docs/ADR_001_TRustKit_Architecture.md

# Lista marcos cadastrados, ordem topologica, dependencias e status
tk milestone list

# Inspeciona detalhes completos, capacidades e citacoes de um marco
tk milestone inspect M01

# Inicializa o Gate 0 direcionado por marco planejado
tk init --milestone docs/milestones/M01-fundacao.md
```

### 3.1 Tarefa 1: Iniciar um Ciclo de Desenvolvimento (Gate 0)

Prepara a arvore de trabalho, valida a conta do GitHub e registra a sessao transacional.

```bash
# Iniciar ciclo vinculado a issue 45:
tk init --issue 45

# Ou importar regras a partir de um PRD:
tk init --issue 45 --from-prd docs/PRD_TRustKit.md
```

### 3.2 Tarefa 2: Formular, Validar e Congelar a Especificacao (Gates 1 & 2)

Garante que todos os requisitos funcionais, fluxos de tela e termos de negocio estejam blindados antes de programar.

#### Passo 1: Gerar o esqueleto da especificacao injetando o Glossario Ubiquo
```bash
tk spec prompt --idea "Autenticacao MFA via TOTP com codigos de recuperacao" --budget 4000
```

#### Passo 2: Validar a sintaxe da especificacao (Linter Pre-Lock em < 3ms)
```bash
# Valida sintaxe Gherkin, diagramas Mermaid, matriz e termos canonicos (regra SPEC-008)
tk spec lint
```

#### Passo 3: Congelar a especificacao com o Selo Criptografico BLAKE3 (Gate 2)
```bash
tk spec lock
```

### 3.3 Tarefa 3: Decompor Tarefas e Planejar Ondas em DAG Expand-Contract (Gate 3)

Valida as tarefas atomicas em `tasks/TASK-NNN.md` e organiza a ordem de execucao sem conflitos atraves da topologia Expand-Contract com calculo de Blast Radius:

```bash
tk wave plan
```

#### Inspecao Passiva da Hierarquia de Ondas (`tk wave view`)
Para inspecionar as tarefas agrupadas por onda sem recalcular o planejamento nem alterar estado:

```bash
# Inspecao visual formatada no terminal:
tk wave view

# Exportacao JSON deterministica livre de nulos:
tk wave view --json
```

### 3.4 Tarefa 4: Execucao Paralela, Worktrees, TDD e Dual-Axis Review (Gate 4)

Implementa cada tarefa em arvores isoladas, com notificacao estruturada de criacao, exigencia de comprovacao de falha (RED), revisao em dois eixos (Dual-Axis) e blindagem estrita de commit.

#### Passo 1: Despachar a onda de trabalho
```bash
tk wave run --wave 1
```

O sistema notifica explicitamente a criacao do worktree isolado no terminal:
```text
[INFO] Branch 'worktree/TASK-001' criada com sucesso.
[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/TASK-001
[INFO] Branch ativa do repositorio raiz preservada: 'develop'
[PASS] Wave 1 executed: 2/2 tasks passed in 48ms
```

#### Passo 2: Registrar a Fase RED (Comprovacao de Falha)
```bash
# Exige teste compilavel que falhe exclusivamente por assercao de dominio
tk task test TASK-001 --phase red
```

#### Passo 3: Implementar a solucao e registrar a Fase GREEN
```bash
tk task test TASK-001 --phase green
```

#### Passo 4: Realizar a Revisao de Codigo em Dois Eixos (Dual-Axis Review - Fase REFACTOR)
```bash
# Audita Eixo 1 (Scope Creep & Spec Compliance) e Eixo 2 (Fowler Smells via AST)
tk task review TASK-001 --effort medium
```

Saida formatada:
```text
=== DUAL-AXIS CODE REVIEW: TASK-001 ===
Overall Verdict: PASS

[Standards & Fowler Smells]
Verdict: PASS (findings: 0)
  (no findings)

[Spec Compliance & Scope Creep]
Verdict: PASS (findings: 0)
  (no findings)
```

#### Passo 5: Concluir e Homologar a Tarefa (Blindagem Estrita de Commit - RN-001)
```bash
# O desenvolvedor ou agente deve registrar o commit das alteracoes no worktree antes da conclusao:
git commit -am "feat: implement totp core domain logic"

# Conclui a tarefa atomica:
tk task complete TASK-001
```

> [!WARNING]
> **Regra RN-001: Bloqueio de Working Tree Suja:**  
> Se houver alteracoes nao comitadas no worktree, o comando `task complete` sera rejeitado com o erro:  
> `Cannot complete task 'TASK-001': working tree has N uncommitted changes (RN-001)`.

### 3.5 Tarefa 5: Submeter ao Gauntlet Loop e Revisao de Seguranca (Gate 5)

Auditoria profunda dividida em duas camadas deterministas:

#### Passo 1: Auditoria Sintatica Instantanea (Camada 0 em < 3ms)
```bash
# Audita anti-cheat, zero emojis e ausencia de trailers de IA via AST Tree-Sitter
tk gauntlet ast-audit
```

#### Passo 2: Disparar o Gauntlet Completo e Testes E2E (Camada 1)
```bash
# Execucao interativa padrao:
tk gauntlet run

# Ou forcando explicitamente a revisao de seguranca do diff:
tk gauntlet run --security-review

# Ou dispensando temporariamente a revisao de seguranca:
tk gauntlet run --skip-security-review
```

### 3.6 Tarefa 6: Certificacao Oficial e Fechamento com `close` (Gate 6)

Com todos os testes, invariantes e suites aprovadas, fechamos o ciclo com governanca formal.

#### Passo 1: Gerar o Relatorio de Certificacao e Comentario da Issue
```bash
tk done
```

Esse comando consolida o dossie oficial `gauntlet-report.md` e publica o comentario formal na issue vinculada do GitHub.

#### Passo 2: Fechar Formalmente o Ciclo Ativo (Comando Oficial: `close`)
```bash
tk close
```

O comando `tk close` valida que o ciclo foi certificado pelo Gauntlet (rejeitando fechamento de ciclos nao certificados), registra o fechamento no SQLite WAL e atualiza a coluna do card no GitHub Projects para `ReadyForTest`.

> [!IMPORTANT]
> **Depreciacao do Comando `merge`:**  
> O antigo comando `trustkit merge` foi descontinuado. Adote `trustkit close` (ou `tk close`) como comando padrao de fechamento.

#### Passo 3: Abrir o Pull Request (Sob Demanda Humana Explicita)
```bash
tk pr --target develop
```

O Pull Request formal e publicado na branch alvo apenas mediante solicitacao humana explicita.

### 3.7 Exemplo Pratico: Ciclo Completo de Feature com 3 Waves

```bash
# Gate 0: Inicializacao
tk init --issue 42

# Gate 1: Especificacao e Linter
tk spec prompt --idea "Modulo de autenticacao multifator TOTP" --budget 4000
tk spec lint

# Gate 2: Congelamento Criptografico BLAKE3
tk spec lock

# Gate 3: Planejamento topologico em ondas (DAG)
tk wave plan

# Gate 4: Execucao da Onda 1
tk wave run --wave 1
tk task test TASK-001 --phase red
tk task test TASK-001 --phase green
tk task review TASK-001 --effort medium
git commit -am "feat: implement totp core domain logic"
tk task complete TASK-001

tk task test TASK-002 --phase red
tk task test TASK-002 --phase green
tk task review TASK-002 --effort medium
git commit -am "feat: implement persistence adapter"
tk task complete TASK-002

# Gate 4: Execucao da Onda 2
tk wave run --wave 2
tk task test TASK-003 --phase red
tk task test TASK-003 --phase green
tk task review TASK-003 --effort medium
git commit -am "feat: implement totp auth service"
tk task complete TASK-003

# Gate 4: Execucao da Onda 3
tk wave run --wave 3
tk task test TASK-004 --phase red
tk task test TASK-004 --phase green
tk task review TASK-004 --effort medium
git commit -am "feat: implement cli handlers"
tk task complete TASK-004

# Gate 5: Gauntlet Loop V2
tk gauntlet ast-audit
tk gauntlet run

# Gate 6: Certificacao Oficial e Fechamento Formal
tk done
tk close
tk pr --target develop
```

### 3.8 Orquestracao Autonoma de Ponta a Ponta (`trustkit auto` / `tk auto`)

Para execucoes em que a especificacao formal ja existe ou os requisitos estao consolidados em uma issue do GitHub, o comando `tk auto` executa os 7 gates (0 ao 6) de forma autonoma com blindagens deterministicas:

- **Reconciliacao de Ciclos Concorrentes:** Inspeciona o banco SQLite WAL em busca de ciclos anteriores em status ativo. Em modo interativo (TTY), pergunta ao usuario se deseja reutilizar, fechar ciclos cujas branches ja foram mergeadas no Git ou arquivar rascunhos orfaos vazios. Em modo autonomo (`--json`), suporta flags declarativas (`--new`, `--close-merged`, `--reuse-cycle <ID>`) e emite diagnostico estruturado, prevenindo bloqueios em `stdin` e impedindo a criacao de ciclos zumbis orfaos no Gate 0.
- **Gate 0 (Setup & Init):** Inspeciona identidade Git, branches e baseline limpo, inicializando no SQLite WAL.
- **Gate 1 (Spec Lint):** Valida o linter nativo AST em microssegundos (sem emojis, Gherkin e matriz fechada).
- **Gate 2 (Spec Lock):** Sela a especificacao congelada com hash BLAKE3 criptografico.
- **Gate 3 (Wave Plan & Atomizacao):** Atomiza deterministicamente a especificacao se `tasks/` nao existir, dispara o disjuntor `EmptyTaskBacklogFault` se houver zero tarefas e calcula as ondas DAG.
- **Gate 4 (Multi-Wave TDD Loop & Builder Dispatch):** Sintetiza testes RED compilaveis, bloqueia filtros vazios (`ZeroFilteredTestsFault`) e erros de compilacao mascarados na fase RED (`CompilationFailureInRedPhaseFault`), despacha agentes construtores e comita alteracoes de tarefas em cada worktree antes da homologacao.
- **Gate 5 (Gauntlet V2):** Executa auditoria Camada 0 e consolida vereditos dos criticos adversariais, de contrato, estaticos e Playwright.
- **Gate 6 (Certify & Close):** Emite `gauntlet-report.md`, atualiza o card para `ReadyForTest` e realiza o fechamento formal via `close`.

```bash
# A partir de uma issue remota do GitHub:
tk auto --issue 45 --target develop

# A partir de um arquivo local de especificacao:
tk auto --spec docs/specs/minha-feature/spec.md --target develop

# Modo Agente com saida JSON determinista (livre de nulos):
tk auto --spec docs/specs/minha-feature/spec.md --target develop --runtime native --json
```

---

## 4. Recursos Avancados e Modulos Especializados

### 4.1 Memoria Perene do Projeto, Glossario Ubiquo e Triagem ADR (`tk memory`)

O modulo `memory` gerencia o conhecimento perene do sistema em `.trustkit/wiki/`, com indexacao SQLite FTS5 BM25, wikilinks, glossario canonico e triagem de decisoes tecnicas.

#### Comandos da Wiki
```bash
# Busca semantica lexical FTS5 com ranqueamento BM25:
tk memory search "persistencia atomica WAL" --limit 5

# Leitura de nota especifica por slug:
tk memory read adrs/adr-002-storage-strategy

# Gravacao atomica de nota na wiki:
tk memory write adrs/adr-002-storage-strategy --title "Estrategia de Armazenamento" --tags "adr,storage,sqlite" --content "Decisao de persistencia atomica..."

# Listagem de todas as notas cadastradas:
tk memory list

# Reconstrucao deterministica do indice FTS5 e grafo de wikilinks:
tk memory rebuild
```

#### Gestao de Glossario Ubiquo (Linguagem Canonica)
```bash
# Adicionar ou atualizar termo canonico no glossario:
tk memory glossary add --term "SddCycle" --definition "Instancia isolada e auditavel correspondente ao ciclo de vida de uma entrega" --rejected "sprint,task-bundle"

# Listar termos canonicos cadastrados:
tk memory glossary list
```

#### Triagem de Proposta de ADR (Filtro de 3 Criterios Arquiteturais)
```bash
# Avalia se a mudanca proposta atende aos 3 criterios de um ADR formal:
tk memory adr-check --title "Adocao de SQLite WAL para Persistencia" --context "Persistencia de evidencias de ciclo SDD" --hard-to-reverse --surprising --tradeoff
```

### 4.2 Inteligencia de Grafo Estrutural (`tk graph`)

```bash
# Construir ou atualizar o mapa estrutural (gera GRAPH_REPORT.md):
tk graph build

# Auditar a saude do grafo e detectar God Nodes:
tk graph health --top-god-nodes 5

# Encontrar o menor caminho de chamadas entre dois simbolos (Busca BFS):
tk graph path --source "crates/trustkit-core::SddCycle" --target "crates/trustkit-db::SqliteWalRepositoryAdapter"

# Explicar relacoes e dependencias de um componente:
tk graph explain --node "crates/trustkit-core/src/entities.rs::SddCycle"
```

### 4.3 Fatiamento Cirurgico de Contexto (`tk context`)

```bash
# Fatiamento classico de arquivo respeitando limite de tokens:
tk context slice --file crates/trustkit-core/src/lib.rs --budget 500

# Fatiamento multi-hop guiado pelo grafo estrutural (BFS):
tk context slice --file crates/trustkit-core/src/entities.rs --hops 2 --strategy bfs --budget 800
```

### 4.4 Protecao Proativa com Fast Hooks (`tk hook`)

Interceptador de chamadas de ferramentas agênticas que roda em microssegundos:

```bash
tk hook pre-tool --command "git add ."
```

Bloqueia comandos perigosos (`git stash`, `git reset --hard`) e commits com emojis ou trailers de IA antes da execucao.

### 4.5 Servidor Nativo MCP — 23 Ferramentas Expostas (`tk mcp`)

Inicia o servidor Model Context Protocol via stdio com 22 ferramentas deterministas:

```bash
tk mcp
```

| Modulo | Ferramenta MCP | Finalidade Operacional |
| :--- | :--- | :--- |
| **Ciclo SDD** | `trustkit_init` | Inicializa o ciclo SDD deterministico (Gate 0). |
| **Ciclo SDD** | `trustkit_spec_lock` | Valida e congela especificacao com digest BLAKE3 (Gates 1 e 2). |
| **Ciclo SDD** | `trustkit_wave_plan` | Gera planejamento topologico em ondas a partir do DAG (Gate 3). |
| **Ciclo SDD** | `trustkit_wave_view` | Inspeciona hierarquia de ondas e status TDD das tarefas do ciclo ativo. |
| **Ciclo SDD** | `trustkit_wave_run` | Despacha tarefas da onda em git worktrees isoladas (Gate 4). |
| **Ciclo SDD** | `trustkit_task_test` | Executa teste fiscalizado de fase TDD (RED ou GREEN) (Gate 4). |
| **Ciclo SDD** | `trustkit_gauntlet_run` | Executa a bateria de auditoria do Gauntlet (Gate 5). |
| **Ciclo SDD** | `trustkit_done` | Gera certificacao do ciclo e relatorio gauntlet-report.md (Gate 6). |
| **Ciclo SDD** | `trustkit_pr` | Abre o Pull Request sob demanda no GitHub. |
| **Memoria & Wiki** | `trustkit_knowledge_search` | Busca lexical FTS5 ranqueada por BM25 na memoria perene. |
| **Memoria & Wiki** | `trustkit_knowledge_read` | Le documento da memoria perene (.trustkit/wiki/<slug>.md). |
| **Memoria & Wiki** | `trustkit_knowledge_write` | Grava atomicamente nota Markdown em .trustkit/wiki/ e indexa no FTS5. |
| **Memoria & Wiki** | `trustkit_knowledge_rebuild` | Reconstroi deterministicamente o indice FTS5 e grafo de wikilinks. |
| **Glossario Ubiquo**| `trustkit_knowledge_glossary_add` | Adiciona termo canonico e sinonimos rejeitados ao glossario perene. |
| **Glossario Ubiquo**| `trustkit_knowledge_glossary_list`| Lista todos os termos do glossario de linguagem ubiqua. |
| **Triagem ADR** | `trustkit_knowledge_adr_check` | Aplica triagem de 3 criterios para criacao de ADR. |
| **Grafo AST** | `trustkit_graph_build` | Constroi ou atualiza o grafo estrutural de dependencias. |
| **Grafo AST** | `trustkit_graph_health` | Diagnostica a saude do grafo e detecta componentes hiperacoplados. |
| **Grafo AST** | `trustkit_graph_path` | Calcula o menor caminho de chamadas entre dois simbolos via BFS. |
| **Grafo AST** | `trustkit_graph_explain` | Explica vizinhanca e dependencias estruturais de um componente. |
| **Terminal & PTY** | `trustkit_terminal_spawn` | Inicia pseudo-terminal (PTY) interativo isolado para execucao. |
| **Terminal & PTY** | `trustkit_terminal_send` | Envia comandos e entradas de texto para o processo no PTY. |
| **Terminal & PTY** | `trustkit_terminal_inspect` | Inspeciona a saida e o estado do buffer de execucao no PTY. |

### 4.6 Auditoria Global Pre-Release (`tk audit security`)

```bash
tk audit security --profile standard --scope . --output-dir .trustkit/audits/security
```

---

## 5. Perguntas Frequentes (FAQ)

### Qual e a diferenca entre o binario trustkit e o alias tk?
Nao ha nenhuma diferenca funcional ou semantica. O comando `tk` e um alias de primeira classe com paridade contratual de 100% com `trustkit`. Ambos compartilham o mesmo banco de dados (`.trustkit/state.db`), maquina de estados e regras. Utilize `tk` para maior agilidade no terminal.

### Por que o comando trustkit close substituiu o antigo trustkit merge?
O comando `trustkit close` (ou `tk close`) implementa o use case formal `close_cycle`, que valida a certificacao previa do Gauntlet (rejeitando fechamento de ciclos nao certificados), atualiza o estado persistido no SQLite WAL e sincroniza automaticamente o card no GitHub Projects para `ReadyForTest`. O antigo comando `merge` foi descontinuado.

### Por que recebi o erro "working tree has uncommitted changes (RN-001)" no task complete?
A regra RN-001 exige integridade total da arvore de trabalho antes da homologacao de qualquer tarefa. Voce nao pode executar `task complete` com arquivos modificados ou novos nao commitados dentro da worktree (pastas temporarias como `target/` e `node_modules/` sao ignoradas). Realize o commit das alteracoes antes de homologar a tarefa.

### Como funciona a revisao em dois eixos (Dual-Axis Review) no Gate 4?
Durante a fase REFACTOR (`task review`), o sistema analisa simultaneamente:
- **Eixo 1 (Spec Compliance):** avalia se a tarefa implementou estritamente o que foi contratado ou se introduziu desvios de escopo (Scope Creep);
- **Eixo 2 (Fowler Smells via AST):** o Tree-Sitter audita o diff procurando code smells lexicais (Long Function, God Structs, Primitive Obsession, etc.), permitindo overrides especificos configurados pelo repositorio.

### Como os agentes de IA consom o array next_steps e os humanos usam o rodape NEXT STEP:?
No terminal TTY, o TRustKit renderiza o rodape legivel `NEXT STEP:` com os comandos priorizados (`[PRIMARY]`, `[SECONDARY]`). No modo `--json`, inclui o array estruturado `next_steps` livre de campos nulos. Isso guia agentes autonomos deterministamente sem probabilidade de LLMs.

---

## 6. Guia de Solucao de Problemas (Troubleshooting)

### 6.1 Tabela de Codigos de Saida Semanticos (POSIX)

| Codigo | Identificador | Causa Frequente | Acao Recomendada |
| :---: | :--- | :--- | :--- |
| `0` | `CleanPass` | Operacao concluida com aprovacao. | Avancar para o proximo comando indicado no `NEXT STEP:`. |
| `1` | `InfrastructureBoundaryFault` | Erro de I/O em disco, SQLite WAL, Git ou disjuntor de tarefas vazias (`EmptyTaskBacklogFault`). | Verificar permissoes de escrita, espaco em disco ou revisar itens na especificacao. |
| `2` | `SpecOrTraceabilityIncomplete` | Spec incompleta, lacuna na matriz ou violacao de glossario ubiquo (`SPEC-008`). | Executar `tk spec lint` e cobrir regras orfas ou ajustar termos rejeitados. |
| `3` | `TddLifecycleViolation` | Tentativa de GREEN sem RED comprovado, filtro vazio (`ZeroFilteredTestsFault`), erro de compilacao na fase RED (`CompilationFailureInRedPhaseFault`) ou arvore suja (`RN-001`). | Garantir teste compilavel que falhe por assercao de dominio e comitar alteracoes antes de `task complete`. |
| `4` | `AstIntegrityOrInvariantViolation` | Presenca de emojis, trailers de IA no commit ou alteracao indevida de testes protegidos. | Executar `tk gauntlet ast-audit` para identificar e eliminar os grafismos ou trailers. |
| `5` | `CriticSwarmOrPlaywrightFailed` | Falha de compilacao no swarm, linter de codigo ou quebra de cenarios Playwright E2E. | Inspecionar relatorio de testes e corrigir as falhas funcionais apontadas. |
| `6` | `CircuitBreakerTripped` | Disjuntor disparado apos 5 falhas consecutivas ou ativacao do Merge Danger Door. | Executar `tk doctor` para diagnosticar a causa-raiz e seguir os passos de recuperacao. |

### 6.2 Mensagens Comuns de Bloqueio e Como Resolver

#### Bloqueio 1: `MissingRedPhaseProof`
- **Causa:** Tentativa de registrar `task test --phase green` sem a fase RED previa no SQLite WAL.
- **Solucao:** Escreva o teste, execute `tk task test <ID> --phase red` confirmando a falha legitima e depois implemente o codigo.

#### Bloqueio 2: `InvariantViolation: EmojiDetected`
- **Causa:** Presenca de icones de emoji em arquivos modificados ou mensagens de commit.
- **Solucao:** Remova todos os caracteres graficos e substitua-os por texto alfanumerico (ex: `[PASS]`, `[FAIL]`).

#### Bloqueio 3: `TraceabilityGap: OrphanRule`
- **Causa:** Regra de Negocio na spec sem correspondencia com Tarefa ou Teste E2E na matriz.
- **Solucao:** Execute `tk spec lint`, identifique a regra orfa e complete o mapeamento antes de `tk spec lock`.

#### Bloqueio 4: `working tree has uncommitted changes (RN-001)`
- **Causa:** Tentativa de executar `task complete` com alteracoes de arquivos pendentes na worktree.
- **Solucao:** Execute `git commit -am "<mensagem>"` no diretorio da tarefa antes de rodar `tk task complete <ID>`.

### 6.3 Diagnostico e Recuperacao com o Assistente Doutor (`tk doctor`)

Caso o ciclo seja interrompido, o banco apresente anomalias ou o disjuntor de seguranca seja disparado, invoque o diagnosticador:

```bash
tk doctor
```

O comando inspeciona a conexao com `state.db`, a saude dos worktrees ativos, conflitos potenciais na branch alvo (Merge Danger Door), o status do cache local de compilacao (`sccache` via `RUSTC_WRAPPER=sccache` ou fallback para `shared_target_dir` em `.trustkit/cache/target` segundo `ToolCachingPolicy` e `BatchExecutionConfig` na secao `[harness.execution]` do `trustkit.toml`, com `compilation_cache` e `batch_grouping_mode`) e fornece o roteiro de desbloqueio e instalacao (`cargo install sccache`).

---

## 7. Glossario de Termos e Linguagem Ubiqua

| Termo Ubiquo | Definicao no TRustKit |
| :--- | :--- |
| **tk** | Alias de linha de comando oficial de primeira classe com 100% de paridade contratual com o binario `trustkit`. |
| **Dual-Audience Presenter** | Padrao de apresentacao que emite blocos legiveis com `NEXT STEP:` para operadores humanos e JSON estritamente livre de nulos com `next_steps` para agentes. |
| **Dual-Axis Review** | Revisao de codigo em dois eixos simultaneos: Eixo 1 (Scope Creep & Spec Compliance) e Eixo 2 (Fowler Smells via AST Tree-Sitter). |
| **Expand-Contract Topology** | Padrao de planejamento de ondas topologicas que expande modelos e entidades em ondas iniciais antes de migrar os pontos de consumo. |
| **Ubiquitous Glossary** | Vocabulario controlado de termos canonicos e sinonimos rejeitados registrado em `.trustkit/wiki/glossary.md` e auditado via regra `SPEC-008`. |
| **ADR Triage (adr-check)** | Filtro objetivo de 3 criterios para avaliar a necessidade real de criacao de um Architectural Decision Record. |
| **SddCycle** | Instancia de execucao isolada e auditavel correspondente a entrega de uma funcionalidade. |
| **GateStage** | Cada um dos 7 marcos deterministas que governam a progressao do ciclo de desenvolvimento. |
| **ArtifactSeal** | Selo criptografico BLAKE3 que assegura a integridade imutavel da especificacao no Gate 2. |
| **TraceabilityLink** | Amarra verificavel entre Regra de Negocio, Requisito Funcional, Tarefa, Teste E2E e Screenshot. |
| **AtomicTask** | Unidade minima de implementacao restrita a uma camada arquitetural e sujeita ao ciclo TDD. |
| **ExecutionWave** | Conjunto de tarefas independentes entre si despachadas simultaneamente em Git Worktrees. |
| **CircuitBreaker** | Disjuntor de seguranca acionado apos a 5ª reprovacao consecutiva na esteira do Gauntlet. |
| **Gauntlet Loop V2** | Bateria de verificacao adversarial multi-agente que audita integridade estatica, TDD, Playwright E2E e seguranca antes do fechamento. |
