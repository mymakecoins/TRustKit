# Guia do Usuario TRustKit

Manual Oficial de Operacao, Governanca de Spec-Driven Development (SDD) e Execucao Deterministica de Agentes.

Versao: 2.0 (Motor Autonomo, Milestones, Inteligencia AST e Gauntlet V2)
Data: Novembro/2026

---

## 1. Introducao e Principios Fundamentais

O TRustKit e um motor deterministico escrito nativamente em Rust projetado para governar fluxos de Spec-Driven Development (SDD), isolamento de execucao em Git Worktrees, Gauntlet Loop V2 e inteligencia de codigo estrutural.

Diferente de abordagens puramente declarativas ou harnesses legados baseados em linguagens interpretadas (Python ou Node.js), o TRustKit garante:
- Desempenho nativo com cold-starts em microssegundos (~1 a 3ms), permitindo a execucao de hooks sincronos sem latencia perceptivel.
- Eliminacao de alucinacoes na governanca: validacoes estruturais, integridade sintatica e verificacao de invariantes sao executadas pelo binario compilado `trustkit`, sem depender de inferencia probabilistica de LLMs.
- Isolamento rigoroso via Git Worktrees com Copy-on-Write (reflink), permitindo que agentes construtores trabalhem concorrentemente sem risco de condicao de corrida no indice do Git.
- Auditoria auditavel e imutavel via Event Sourcing gravada em SQLite em modo WAL (`.trustkit/state.db`) com hashes BLAKE3.

### Invariantes Constitucionais Inegociaveis

1. Proibicao Absoluta de Emojis: Nenhum emoji ou icone grafico e tolerado em codigo, documentacao, mensagens de commit ou saidas de terminal.
2. Proibicao Absoluta de Trailers de Autoria IA: Nenhuma marcacao como `Co-Authored-By` ou indicacoes geradas por modelos sao aceitas em commits ou documentacoes.
3. Mutacoes Deterministicas de Estado: Toda transicao de ciclo, validacao de gate e selo de artefato deve ser processada exclusivamente pelo binario `trustkit`.
4. Ciclo TDD Mecanico Estrito: Na fase de execucao de tarefas (Gate 4), o registro da fase RED (falha comprovada) e compulsorio antes de qualquer aceitacao da fase GREEN (sucesso comprovado).
5. Abertura de Pull Request Sob Demanda: Abertura de PR ocorre unicamente mediante ordem explicita do desenvolvedor humano (`trustkit pr`).
6. Protecao Estrita de Branches: Bloqueio absoluto de pushes diretos em branches protegidas (`main`, `master`, `staging`, `develop`).
7. Orientacao Contextual de Proximo Passo: Apresentacao compulsoria do array `next_steps` (JSON) ou do rodape `NEXT STEP:` (ASCII) emitido pelo motor `NextStepResolver`.
8. Paridade Documental Mandatoria em Releases: Toda promocao ou release do TRustKit deve ser publicada com sua base documental integralmente sincronizada (`README.md`, `README.en.md`, manuais em Markdown `docs/manual/MANUAL_DO_USUARIO.md` e `docs/manuals/user-guide.md`, bem como versoes HTML standalone `docs/manual/manual.html` e `docs/manuals/user-guide.html`). E proibido promover ciclos com documentacao defasada.

---

## 2. Planejamento Previo de Marcos (`trustkit milestones` / `tk milestone`)

Antes de iniciar o ciclo SDD contra documentos extensos de especificacao, o TRustKit disponibiliza o subsistema de planejamento de marcos para decompor o PRD e o ADR de forma aciclica (suporta tanto `trustkit milestones` quanto o alias singular `trustkit milestone` ou `tk milestone`).

### Analise e Planejamento (`trustkit milestones plan`)

Analisa estaticamente os arquivos PRD e ADR utilizando os parsers AST do `trustkit-ast`, identifica capacidades de negocio e restricoes tecnicas, deriva uma ordenacao topologica aciclica (DAG) e gera os arquivos Markdown individuais em `docs/milestones/` com citacao exata de intervalos de linhas (`#LXX-LYY`).

