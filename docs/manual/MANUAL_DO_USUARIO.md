# Manual de Uso do TRustKit

**Versão da Documentação:** 1.0.0  
**Motor:** `trustkit` (Rust Nativo — 6 Crates)  
**Ambiente:** Spec-Driven Development (SDD), Isolamento em Git Worktrees e Gauntlet Loop V2  
**Data:** Novembro de 2026  

---

## Sumario

1. [Visao Geral & Boas-Vindas](#1-visao-geral--boas-vindas)
   - [1.1 O que e o TRustKit?](#11-o-que-e-o-trustkit)
   - [1.2 Para quem se destina este manual?](#12-para-quem-se-destina-este-manual)
   - [1.3 O Paradigma Hybrid Rust Core + Thin Skill](#13-o-paradigma-hybrid-rust-core--thin-skill)
   - [1.4 Principios Constitucionais de Governanca](#14-principios-constitucionais-de-governanca)
2. [Primeiros Passos](#2-primeiros-passos)
   - [2.1 Requisitos do Sistema](#21-requisitos-do-sistema)
   - [2.2 Verificando a Instalacao e o Binario](#22-verificando-a-instalacao-e-o-binario)
   - [2.3 Modos de Operacao: Humano vs Agente](#23-modos-de-operacao-humano-vs-agente)
   - [2.4 Diagnosticando a Sessao Atual](#24-diagnosticando-a-sessao-atual)
   - [2.5 Guia de Adocao: Novo Projeto vs Projeto em Andamento](#25-guia-de-adocao-novo-projeto-vs-projeto-em-andamento)
3. [Guia Operacional Passo a Passo (A Jornada dos 7 Gates)](#3-guia-operacional-passo-a-passo-a-jornada-dos-7-gates)
   - [3.1 Tarefa 1: Iniciar um Ciclo de Desenvolvimento (Gate 0)](#31-tarefa-1-iniciar-um-ciclo-de-desenvolvimento-gate-0)
   - [3.2 Tarefa 2: Formular, Validar e Congelar a Especificacao (Gates 1 & 2)](#32-tarefa-2-formular-validar-e-congelar-a-especificacao-gates-1--2)
   - [3.3 Tarefa 3: Decompor Tarefas e Planejar Ondas em DAG (Gate 3)](#33-tarefa-3-decompor-tarefas-e-planejar-ondas-em-dag-gate-3)
   - [3.4 Tarefa 4: Execucao Paralela, Worktrees e Ciclo TDD Estrito (Gate 4)](#34-tarefa-4-execucao-paralela-worktrees-e-ciclo-tdd-estrito-gate-4)
   - [3.5 Tarefa 5: Submeter ao Gauntlet Loop e Revisao de Seguranca (Gate 5)](#35-tarefa-5-submeter-ao-gauntlet-loop-e-revisao-de-seguranca-gate-5)
   - [3.6 Tarefa 6: Certificacao, Merge e Abertura de Pull Request (Gate 6)](#36-tarefa-6-certificacao-merge-e-abertura-de-pull-request-gate-6)
4. [Recursos Avancados e Modulos Especializados](#4-recursos-avancados-e-modulos-especializados)
   - [4.1 Memoria Perene do Projeto (`trustkit memory`)](#41-memoria-perene-do-projeto-trustkit-memory)
   - [4.2 Inteligencia de Grafo Estrutural (`trustkit graph`)](#42-inteligencia-de-grafo-estrutural-trustkit-graph)
   - [4.3 Fatiamento Cirurgico de Contexto (`trustkit context`)](#43-fatiamento-cirurgico-de-contexto-trustkit-context)
   - [4.4 Protecao Proativa com Fast Hooks (`trustkit hook`)](#44-protecao-proativa-com-fast-hooks-trustkit-hook)
   - [4.5 Servidor Nativo MCP (`trustkit mcp`)](#45-servidor-nativo-mcp-trustkit-mcp)
   - [4.6 Auditoria Global Pre-Release (`trustkit audit security`)](#46-auditoria-global-pre-release-trustkit-audit-security)
5. [Perguntas Frequentes (FAQ)](#5-perguntas-frequentes-faq)
6. [Guia de Solucao de Problemas (Troubleshooting)](#6-guia-de-solucao-de-problemas-troubleshooting)
   - [6.1 Tabela de Codigos de Saida Semanticos (POSIX)](#61-tabela-de-codigos-de-saida-semanticos-posix)
   - [6.2 Mensagens Comuns de Bloqueio e Como Resolver](#62-mensagens-comuns-de-bloqueio-e-como-resolver)
   - [6.3 Recuperacao de Disjuntor Acionado (Circuit Breaker)](#63-recuperacao-de-disjuntor-acionado-circuit-breaker)
7. [Glossario de Termos e Linguagem Ubiqua](#7-glossario-de-termos-e-linguagem-ubiqua)

---

## 1. Visao Geral & Boas-Vindas

### 1.1 O que e o TRustKit?

O **TRustKit** e um motor de governanca deterministica e execucao de alta velocidade desenvolvido em Rust nativo. Ele foi projetado especificamente para gerenciar fluxos de **Spec-Driven Development (SDD)** em projetos de software modernos, onde atuam equipes multidisciplinares e agentes autonomos de inteligencia artificial (Claude Code, Antigravity, Cursor, Codex).

Diferente de abordagens baseadas exclusivamente em instrucoes textuais que podem sofrer com desvios ou alucinacoes, o TRustKit delega as decisoes criticas a um binario compilado (`trustkit`). Esse binario valida sintaxes em microssegundos, fiscaliza a integridade dos testes de ponta a ponta, impede alteracoes indevidas em regras de negocio congeladas e sincroniza automaticamente o estado das entregas em um banco transacional auditavel (SQLite WAL).

### 1.2 Para quem se destina este manual?

Este manual serve como guia pratico para quatro papeis fundamentais:

* **Desenvolvedores de Software e Arquitetos:** Que precisam planejar, codificar e entregar features complexas garantindo que cada teste e regra de negocio permaneca integro.
* **Operadores e Agentes Agênticos:** Que executam comandos de terminal de forma autonoma e demandam respostas sem ambiguidades, com baixo consumo de memoria e sem campos nulos (`--json`).
* **Lideres Tecnicos (Tech Leaders):** Que precisam auditar o cumprimento dos requisitos antes de autorizar a integracao de codigo no repositorio principal.
* **Analistas de Qualidade (QA) e Homologadores:** Que buscam rastreabilidade matematica entre a regra de negocio acordada, os testes executados e as capturas de tela geradas.

### 1.3 O Paradigma Hybrid Rust Core + Thin Skill

Para aliar a flexibilidade do raciocinio humano e agêntico a solidez de regras imutaveis, o TRustKit adota uma arquitetura em duas camadas:

1. **Thin Skill Declarativa (`SKILL.md`):** Camada leve que reside no contexto do desenvolvedor ou agente, fornecendo orientacoes de alto nivel e fluxos conceituais sem sobrecarregar a janela de contexto.
2. **Native Rust Core (`trustkit`):** Binario estatico compilado em Rust que executa 100% das mutacoes de estado, validacoes sintaticas de AST, congelamento criptografico BLAKE3, criacao de Git Worktrees e verificacao estrita de TDD.

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
|                   Camada 2: Native Rust Core (trustkit)             |
|   - Persistencia Transacional SQLite WAL (.trustkit/state.db)      |
|   - Validador AST Tree-Sitter (< 3ms) e Anti-Cheat de Testes       |
|   - Isolamento Paralelo de Tarefas em Git Worktrees (CoW)          |
|   - Selo Criptografico BLAKE3 para Especificacoes Congeladas       |
|   - Servidor MCP Nativo, Grafo de Conhecimento e Memoria Wiki FTS5 |
+--------------------------------------------------------------------+
```

### 1.4 Principios Constitucionais de Governanca

Todas as rotinas operadas pelo TRustKit sao regidas por regras fundamentais de qualidade e conformidade:

* **Invariante #1: Sem Codigo Sem Spec Congelada.** Nenhum arquivo de producao pode ser comitado ou implementado sem que a especificacao funcional esteja validada e selada com hash criptografico.
* **Invariante #2: Evidencia sobre Assercao.** Nenhuma etapa de teste ou compilacao e aprovada por declaracao verbal. O binario captura a saida literal, a duracao em milissegundos e o codigo de retorno POSIX.
* **Invariante #4: Integridade Imutavel de Testes (Anti-Cheat).** E proibido remover, comentar ou enfraquecer assercoes de testes existentes para forcar aprovacoes.
* **Invariante #5: Disjuntor de Seguranca (Circuit Breaker).** Se uma etapa de verificacao falhar por 5 vezes consecutivas, o sistema bloqueia novos ciclos e exige intervencao humana orientada.
* **Invariante #6: GitFlow Rigoroso.** Ramificacoes baseadas em `develop -> staging -> main`. Bloqueio de comandos destrutivos (`git reset --hard`, `git stash`, `git clean -fd`).
* **Invariante #7: Proibicao Absoluta de Emojis.** Toda comunicacao, log, arquivo de codigo e mensagem de commit deve empregar exclusivamente caracteres alfanumericos e marcadores textuais claros (ex: `[PASS]`, `[FAIL]`).
* **Invariante #12: Pull Request Sob Demanda.** Pull Requests nunca sao abertos automaticamente; dependem de solicitacao expressa do desenvolvedor.
* **Invariante #13: Proibicao de Trailers de IA.** Vetado o uso de marcas como `Co-Authored-By: Claude` ou mensagens geradas automaticamente em commits e documentos.

---

## 2. Primeiros Passos

### 2.1 Requisitos do Sistema

Antes de iniciar as atividades, confirme se as seguintes ferramentas estao presentes no ambiente operacional:

* **Sistema Operacional:** Linux (x86_64 ou AArch64) ou macOS (Darwin).
* **Compilador e Ferramental Rust:** Versao 1.82 ou superior com Cargo instalado.
* **Git:** Versao 2.38 ou superior (com suporte ativo a worktrees e sparse-checkout).
* **SQLite:** Versao 3.35 ou superior com suporte a modo WAL.
* **GitHub CLI (`gh`):** Autenticado na conta corporativa com acesso de leitura e escrita ao repositorio.

### 2.2 Verificando a Instalacao e o Binario

Para compilar o binario de producao do TRustKit no workspace atual:

```bash
cargo build --release --workspace
```

O binario executavel ficara disponivel em `target/release/trustkit`. Para facilitar o acesso global, recomenda-se adiciona-lo ao seu `PATH` ou criar um atalho simbolico:

```bash
sudo ln -sf "$(pwd)/target/release/trustkit" /usr/local/bin/trustkit
```

Execute o comando de versao para validar a presenca:

```bash
trustkit --version
```

Saida esperada:
```text
trustkit 0.1.0
```

Para consultar a lista completa de subcomandos disponiveis:

```bash
trustkit --help
```

### 2.3 Modos de Operacao: Humano vs Agente

O TRustKit oferece flexibilidade visual completa para dois tipos de publico:

1. **Modo Humano (Padrao via Terminal TTY):**  
   Exibe informacoes limpas em formato de tabela ASCII legivel, usando prefixos padronizados como `[GATE-0]`, `[PASS]`, `[FAIL]`, `[WARN]` e `[INFO]`, sem decoracoes superfluas.

2. **Modo Agente (`--json` ou variavel `TRUSTKIT_AGENT_MODE=1`):**  
   Emite a saida como um documento JSON minificado e deterministico.  
   * **Garantia de ausencia de valores nulos:** Propriedades opcionais sao expressas como listas vazias `[]` ou objetos de caso especial explicitos, evitando quebras em parsers agênticos.

Exemplo de chamada em modo agente:
```bash
trustkit status --json
```

### 2.4 Diagnosticando a Sessao Atual

Para verificar se ha um ciclo SDD ativo no diretorio de trabalho e qual o estagio dos gates:

```bash
trustkit status
```

Exemplo de retorno de status no terminal:
```text
[STATUS] Ciclo Atual: issue-45-202611041030
[STATUS] Gate Atual: GATE_1_SPECIFICATION
[STATUS] Estado da Spec: DRAFT_UNVERIFIED
[STATUS] Invariantes: 16 de 16 Ativas
[STATUS] Banco SQLite WAL: .trustkit/state.db (Ok)
```

### 2.5 Guia de Adocao: Novo Projeto vs Projeto em Andamento

O TRustKit foi projetado para eliminar atritos operacionais de configuracao manual. O proprio comando `trustkit init` encarrega-se de inspecionar o ambiente, gerar ou atualizar o `.gitignore`, criar o template de governanca `trustkit.toml`, inicializar o banco transacional SQLite WAL e criar as pastas necessarias.

Abaixo estao as diretrizes simplificadas para projetos novos (greenfield) e em andamento (brownfield):

#### Cenario A: Inicializando o TRustKit em um Projeto Novo (Greenfield / Sem Codigo)

Quando voce esta iniciando um novo repositorio sem nenhum codigo pre-existente:

1. **Boa Pratica Previa de Repositorio (Git e Remote):**
   Antes de invocar qualquer ferramenta, a boa engenharia de sistemas recomenda configurar o repositorio Git local com sua branch base e apontar para o remote de origem:
   ```bash
   mkdir meu-novo-sistema && cd meu-novo-sistema
   git init -b develop
   git remote add origin git@github.com:organizacao/meu-novo-sistema.git
   git commit --allow-empty -m "chore: initial repository bootstrap"
   ```
   *(Nota: O motor de inspecao `GitInspector` exige a presenca de um commit raiz com hash `HEAD` valido).*

2. **Comando Unico de Inicializacao e Scaffolding (`trustkit init`):**
   Com o repositorio Git pronto, basta executar:
   ```bash
   # Vinculado a uma issue do GitHub:
   trustkit init --issue 1

   # Ou para ciclo puramente local (desconectado):
   trustkit init
   ```

   **O que o `trustkit init` faz de forma 100% automatica:**
   * **Gerencia o `.gitignore`:** Cria o arquivo `.gitignore` com as regras necessarias (`.trustkit/state.db*`, `.trustkit/worktrees/`, `.trustkit/audits/` e a preservacao da base viva `!.trustkit/wiki/`). Caso o arquivo ja exista, o TRustKit anexa essas regras sem sobrescrever o conteudo anterior.
   * **Gera a politica de governanca (`trustkit.toml`):** Cria o template padrao de configuracao do Gauntlet (`security_review_mode = "prompt"`, `min_confidence_threshold = 8`) caso ele nao exista.
   * **Provisiona a infraestrutura:** Cria o diretorio `.trustkit/`, inicializa o banco SQLite em modo WAL (`.trustkit/state.db`) aplicando todas as migrations e cria a pasta `docs/specs/<ciclo>/`.

3. **Ativar os Fast Hooks no assistente agêntico (Claude Code, Cursor ou Antigravity):**
   Configure o interceptador `PreToolUse` do seu assistente para chamar:
   ```bash
   trustkit hook pre-tool --command "$COMMAND"
   ```

---

#### Cenario B: Implantando o TRustKit em um Projeto em Andamento (Brownfield / Existente)

Para projetos legados ou em producao (em Rust, TypeScript, JavaScript, Python, etc.) que ja possuem base de codigo:

1. **Verificar a arvore de trabalho Git:**
   * O TRustKit tolera modificacoes locais sem comitar: ele as cataloga como `pre_existing_dirty_files` e as protege sem executar `git stash` destrutivo.
   * Apenas assegure-se de estar em uma branch ativa (evite estado de *detached HEAD*).

2. **Executar a inicializacao no projeto (`Gate 0`):**
   ```bash
   trustkit init --issue 45
   # Ou aproveitando um PRD funcional legado ja existente:
   trustkit init --issue 45 --from-prd docs/PRD_Legado.md
   ```
   > [!IMPORTANT]
   > **Edicao Automatica do `.gitignore` Existente:**  
   > Se o seu projeto brownfield ja possuir um `.gitignore`, o `trustkit init` ira inspeciona-lo e, caso as entradas do TRustKit ainda nao estejam presentes, **anexara o bloco de regras ao final do arquivo**, preservando integralmente todas as exclusoes anteriores (`node_modules/`, `target/`, `.env`, etc.). O mesmo se aplica ao `trustkit.toml`, que so e criado se estiver ausente.

3. **Gerar o Grafo Estrutural Imediato do Legado:**
   Execute a extracao sintatica via Tree-Sitter sobre todo o codigo existente para indexar dependencias e diagnosticar a arquitetura atual:
   ```bash
   # Mapeia componentes e emite .trustkit/wiki/graph/GRAPH_REPORT.md
   trustkit graph build

   # Identifica os maiores God Nodes (componentes hiperacoplados legados)
   trustkit graph health --top-god-nodes 10
   ```

4. **Indexar a Memoria Perene do Sistema:**
   Registre notas fundamentais sobre a arquitetura legada em `.trustkit/wiki/`:
   ```bash
   trustkit memory write --slug "arquitetura-legada" --title "Visao da Arquitetura Legada" --tags "overview,brownfield" --body "Visao dos servicos e modulos existentes..."
   trustkit memory rebuild
   ```

5. **Habilitar o Servidor MCP Nativo:**
   Inicie o servidor MCP (`trustkit mcp`) para que seus assistentes de IDE (Claude Code, Cursor, Antigravity) consumam as ferramentas deterministas diretamente na sessao de trabalho.

---

## 3. Guia Operacional Passo a Passo (A Jornada dos 7 Gates)

O desenvolvimento de qualquer funcionalidade no TRustKit atravessa uma sequencia de 7 estagios deterministas conhecidos como **Os 7 Gates do SDD**. Abaixo, detalhamos como executar cada etapa no seu dia a dia.

```
  +---------+     +---------+     +---------+     +---------+
  | Gate 0  | --> | Gate 1  | --> | Gate 2  | --> | Gate 3  |
  | Setup & |     | Spec    |     | Selo    |     | Ondas   |
  | Context |     | V2      |     | BLAKE3  |     | DAG     |
  +---------+     +---------+     +---------+     +---------+
                                                       |
  +---------+     +---------+     +---------+          |
  | Gate 6  | <-- | Gate 5  | <-- | Gate 4  | <--------+
  | Merge & |     | Gauntlet|     | TDD em  |
  | PR      |     | Loop V2 |     | Worktree|
  +---------+     +---------+     +---------+
```

---

### 3.1 Tarefa 1: Iniciar um Ciclo de Desenvolvimento (Gate 0)

O Gate 0 valida o ambiente de trabalho, garante que o desenvolvedor esteja operando na conta correta do GitHub e inicializa a pasta de especificacoes isolada no banco de dados.

#### Passo 1: Executar o comando de inicializacao

Caso voce possua uma demanda oficial (Issue #45) no quadro de projetos:

```bash
trustkit init --issue 45
```

Se preferir derivar a especificacao diretamente a partir de um documento funcional ou PRD existente:

```bash
trustkit init --issue 45 --from-prd docs/PRD_TRustKit.md
```

Se for um ciclo puramente experimental ou local sem issue registrada:

```bash
trustkit init
```

#### O que esperar na tela:

```text
[GATE-0] Inicializando ciclo SDD...
[PASS] Identidade GitHub validada com sucesso.
[PASS] Arvore Git inspecionada. Arquivos pre-existentes protegidos.
[PASS] Diretorio criado: docs/specs/issue-45-202611041030/
[PASS] Card #45 transicionado para 'In progress' no GitHub Projects.
[PASS] Sessao registrada no SQLite WAL: .trustkit/state.db
[INFO] Ciclo ativo: issue-45-202611041030
```

---

### 3.2 Tarefa 2: Formular, Validar e Congelar a Especificacao (Gates 1 & 2)

Nesta etapa, o objetivo e documentar a regra de negocio com precisao cirurgica antes de escrever qualquer linha de codigo de producao.

#### Passo 1: Gerar o esqueleto da especificacao

Para que o TRustKit monte uma estrutura inicial enriquecida com informacoes do grafo do projeto e da memoria viva:

```bash
trustkit spec prompt --idea "Autenticacao multifator via TOTP com recovery codes" --budget 4000
```

Esse comando cria o arquivo de especificacao em `docs/specs/<ciclo>/spec.md` contendo:
- Product Trio (Contexto, Proposta de Valor e Regras de Negocio - RN)
- Requisitos Funcionais (RF) com criterios de aceite em formato Gherkin (`Dado / Quando / Entao`)
- Fluxograma de jornada em sintaxe Mermaid (`flowchart LR`)
- Matriz de Rastreabilidade Ponta a Ponta ($\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$)

#### Passo 2: Validar a sintaxe da especificacao (Linter Pre-Lock)

Antes de tentar selar o documento, passe pelo linter estatico em memoria:

```bash
trustkit spec lint
```

O linter executara em menos de 3 milissegundos e apontara se ha:
- Blocos Mermaid com sintaxe invalida.
- Regras de negocio orfas (uma regra sem tarefa ou teste correspondente na matriz).
- Presenca indevida de emojis.

#### Passo 3: Congelar a especificacao com o Selo Criptografico BLAKE3 (Gate 2)

Assim que a especificacao estiver revisada e aprovada:

```bash
trustkit spec lock
```

#### O que esperar na tela:

```text
[GATE-2] Validando fechamento da Matriz de Rastreabilidade...
[PASS] 6 Regras de Negocio (RN) cobertas.
[PASS] Diagrama Mermaid validado.
[PASS] Criterios Gherkin conformes.
[PASS] Digest BLAKE3 gerado: a7f8e3...9b2c
[PASS] Especificacao CONGELADA com sucesso. Transicao para GATE_2_LOCKED.
```

> [!IMPORTANT]
> Uma vez congelada, a especificacao nao deve ser editada manualmente. Qualquer mutacao em seu conteudo provocara rejeicao imediata nos gates subsequentes, a menos que seja intencionalmente liberada com comando proprio.

---

### 3.3 Tarefa 3: Decompor Tarefas e Planejar Ondas em DAG (Gate 3)

Com a regra selada, decompomos a entrega em tarefas atomicas de implementacao salvas no diretorio `tasks/` (ex: `tasks/TASK-001.md`, `tasks/TASK-002.md`).

#### Passo 1: Gerar o plano topologico de execucao

Execute o planejador de ondas:

```bash
trustkit wave plan
```

O TRustKit le todas as tarefas atomicas, extrai suas dependencias declaradas, analisa possiveis acoplamentos de arquivos e monta um Grafo Aciclico Direcionado (DAG).

#### O que esperar na tela:

```text
[GATE-3] Compilando Grafo de Tarefas (DAG)...
[PASS] 4 tarefas atomicas encontradas.
[PASS] Nenhuma dependencia circular detectada.
[INFO] Cronograma de Ondas Planejado:
  - Onda 1: TASK-001 (Dominio Core), TASK-002 (Adaptador de Banco)
  - Onda 2: TASK-003 (Servico de Autenticacao)
  - Onda 3: TASK-004 (Endpoint CLI / Web)
[PASS] Estado salvo no banco de dados. Transicao para GATE_3_WAVE_PLANNED.
```

---

### 3.4 Tarefa 4: Execucao Paralela, Worktrees e Ciclo TDD Estrito (Gate 4)

No Gate 4, o codigo ganha vida. O TRustKit isola cada tarefa em uma arvore de trabalho propria (Git Worktree efemera) e fiscaliza mecanicamente o ciclo de Test-Driven Development (TDD).

#### Passo 1: Despachar a onda de trabalho

Para inicializar as arvores isoladas da primeira onda:

```bash
trustkit wave run --wave 1
```

O sistema provisiona o diretorio `.trustkit/worktrees/task-001` sem interferir na arvore principal do repositorio.

#### Passo 2: Registrar a Fase RED (Comprovacao de Falha)

Antes de implementar a solucao funcional, o teste unitario ou de integracao correspondente deve ser escrito e executado para provar que ele falha:

```bash
trustkit task test TASK-001 --phase red
```

O TRustKit executa o comando de teste, confirma que o resultado retornou erro (`Exit Code != 0`), sanitiza a saida e registra o hash da falha no banco de dados.

#### Passo 3: Implementar a solucao e registrar a Fase GREEN (Sucesso Comprovado)

Escreva o codigo de producao ate que o teste passe integralmente. Em seguida, registre o sucesso:

```bash
trustkit task test TASK-001 --phase green
```

O TRustKit confirma que o teste agora passa (`Exit Code == 0`). Se voce tentar executar este passo sem ter registrado a fase RED antes, o comando sera sumariamente recusado.

#### Passo 4: Realizar a Revisao Semantica Multi-Angulo (Fase REFACTOR)

Durante a fase de refatoracao, voce pode submeter as alteracoes a uma auditoria semantica automatizada para garantir conformidade de tipagem e arquitetura:

```bash
trustkit task review TASK-001 --effort medium
```

Parametros aceitos para `--effort`: `low`, `medium`, `high`, `max`.

#### Passo 5: Concluir e Homologar a Tarefa

Apos concluir os testes e a revisao, finalize a tarefa atomica:

```bash
trustkit task complete TASK-001
```

O TRustKit consolida o commit da tarefa no formato oficial, encerra a worktree efemera e incorpora as mudancas na branch de trabalho da funcionalidade (`marcos/<id>/<slug>`).

---

### 3.5 Tarefa 5: Submeter ao Gauntlet Loop e Revisao de Seguranca (Gate 5)

O Gauntlet e a esteira de verificacao mais rigorosa do sistema. Ele impede que defeitos, vulnerabilidades ou quebras de regras atinjam os ramos estaveis do projeto.

#### Passo 1: Executar a Auditoria Sintatica Instantanea (Camada 0)

A Camada 0 inspeciona a AST do codigo em menos de 5 milissegundos via `tree-sitter`:

```bash
trustkit gauntlet ast-audit
```

Esta camada bloqueia imediatamente:
- Tentativas de comentar ou remover testes pre-existentes (Anti-Cheat).
- Inclusao de emojis em qualquer arquivo modificado.
- Atribuicao de autoria para assistentes de IA (trailers de IA).
- Uso indevido de links `raw.githubusercontent` ou tipos genericos `any`.

#### Passo 2: Disparar o Gauntlet Completo e Testes E2E (Camada 1)

Assim que a Camada 0 aprovar, dispare a rodada completa:

```bash
trustkit gauntlet run
```

Durante a execucao, o sistema podera perguntar no terminal se deseja realizar a Revisao de Seguranca do Diff (`security-review`):

```text
Deseja executar a revisao de seguranca de aplicacao neste diff? [s/N]:
```

Voce tambem pode definir o comportamento explicitamente via linha de comando:

```bash
# Executa forçando a revisao de seguranca:
trustkit gauntlet run --security-review

# Executa dispensando a revisao de seguranca nesta rodada:
trustkit gauntlet run --skip-security-review
```

#### O que esperar na tela:

```text
[GATE-5] Iniciando Gauntlet Loop V2...
[PASS] Camada 0: Auditoria AST Limpa (0 violacoes em 2ms).
[PASS] Camada 1: Compilacao de Release bem-sucedida.
[PASS] Camada 1: Linter corporativo sem avisos.
[PASS] Camada 1: Suite Playwright E2E executada.
[PASS] Capturas de tela gravadas em docs/specs/.../evidence/.
[PASS] Restauradas screenshots alheias intactas.
[PASS] Swarm de Criticos: Aprovacao unanime.
[PASS] Gauntlet CONCLUIDO com sucesso absoluto.
```

---

### 3.6 Tarefa 6: Certificacao, Merge e Abertura de Pull Request (Gate 6)

Com todos os testes, invariantes e suites aprovadas, consolidamos a entrega.

#### Passo 1: Gerar o Relatorio de Certificacao e Comentario da Issue

```bash
trustkit done
```

Esse comando analisa os eventos gravados no SQLite WAL e compila o relatorio final `gauntlet-report.md`. Caso a issue esteja vinculada ao GitHub, o TRustKit publica automaticamente o comentario formal de encerramento contendo a matriz fechada e as evidencias visuais.

#### Passo 2: Executar o Merge Deterministico

Para incorporar as alteracoes a branch de destino homologada (geralmente `develop`):

```bash
trustkit merge --target develop
```

O TRustKit efetua o merge limpo, remove a branch de trabalho local e atualiza a coluna do card no GitHub Projects para `Await Staging`.

#### Passo 3: Abrir o Pull Request (Sob Demanda Humana)

Seguindo a Invariante Constitucional #12, a abertura de PR **nunca e automatica**. Quando o desenvolvedor desejar submeter o trabalho para revisao dos pares:

```bash
trustkit pr --target develop
```

O comando configura automaticamente os revisores oficiais, os links de rastreabilidade e move o card do quadro para a coluna `In review`.

---

## 4. Recursos Avancados e Modulos Especializados

Alem do fluxo linear dos 7 Gates, o TRustKit conta com subsistemas nativos que potencializam a gestao tecnica do repositorio.

### 4.1 Memoria Perene do Projeto (`trustkit memory`)

O TRustKit mantem uma base viva de conhecimento em arquivos Markdown com metadados estruturados sob o diretorio `.trustkit/wiki/`. Esses arquivos sao indexados em SQLite FTS5 com algoritmo de ranqueamento BM25 e suportam ligacoes bidirecionais via wikilinks (`[[slug]]`).

#### Buscar conhecimento na wiki:
```bash
trustkit memory search --query "politica de invariantes" --limit 5
```

#### Ler uma nota especifica:
```bash
trustkit memory read --slug "architecture-decision-001"
```

#### Criar ou atualizar uma nota de conhecimento:
```bash
trustkit memory write --slug "padroes-de-seguranca" --title "Padroes de Seguranca" --tags "auth,crypto" --body "Texto explicativo..."
```

#### Listar todas as notas cadastradas:
```bash
trustkit memory list
```

#### Reconstruir o indice de busca a partir dos arquivos em disco:
```bash
trustkit memory rebuild
```

---

### 4.2 Inteligencia de Grafo Estrutural (`trustkit graph`)

O motor de grafo analisa o codigo-fonte (Rust, TypeScript, JavaScript) via Tree-Sitter e extrai as dependencias entre modulos, funcoes, structs, imports e notas de documentacao.

#### Construir ou atualizar o mapa estrutural:
Gera o documento analitico `.trustkit/wiki/graph/GRAPH_REPORT.md`:
```bash
trustkit graph build
```

#### Auditar a saude do grafo e detectar God Nodes:
Identifica classes ou funcoes com acoplamento excessivo que representam risco a arquitetura:
```bash
trustkit graph health --top-god-nodes 5
```

#### Encontrar o menor caminho entre dois simbolos (Busca BFS):
Descobre como duas entidades de codigo se comunicam atraves das chamadas internas:
```bash
trustkit graph path --source "crates/trustkit-core::SddCycle" --target "crates/trustkit-db::SqliteWalRepositoryAdapter"
```

#### Explicar a vizinhanca de um componente:
Exibe quem importa, quem chama e quais notas da wiki mencionam determinado no:
```bash
trustkit graph explain --node "crates/trustkit-core/src/entities.rs::SddCycle"
```

---

### 4.3 Fatiamento Cirurgico de Contexto (`trustkit context`)

Ao interagir com modelos de linguagem, enviar arquivos inteiros consome orcamento de tokens e polui a atencao do modelo. O fatiador extrai cirurgicamente apenas as partes relevantes do codigo:

```bash
# Fatiamento classico de arquivo respeitando limite de linhas/tokens:
trustkit context slice --file crates/trustkit-core/src/lib.rs --budget 500

# Fatiamento multi-hop guiado pelo grafo estrutural (saltando de dependencia em dependencia):
trustkit context slice --file crates/trustkit-core/src/entities.rs --hops 2 --strategy bfs --budget 800
```

---

### 4.4 Protecao Proativa com Fast Hooks (`trustkit hook`)

O TRustKit intercepta chamadas de shell em menos de 3 milissegundos para impedir que agentes ou scripts executem operacoes que comprometam a integridade do repositorio:

```bash
trustkit hook pre-tool --command "git add ."
```

#### Operacoes bloqueadas na hora:
* `git stash` (risco de perder alteracoes sem registro)
* `git add .` ou `git add -A` (risco de comitar artefatos indevidos ou segredos)
* `git reset --hard` ou `git clean -fd`
* Pushes diretos para branches protegidas (`develop`, `staging`, `main`)
* Commits que contenham emojis ou referencias como `Co-Authored-By: Claude`

---

### 4.5 Servidor Nativo MCP (`trustkit mcp`)

Para desenvolvedores que utilizam assistentes integrados em editores (Claude Code, Cursor, Antigravity), o TRustKit expoe suas funcoes nativamente atraves do protocolo **Model Context Protocol (MCP)** sobre `stdio`:

```bash
trustkit mcp
```

#### Principais Ferramentas MCP Expostas:
| Ferramenta MCP | Finalidade Operacional |
| :--- | :--- |
| `trustkit_spec_template` | Gera o template padrao de especificacao com regras constitucionais |
| `trustkit_spec_context` | Obtem contexto enriquecido para elaboracao de novas specs |
| `trustkit_spec_lint` | Executa a verificacao rapida em memoria da especificacao |
| `trustkit_cycle_status` | Consulta o gate atual e o diagnostico da sessao ativa |
| `trustkit_context_slice` | Executa o corte inteligente de arquivos para nutrir a conversa |
| `trustkit_memory_search` | Realiza buscas textuais na base de conhecimento perene |
| `trustkit_graph_query` | Consulta relacoes e componentes do grafo de arquitetura |
| `trustkit_graph_health` | Verifica a saude do sistema e aponta componentes hiperacoplados |

---

### 4.6 Auditoria Global Pre-Release (`trustkit audit security`)

Antes de realizar o lancamento de uma versao maior ou merge definitivo nas branches mais criticas, execute a auditoria global defensiva em 6 fases:

```bash
trustkit audit security --profile standard --scope .
```

Perfis disponiveis:
- `quick`: Verificacao sintatica e checagem de segredos.
- `standard`: Analise de dependencias, analise de fluxo de dados e vulnerabilidades conhecidas.
- `deep`: Auditoria rigorosa de todas as camadas com geracao de ledger deterministico em `.trustkit/audits/security/findings.json`.

---

## 5. Perguntas Frequentes (FAQ)

### O que acontece se eu precisar alterar uma regra de negocio apos o Gate 2?
Apos o congelamento (Gate 2), qualquer modificacao manual em `spec.md` invalida o selo BLAKE3 e impede a conclusao das tarefas. Caso seja estritamente necessario alterar os requisitos, voce deve executar uma revisao formal que recalcule os hashes e atualize a Matriz de Rastreabilidade.

### Posso rodar as tarefas de uma mesma onda no mesmo terminal?
Sim. O TRustKit gerencia o isolamento fisico das tarefas criando pastas temporarias independentes (Git Worktrees efemeras). Mesmo que as tarefas sejam executadas simultaneamente, cada processo operara em sua propria arvore de trabalho, eliminando conflitos de arquivos.

### Por que o comando `trustkit pr` nao e chamado automaticamente ao final do ciclo?
Por exigencia constitucional de governanca (Invariante #12). A decisao de abrir um Pull Request para os ramos estaveis deve ser uma acao consciente e deliberada do desenvolvedor humano responsavel, e nunca uma automacao silenciosa de um bot ou agente.

### E obrigatorio utilizar o GitHub Projects para rodar o TRustKit?
Nao. O TRustKit implementa o padrao de Caso Especial (`UnlinkedLocalCycle`). Se voce iniciar o sistema sem fornecer uma issue (`trustkit init`), o ciclo operara normalmente no ambiente local, armazenando todos os dados no SQLite e ignorando as chamadas remotas de rede.

### O que significa a mensagem de erro sobre trailers de IA no commit?
Significa que o comando identificou textos como `Co-Authored-By: Claude` ou rodapes gerados automaticamente por ferramentas agênticas. Conforme a Invariante #13, o historico de commits deve refletir responsabilidade de engenharia limpa e profissional, sem tags de geracao artificial.

---

## 6. Guia de Solucao de Problemas (Troubleshooting)

### 6.1 Tabela de Codigos de Saida Semanticos (POSIX)

Ao integrar scripts de automacao ou investigar encerramentos inesperados, utilize a tabela oficial de codigos de retorno:

| Codigo | Identificador Semantico | Significado | O que fazer |
| :---: | :--- | :--- | :--- |
| **0** | `CleanPass` | Operacao concluida com aprovacao integral. | Prosseguir para a proxima etapa do fluxo. |
| **1** | `InfrastructureBoundaryFault` | Falha de disco, permissao, banco SQLite WAL ou Git. | Verificar espaco em disco, permissoes de escrita e integridade do `.git`. |
| **2** | `SpecOrTraceabilityIncomplete` | Spec incompleta, falha em Mermaid/Gherkin ou regra orfa. | Executar `trustkit spec lint` e preencher as colunas pendentes da matriz. |
| **3** | `TddLifecycleViolation` | Tentativa de registrar GREEN sem registro previo de RED. | Executar primeiro `trustkit task test <ID> --phase red` com o teste falhando. |
| **4** | `AstIntegrityOrInvariantViolation` | Violacao de invariantes: emojis, trailers IA, assercoes removidas. | Executar `trustkit gauntlet ast-audit` para visualizar o arquivo e linha exatos. |
| **5** | `CriticSwarmOrPlaywrightFailed` | Falha de compilacao, erro de linter ou reprovacao em testes E2E. | Verificar os logs de erro da suite de testes ou relatorio do Playwright. |
| **6** | `CircuitBreakerTripped` | Disjuntor acionado apos 5 reprovacoes consecutivas no Gauntlet. | Executar `trustkit doctor` para diagnostico e corrigir as falhas antes de reiniciar. |

---

### 6.2 Mensagens Comuns de Bloqueio e Como Resolver

#### Bloqueio 1: `MissingRedPhaseProof`
* **Causa:** Voce tentou marcar uma tarefa como completa ou registrar fase verde sem antes comprovar que o teste falhava.
* **Solucao:** Execute `trustkit task test <TASK_ID> --phase red` garantindo que o teste falhe de forma legitima. Apenas depois registre a fase green.

#### Bloqueio 2: `InvariantViolation: EmojiDetected`
* **Causa:** Foi encontrado um caractere de emoji ou simbolo visual grafico em algum arquivo modificado ou na mensagem de commit.
* **Solucao:** Remova o emoji do arquivo ou texto. Substitua por marcadores textuais claros como `[PASS]`, `[OK]` ou `[INFO]`.

#### Bloqueio 3: `TraceabilityGap: OrphanRule`
* **Causa:** Uma regra de negocio `RN-XXX` listada na especificacao nao possui uma tarefa `TASK` ou cenario de teste `E2E` amarrado na tabela de rastreabilidade.
* **Solucao:** Abra a especificacao `spec.md`, localize a tabela da Matriz de Rastreabilidade e garanta que todas as linhas de `RN` possuam vinculo completo ate a coluna de evidencias.

---

### 6.3 Recuperacao de Disjuntor Acionado (Circuit Breaker)

Se o ciclo falhar repetidamente por 5 vezes durante as verificacoes do Gauntlet, o sistema trava preventivamente para evitar consumo desnecessario de recursos.

Para resolver a situacao:

1. **Obter o relatorio detalhado de causas:**
   ```bash
   trustkit doctor
   ```
2. **Analisar as mensagens de erro:** O diagnostico informara quais assercoes, suites ou criticos reprovaram as tentativas.
3. **Aplicar as correcoes no codigo de producao ou nos testes.**
4. **Reiniciar a verificacao:** Uma vez ajustado o codigo, execute novamente o comando do Gauntlet para reavaliar a esteira.

---

## 7. Glossario de Termos e Linguagem Ubiqua

Para assegurar clareza e alinhamento em toda a comunicacao tecnica, utilize os termos formais do dominio do TRustKit:

* **`SddCycle` (Ciclo SDD):** Instancia de execucao isolada e auditavel correspondente a uma funcionalidade em desenvolvimento, identificada no formato `issue-<id>-AAAAMMDDHHMM`.
* **`GateStage` (Estagio de Portao):** Cada um dos marcos obrigatorios (Gate 0 a Gate 6) que condicionam a passagem para a etapa seguinte.
* **`ArtifactSeal` (Selo Criptografico):** Hash BLAKE3 combinado a contagem de nos de assercao que atesta a imutabilidade da especificacao no Gate 2.
* **`TraceabilityLink` (Elos de Rastreabilidade):** Associacao formal entre a Regra de Negocio (`RN`), o Requisito Funcional (`RF`), a Tarefa (`TASK`), o Teste de Tela (`E2E`) e a Evidencia (`Screenshot`).
* **`AtomicTask` (Tarefa Atomica):** Menor divisao de trabalho de desenvolvimento, restrita a um escopo coeso e executada sob ciclo TDD.
* **`ExecutionWave` (Onda de Execucao):** Grupo de tarefas atomicas independentes entre si que podem ser codificadas simultaneamente em arvores Git isoladas.
* **`ExecutionEvidence` (Evidencia de Execucao):** Registro digital imutavel contendo comando executado, duracao, saida sanitizada e codigo de retorno gravados no banco SQLite WAL.
* **`CircuitBreaker` (Disjuntor de Seguranca):** Mecanismo de protecao que interrompe a automacao ao atingir o teto de 5 falhas consecutivas.
* **`Git Worktree`:** Arvore de trabalho adicional e independente vinculada ao mesmo repositorio Git, permitindo manter multiplas branches ativas ao mesmo tempo sem alternancia de contexto destrutiva.
* **`AST (Abstract Syntax Tree)`:** Representacao estruturada em arvore do codigo-fonte utilizada pelo validador Tree-Sitter para verificar assercoes e regras lexicas em microssegundos.
* **`Modo Agente`:** Formato de execucao do CLI ativado via `--json` que entrega respostas estruturadas sem decoracoes e estritamente livres de campos nulos (`null`).