```bash
# Planeja marcos a partir de PRD e ADR
trustkit milestones plan --prd docs/PRD_TRustKit.md --adr docs/ADR_001_TRustKit_Architecture.md

# Especificando diretorio de saida customizado
trustkit milestones plan --prd docs/PRD_TRustKit.md --adr docs/ADR_001_TRustKit_Architecture.md --output-dir docs/milestones

# Saida estruturada em formato JSON sem campos nulos
trustkit milestones plan --prd docs/PRD_TRustKit.md --adr docs/ADR_001_TRustKit_Architecture.md --json
```

### Listagem de Marcos (`trustkit milestones list`)

Exibe todos os marcos planejados e persistidos no SQLite WAL, com ordem de execucao, dependencias e status atual:

```bash
# Listagem tabular no terminal
trustkit milestones list

# Listagem em formato JSON para automacoes
trustkit milestones list --json
```

### Inspecao Detalhada de Marco (`trustkit milestones inspect`)

Consulta os metadados completos de um marco especifico, incluindo descricao, lista de capacidades mapeadas, restricoes tecnicas e citacoes exatas de origem:

```bash
# Inspecao do marco M01
trustkit milestones inspect M01

# Inspecao com saida JSON
trustkit milestones inspect M01 --json
```

### Desmembramento do Marco em Features Executaveis (`trustkit features` / `tk feature`)

Apos definir os marcos evolutivos (`M01`, `M02`), o subsistema `tk features` (ou alias singular `tk feature`) le um Milestone e o desmembra em **Features** funcionais coesas (`M01-F01: Caminho Feliz`, `M01-F02: Validacoes`), sintetizando para cada Feature um arquivo `spec.md` (Specification V2) completo estruturado nas 4 camadas arquiteturais canonicas (`Schema & Contrato`, `Core & Dominio`, `API, UI & Adaptadores`, `Telemetria, Auditoria & E2E`):

```text
[ PRD.md (O Que & Por Que) ]  +  [ ADR.md (Como & Restricoes) ]
                         |
        +----------------+----------------+  (1. tk milestone plan)
        v                                 v
+-------------------------------+ +-------------------------------+
|   MILESTONE M01 (Fundacao)    | |   MILESTONE M02 (Expansao)    |
+-------------------------------+ +-------------------------------+
        |
        +--> [ FEATURE M01-F01: Caminho Feliz ]  (2. tk features plan --milestone M01)
        |      +-- TASK-001: Schema & Contrato   (3. tk auto --feature M01-F01)
        |      +-- TASK-002: Core & Dominio
        |      +-- TASK-003: API, UI & Adaptadores
        |      +-- TASK-004: Telemetria, Auditoria & E2E
        |
        +--> [ FEATURE M01-F02: Validacoes e Limites ]
```

```bash
# 1. Extrai e sintetiza as Features e seus spec.md V2 a partir do Milestone M01
tk features plan --milestone M01
trustkit features plan --milestone M01 --output-dir docs/features --json

# 2. Lista as Features planejadas do Milestone M01
tk features list --milestone M01
trustkit features list --milestone M01 --json

# 3. Inspeciona detalhes e blueprints de tarefas em 4 camadas da Feature M01-F01
tk features inspect M01-F01
trustkit features inspect M01-F01 --json
```

---

## 3. Inicializacao do Ciclo SDD (Gate 0)

O Gate 0 valida o ambiente de trabalho, o estado da arvore Git, a identidade no GitHub e cria o registro do ciclo no banco transacional SQLite.

### Inicializacao Direcionada por Marco (`trustkit init --milestone`)

Utiliza um arquivo de marco previamente decomposto como escopo fundamental do ciclo de desenvolvimento:

```bash
trustkit init --milestone docs/milestones/M01-fundacao.md
```

### Inicializacao Tradicional por Issue (`trustkit init --issue`)

Vincula o ciclo a uma issue remota do GitHub ou opera localmente:

```bash
# Inicializacao com issue remota
trustkit init --issue 42

# Inicializacao com extracao previa de PRD upstream
trustkit init --issue 42 --from-prd docs/PRD_TRustKit.md

# Inicializacao em modo JSON
trustkit init --issue 42 --json
```

### Consulta de Status, Diagnostico e Orientacao Contextual de Proximo Passo (`trustkit status` / `NextStepResolver`)

Permite verificar a qualquer instante o gate ativo, branch de trabalho, integridade do ciclo e historico do disjuntor (Circuit Breaker). Alem disso, todos os comandos do TRustKit (incluindo `trustkit status`) executam o resolvedor contextual deterministico (`NextStepResolver`), emitindo automaticamente a proxima acao exata para o estado corrente do ciclo:
- Em **Modo Humano (ASCII)**: exibe o bloco padronizado `NEXT STEP:` ao final da saida com prioridades `[PRIMARY]`, `[SECONDARY]` e `[OPTIONAL]`, gate de destino e comando completo.
- Em **Modo Agente (`--json`)**: inclui o array `next_steps` livre de `null` (`priority`, `target_gate`, `command` e `description`), consumido obrigatoriamente pela skill `trustkit-sdd` ao final de cada turno.

```bash
trustkit status
trustkit status --json
```

---

## 4. Modo Autonomo de Ponta a Ponta (`trustkit auto` v2.2)

Para execucoes completamente desatendidas, o TRustKit disponibiliza o comando unificado `trustkit auto`. A versao 2.2 incorpora motor de atomizacao deterministica, reconciliacao de ciclos ativos concorrentes, protecao por Circuit Breaker contra backlogs vazios, sintese de testes RED compilaveis e merge cumulativo seguro:

- **Reconciliacao de Ciclos Ativos Concorrentes:** Detecta ciclos preexistentes no SQLite WAL, testa integracao de branches no Git e suporta resolucao declarativa (`--new`, `--close-merged`, `--reuse-cycle`) sem travar a automacao.
- **Atomizacao Automatica (Gate 3):** Se o diretorio `tasks/` nao existir, o motor extrai da Matriz de Rastreabilidade (`spec.md`) ou da issue remota (`IssueBodyAtomizer`) as tarefas `tasks/TASK-NNN.md` no disco e no SQLite WAL.
- **Circuit Breaker contra Ciclo Vazio (`EmptyTaskBacklogFault`):** Aborta a transicao para o Gate 4 com exit code 1 caso a especificacao resulte em zero tarefas executaveis.
- **Scaffolding de Teste RED (`RedTestScaffolder`):** Sintetiza esqueletos de teste compilaveis na worktree, rejeitando filtros vazios (`ZeroFilteredTestsFault`) e mascaramento de erros de compilacao (`CompilationFailureInRedPhaseFault`).
- **Builder Agent Dispatcher:** Despacha agentes construtores entre as fases RED e GREEN com isolamento de `CARGO_TARGET_DIR`.
- **Merge Acumulativo Seguro (`SafeWorktreeMerge`):** Integra tarefas concorrentes da mesma onda sem executar `git branch -f` destrutivo, preservando todo o historico de commits.

```bash
# Execucao autonoma sobre uma Feature planejada (cria tasks e executa as waves da feature)
tk auto --feature M01-F01 --target develop
trustkit auto --feature M01-F01 --target develop

# Execucao autonoma sequencial de todas as Features pendentes de um Milestone
tk auto --milestone M01 --target develop

# Execucao autonoma a partir de issue remota (alto fluxo e multiplos colaboradores)
trustkit auto --issue 42 --target develop

# Execucao autonoma a partir de arquivo de especificacao local (--spec ou alias --specs)
trustkit auto --spec docs/specs/042-feature/spec.md --target develop
tk auto --specs docs/specs/042-feature/spec.md --target develop

# Com selecao explicita de runtime de terminal (auto, pty, native)
trustkit auto --issue 42 --runtime pty

# Emissao de telemetria completa em JSON
trustkit auto --issue 42 --json
```

---

## 5. Jornada Passo a Passo dos 7 Gates

Para fluxos interativos ou operacao supervisionada por desenvolvedores humanos, cada gate pode ser executado e validado sequencialmente.

### Gate 1: Especificacao V2 e Linting Estatico

O TRustKit disponibiliza ferramentas integradas para confeccionar prompts ricos e auditar especificacoes antes do congelamento:

```bash
# Gera prompt estruturado com fatiamento de contexto AST e busca FTS5
trustkit spec prompt --idea "Implementar modulo de autenticacao JWT" --context-hops 2 --budget 4000

# Executa linter estatico imediato (< 3ms) para validar secoes, diagramas Mermaid e Gherkin
trustkit spec lint --file docs/specs/042-feature/spec.md
```

### Gate 2: Selamento Criptografico (`trustkit spec lock`)

Aplica o selo de inviolabilidade BLAKE3 na especificacao e registra o estado congelado no SQLite WAL:

```bash
trustkit spec lock --spec docs/specs/042-feature/spec.md
```

### Gate 3: Decomposicao Atomica e Planejamento de Ondas (`trustkit wave plan`)

Analisa as definicoes de tarefas em `tasks/TASK-NNN.md`, constroi o grafo direcionado aciclico (DAG) via `petgraph`, detecta possiveis conflitos de arquivos e gera as ondas topologicas:

```bash
trustkit wave plan
trustkit wave plan --json
```

### Gate 4: Execucao Isolada em Git Worktrees e TDD Estrito

O Gate 4 despacha as tarefas em arvores de trabalho efemeras, monitora agentes construtores e fiscaliza o ciclo TDD:

```bash
# Despacha a onda de tarefas em worktrees isoladas
trustkit wave run --wave 1 --runtime auto

# Registro mandatorio da fase RED (comprovacao de falha do teste)
trustkit task test TASK-001 --phase red

# Registro mandatorio da fase GREEN (comprovacao de sucesso apos implementacao)
trustkit task test TASK-001 --phase green

# Revisao semantica multi-angulo de codigo (Gate 4 Refactor)
trustkit task review TASK-001 --effort medium

# Conclusao formal da tarefa atomica
trustkit task complete TASK-001 --code-review medium
```

### Gate 5: Gauntlet V2 e Invariantes de Seguranca

Auditoria em duas camadas que protege a base de codigo antes da homologacao final:

```bash
# Camada 0: Auditoria estatica AST em microssegundos (anti-cheat, emojis, trailers IA)
trustkit gauntlet ast-audit

# Execucao do Swarm de criticos e testes E2E Playwright
trustkit gauntlet run

# Execucao forçando revisao de seguranca da aplicacao no diff
trustkit gauntlet run --security-review

# Diagnostico de falhas, verificacao de cache local (sccache / ToolCachingPolicy / BatchExecutionConfig / compilation_cache / batch_grouping_mode) ou inspecao do disjuntor (Circuit Breaker)
trustkit doctor
```

### Gate 6: Certificacao, Merge e Pull Request

Finalizacao formal do ciclo, consolidacao das evidencias e integracao controlada:

```bash
# Certifica o ciclo, gera gauntlet-report.md e publica comentario na issue
trustkit done

# Merge deterministico na branch de destino
trustkit merge --target develop

# Abertura de Pull Request sob demanda explicita do desenvolvedor
trustkit pr --target develop
```

---

## 6. Auditoria de Seguranca Pre-Release (`trustkit audit security`)

Para homologacoes defensivas globais antes de merges em branches estaveis (`staging` ou `main`), o TRustKit disponibiliza uma auditoria de seguranca abrangente em 6 fases:

```bash
# Execucao no perfil padrao
trustkit audit security --profile standard --scope .

# Execucao em perfil aprofundado com relatorio JSON
trustkit audit security --profile deep --output-dir .trustkit/audits/security --json
```

---

## 7. Memoria Perene e Base de Conhecimento (`trustkit memory`)

O TRustKit mantem uma wiki de documentacao viva em `.trustkit/wiki/` indexada via SQLite FTS5 com busca semantica BM25 e suporte a wikilinks `[[slug]]`:

```bash
# Busca semantica por termos tecnicos
trustkit memory search --query "regras constitucionais" --limit 5

# Leitura estruturada de nota de conhecimento
trustkit memory read --slug "decisoes-arquiteturais"

# Criacao ou atualizacao atomica de nota
trustkit memory write --slug "padroes-de-teste" --title "Padroes de Teste" --tags "qualidade,tdd" --body "Texto da nota..."

# Listagem de todas as notas catalogadas
trustkit memory list

# Reindexacao completa do indice FTS5 a partir dos arquivos Markdown
trustkit memory reindex
```

---

## 8. Inteligencia de Grafo Estrutural (`trustkit graph`)

Analisa as dependencias de codigo e wiki via Tree-Sitter, calculando acoplamento aferente/eferente, detectando God Nodes, avaliando a profundidade de modulos (Deep Modules vs ShallowPassThrough pelo Deletion Test) e classificando costuras (`RealSeam` vs `HypotheticalSeam`):

```bash
# Constroi ou atualiza o grafo com protecao Shrink-Guard
trustkit graph build

# Diagnostica a saude arquitetural, God Nodes, shallow_modules (Deletion Test) e seam_classifications
trustkit graph health --top-god-nodes 10

# Gera 3 briefings arquiteturais contrastantes (Design It Twice) no Gate 1
trustkit spec prompt --idea "Nova porta de dominio" --design-it-twice --json

# Calcula o menor caminho de acoplamento entre dois simbolos (BFS)
trustkit graph path --source "src/lib.rs::SddCycle" --target "crates/trustkit-db::SqliteWalRepositoryAdapter"
```

---

## 9. Referencia Rapida de Comandos

| Comando | Descricao |
| :--- | :--- |
| `trustkit auto` | Executa o ciclo SDD de forma autonoma (Gates 0 a 6) com atomizacao, disjuntor e merge seguro. |
| `trustkit milestones plan` | Decompoe PRD e ADR em marcos aciclicos com citacoes exatas de linhas. |
| `trustkit milestones list` | Lista todos os marcos planejados e persistidos no banco WAL. |
| `trustkit milestones inspect` | Inspeciona capacidades, restricoes e citacoes de um marco. |
| `trustkit init` | Inicializa sessao de ciclo SDD (suporta `--issue` e `--milestone`). |
| `trustkit status` | Consulta o gate ativo e a saude do ciclo corrente. |
| `trustkit spec prompt` | Gera prompt contextualizado para formulacao de especificacoes. |
| `trustkit spec lint` | Executa linter estatico na especificacao (`spec.md`). |
| `trustkit spec lock` | Congela a especificacao com selo criptografico BLAKE3. |
| `trustkit wave plan` | Planeja ondas topologicas de execucao a partir de tarefas atomicas. |
| `trustkit wave run` | Despacha onda de tarefas em Git Worktrees isoladas. |
| `trustkit task test` | Executa e fiscaliza fases do ciclo TDD (`--phase red` ou `green`). |
| `trustkit task review` | Realiza revisao semantica multi-angulo do diff da tarefa. |
| `trustkit task complete` | Conclui formalmente a tarefa apos comprovacao de testes. |
| `trustkit gauntlet ast-audit` | Audita diff na Camada 0 em microssegundos contra violacoes constitucionais. |
| `trustkit gauntlet run` | Executa a esteira completa de criticos e testes E2E do Gauntlet. |
| `trustkit doctor` | Diagnostica falhas e status do disjuntor de seguranca. |
| `trustkit done` | Certifica o ciclo e gera relatorio oficial de homologacao. |
| `trustkit merge` | Executa merge deterministico na branch de destino. |
| `trustkit pr` | Abre Pull Request sob demanda explicita do desenvolvedor humano. |
| `trustkit audit security` | Executa auditoria de seguranca pre-release em 6 fases. |
| `trustkit memory` | Gerencia notas, links e busca semantica FTS5 na memoria perene. |
| `trustkit graph` | Analisa grafo de dependencias AST, acoplamento e saude arquitetural. |
| `trustkit hook` | Rota ultrarrapida (< 3ms) para interceptacao sincrona de comandos de terminal. |
| `trustkit update` | Auto-atualizacao nativa com disjuntor semantico AST (`tk update [--check|--force]`). |
