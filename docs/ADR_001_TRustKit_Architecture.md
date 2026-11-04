# ADR-001: Arquitetura Híbrida em Rust (TRustKit) para Governança Determinística SDD & Gauntlet V2

**Versão:** 1.1 (Enriquecido com Diretrizes Clean Code — Robert C. Martin)  
**Data:** Outubro/2026  
**Status:** Aceito  
**Decisor:** Arquiteto de Software Principal, DBA Sênior & Engenheiro de Software Sênior  
**Contexto:** Especificação técnica e arquitetural do motor `TRustKit` (binário `trustkit`), projetado para substituir a orquestração puramente interpretada/declarativa dos 7 Gates da `mmc-gauntlet-sdd-v2` por um núcleo compilado nativamente em Rust com validação AST em microssegundos, isolamento paralelo de Git Worktrees, Event Sourcing em SQLite WAL e conformidade estrita com os princípios de *Código Limpo* (Capítulos 2, 7, 8, 9, 11 e 12).

---

## 1. Questão ou Problema

Os fluxos modernos de *Spec-Driven Development* (SDD) combinados com crítica adversarial (*Gauntlet Loop*) enfrentam um dilema arquitetural severo entre **velocidade de execução** e **rigor determinístico**:

1. **Skills Declarativas Puras (`SKILL.md`):** São rápidas porque rodam dentro da sessão quente do agente hospedeiro (aproveitando o *Prompt KV-Cache*), mas delegam 100% da fiscalização de invariantes ao próprio LLM. Além disso, o LLM perde dezenas de segundos executando chamadas sequenciais de terminal (`git`, `gh`, `grep`) e gasta milhares de tokens lendo diffs inteiros para auditar regras mecânicas (detecção de adulteração de testes, ausência de emojis, fechamento da matriz de rastreabilidade e formatação de PRs).
2. **Harnesses Externos Tradicionais (Python / Node.js / TypeScript):** Garantem controle de estado fora do LLM, mas impõem um *cold-start* de `300ms` a `2.000ms` por invocação de subprocesso (inviabilizando hooks síncronos `PreToolUse`/`PostToolUse`), consomem muita memória e frequentemente destroem o KV-Cache do modelo ao recriarem sessões descontextualizadas.
3. **Colisão de Estado em Execuções Multi-Agente (Gate 4):** Quando múltiplos *Builders* rodam em paralelo na mesma árvore de trabalho, ocorrem condições de corrida no índice do Git e sobrescrita mútua de arquivos e evidências do Playwright.

**A questão central é:** Como arquitetar um harness para os 7 Gates da `mmc-gauntlet-sdd-v2` que mantenha o *KV-Cache* quente de uma Skill enxuta, elimine o *overhead* de turnos mecânicos do LLM, execute validações de contrato/AST em `< 5ms`, isole *Builders* paralelos sem custo de disco, registre prova criptográfica imutável de cada etapa e mantenha um núcleo de domínio 100% desacoplado de bibliotecas de terceiros?

---

## 2. Decisão

**Adotar a arquitetura "Hybrid Rust Core + Thin Skill" materializada no binário estático `trustkit` (Rust 2024 Edition), estruturado sob Arquitetura Hexagonal Estrita (Ports & Adapters), com modelos de domínio puros (*Plain Old Rust Structs* sem dependências externas e livres de `null`), Injeção de Dependência via *Composition Root* dedicado, tradução centralizada de erros nas bordas, persistência transacional em SQLite Embarcado (WAL mode), análise sintática via `tree-sitter` e isolamento concorrente de *Git Worktrees* com Copy-on-Write.**

### 2.1. Interface & Experiência do Usuário / Agente (Frontend CLI)
* **Paradigma de Interface:** CLI Ultraleve sem TUI (`ratatui`) e sem servidor Web (`axum`), minimizando o tamanho do binário, a superfície de ataque e o tempo de inicialização (`< 3ms`).
* **Parser de Comandos:** `clap` (v4, isolado na crate de borda `trustkit-cli`) para subcomandos fortemente tipados (`init`, `spec`, `atomize`, `wave`, `task`, `gauntlet`, `done`, `merge`, `pr`, `doctor`, `status`, `hook`, `mcp`).
* **Protocolo Agêntico Dual (Livre de `null`):**
  * Modo `--json` (compacto, zero ANSI, schema determinístico baseado em *Special Case Objects* e listas vazias `[]`) para consumo direto por LLMs com gasto mínimo de tokens.
  * Servidor **MCP (Model Context Protocol)** embutido via `stdio` (`trustkit mcp`) para exposição nativa das ferramentas ao Claude Code, Antigravity e IDEs compatíveis.

### 2.2. Motor de Execução & Persistência (Backend Core em Rust)
* **Linguagem:** Rust (`1.82+`, Edition 2024), compilado como binário único estático com *Link-Time Optimization* (`lto = "thin"`, `strip = true`).
* **Núcleo de Domínio Zero-Dependency (`trustkit-core`):** Composto exclusivamente pela biblioteca padrão do Rust (`std`), contendo as entidades da Linguagem Ubíqua, a máquina de estados dos 7 Gates (*Typestate Pattern*), as 16 *Hard Invariants* e as definições dos **Ports (Traits)**. Nenhuma crate externa (`rusqlite`, `tree-sitter`, `gix`, `petgraph`, `blake3`, `tokio`) entra no `Cargo.toml` do `trustkit-core`.
* **Runtime Assíncrono na Borda:** `tokio` restrito aos subcomandos de I/O concorrente na camada de orquestração (`init`, `wave run`, `gauntlet run`, `done`, `pr`); subcomandos síncronos (`hook`, `spec verify`, `status`) executam sem inicializar o runtime assíncrono para garantir `< 3ms` de *cold-start*.
* **Banco de Dados Embarcado (Adapter):** **SQLite 3** via `rusqlite` (`bundled`), operando em modo `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`, `PRAGMA busy_timeout = 5000;` e `NOT NULL` em 100% das colunas de estado.

---

## 3. Justificativa

### 3.1. Por que Rust Nativo ("True Rust") em vez de Python ou TypeScript/Node.js?
**Vantagens:**
* **Cold-start de 1 a 3ms:** Permite interceptar 100% das chamadas de ferramentas do agente via hooks (`PreToolUse` / `PostToolUse`) sem qualquer atraso perceptível. Em Python/Node, 50 chamadas de ferramentas acumulariam até 60 segundos apenas em boot de interpretador.
* **Análise AST em Memória Nativa (`tree-sitter`):** Analisa árvores sintáticas de dezenas de arquivos em `< 5ms` sem serialização FFI lenta, detectando *test dilution*, uso de `any` e quebra de assinaturas instantaneamente.
* **Tipagem Algébrica e *Typestate Pattern* (Eliminação Nativa de `null`):** Em Rust, os estágios (`Gate0Discovery`, `Gate2SpecSealed`, `Gate5GauntletCleared`, `Gate6Certified`) e os Casos Especiais (`IssueLinkage::UnlinkedLocalCycle`) são modelados no sistema de tipos. É matematicamente impossível, em tempo de compilação, chamar a rotina de certificação ou merge sem possuir o *token* de estado `Gate5GauntletCleared`.
* **Distribuição Zero-Dependency:** Um único arquivo binário (`trustkit`) copiado para `~/.local/bin` funciona em qualquer máquina Linux sem conflitos de `virtualenv`, `pip`, `nvm` ou versões do Node.

**Trade-off:** Maior rigor em tempo de compilação e curva de desenvolvimento inicial mais exigente que scripts em Python/TS — amplamente compensado pela eliminação de bugs de runtime e pela velocidade perpétua em todas as execuções futuras.

### 3.2. Por que Arquitetura Hexagonal Estrita (Ports & Adapters) + Thin Skill?
**Vantagens:**
* **Domínio Blindado contra Terceiros (Clean Code Cap. 8):** Se amanhã trocarmos `git2` por `gix`, `rusqlite` por `redb`, ou `petgraph` por um algoritmo próprio de ordenação topológica, **zero linhas** de `trustkit-core` serão alteradas.
* **Testes Unitários F.I.R.S.T. em Sub-Milissegundo:** Como `trustkit-core` depende apenas de traits (Ports), toda a máquina de estados dos 7 Gates e das 16 Invariantes é testada em memória em `< 1ms` usando *In-Memory Fake Adapters*, sem tocar disco, rede ou banco SQLite.

### 3.3. Por que SQLite em Modo WAL + Exportação Markdown?
**Vantagens:**
* **Concorrência Sem Bloqueio (Multiple Readers, Single Atomic Writer):** Durante o Gate 4 e Gate 5, múltiplos processos Builders e Críticos gravam eventos simultaneamente. O modo `WAL` (*Write-Ahead Logging*) com `busy_timeout = 5000ms` garante latência de escrita sub-milissegundo sem erro de `SQLITE_BUSY`.
* **Auditoria Imutável ("O log de eventos prova"):** O relatório final `gauntlet-report.md` não é "escrito criativamente" pelo LLM; ele é **renderizado deterministicamente pelo `trustkit`** a partir das linhas da tabela `execution_evidences` do SQLite, contendo os hashes `BLAKE3` e os `Exit Codes` reais capturados pelo processo Rust.

---

## 4. Pilares de Sustentabilidade Clean Code (Capítulos 7, 8, 11 e 12)

### 4.1. Isolamento de Limites e Terceiros — Ports & Adapters (Capítulo 8)

Nenhuma struct, erro ou tipo de biblioteca externa (`rusqlite`, `tree-sitter`, `gix`, `petgraph`, `blake3`, `reqwest`, `clap`) é importado em `trustkit-core`. O domínio define **6 Contratos de Limite (Ports)** que são implementados por **Adapters** dedicados nas crates de infraestrutura:

| Port Interno (`trustkit-core::ports`) | Responsabilidade no Domínio | Adapter / Wrapper Concreto | Biblioteca Externa Encapsulada |
| :--- | :--- | :--- | :--- |
| **`CycleEvidenceRepositoryPort`** | Persistir e consultar `SddCycle`, `ArtifactSeal`, `TraceabilityLink`, `AtomicTask` e `ExecutionEvidence`. | `SqliteWalRepositoryAdapter` (`trustkit-db`) | `rusqlite` (SQLite3 C bundled) |
| **`ArtifactHasherPort`** | Gerar e verificar assinaturas criptográficas (`ArtifactSeal`) de especificações e suítes de teste. | `Blake3ParallelSealerAdapter` (`trustkit-db`) | `blake3` + `rayon` |
| **`SyntaxTreeAuditorPort`** | Auditar integridade de testes (`VerifyTestIntegrity`), escanear invariantes léxicas, validar Mermaid/Gherkin e fatiar contexto (`SliceAstContext`). | `TreeSitterAstAdapter` (`trustkit-ast`) | `tree-sitter` + gramáticas TS/Rust/Py/Go/SQL |
| **`WaveTopologyPlannerPort`** | Construir o grafo de dependências entre `AtomicTasks`, detectar ciclos e agrupar em `ExecutionWaves`. | `PetgraphTopologyAdapter` (`trustkit-ast`) | `petgraph` |
| **`WorktreeIsolationPort`** | Provisionar e podar *Git Worktrees* efêmeras com CoW/symlinks, inspecionar árvore compartilhada e restaurar evidências alheias (`RestoreForeignEvidence`). | `GitWorktreeReflinkAdapter` (`trustkit-orch`) | `gix` / `git2` + syscalls POSIX `ioctl_ficlone` |
| **`ProjectBoardGatewayPort`** | Verificar identidade GitHub ativa, sincronizar as 9 colunas do GitHub Projects v2, publicar comentário na issue e abrir PR sob demanda. | `GitHubCliGraphQlAdapter` (`trustkit-gh`) | `gh` CLI / GraphQL API v4 |

### 4.2. Separação Estrita entre Construção e Execução — DI & Modelos Limpos (Capítulo 11)

O processo de inicialização e montagem do grafo de objetos (**Construção**) é completamente isolado da lógica de negócio dos 7 Gates (**Execução**):

1. **Modelos de Domínio como PORS (*Plain Old Rust Structs*):** As entidades (`SddCycle`, `AtomicTask`, `TraceabilityLink`, `ExecutionEvidence`) e os serviços de domínio (`GateTransitionService`, `GauntletVerdictEvaluator`) não instanciam adaptadores concretos, não leem variáveis de ambiente e **jamais utilizam *Service Locators* ou *Singletons* globais (`lazy_static!` / `OnceLock` de infraestrutura)**.
2. **Injeção de Dependência Explícita via Construtor:** Os serviços de domínio recebem referências para as abstrações (`&dyn CycleEvidenceRepositoryPort`, `&dyn SyntaxTreeAuditorPort`, etc.).
3. **Composition Root Único (`trustkit-cli::bootstrap`):** Apenas o módulo `bootstrap.rs` na camada mais externa (`trustkit-cli`) conhece simultaneamente o `trustkit-core` e as crates de infraestrutura, montando o pipeline em duas rotas distintas:
   * **Fast Synchronous Bootstrap (`< 1ms`):** Para `trustkit hook`, instancia apenas `TreeSitterAstAdapter` e `SqliteWalRepositoryAdapter` na stack principal, sem inicializar `tokio` nem clientes de rede.
   * **Full Concurrent Bootstrap:** Para `init`, `wave run` e `gauntlet run`, inicializa o runtime `tokio` e injeta todos os 6 Adapters nos casos de uso do domínio.

```text
[main.rs / bootstrap.rs (Composition Root - Construcao)]
   |
   |-- 1. Le argumentos CLI (clap) e configuracao de ambiente
   |-- 2. Instancia Adapters concretos (SqliteWalAdapter, TreeSitterAdapter, GitWorktreeAdapter...)
   |-- 3. Injeta os Adapters como Traits (&dyn Port) no UseCase do `trustkit-core`
   v
[trustkit-core::use_cases (Execucao Pura - Desacoplada)]
   |-- Opera exclusivamente sobre PORS (SddCycle, AtomicTask) e Ports abstratos
```

### 4.3. Estratégia Centralizada de Erros e Exceções (Capítulo 7)

Para respeitar o **Princípio Aberto/Fechado (OCP)** e evitar que erros de baixo nível poluam as assinaturas do domínio ou vazem detalhes internos para o agente LLM:

1. **Erros de Domínio Semânticos e Propagação Limpa (Sem Poluição de Assinatura):**
   * O `trustkit-core` define um único tipo de erro de domínio rico em contexto (`DomainViolation`), propagado de forma ergonômica e não intrusiva pelo operador `?` do Rust (equivalente funcional às *Unchecked Domain Exceptions* de Martin, pois adicionar um novo erro de infraestrutura em um Adapter nunca altera a assinatura dos métodos do domínio).
2. **Tradução Obrigatória na Borda (*Boundary Error Translation*):**
   * Cada Adapter captura os erros nativos da biblioteca de terceiros (`rusqlite::Error`, `gix::Error`, `serde_json::Error`) imediatamente na borda e os traduz para variantes ricas do domínio contendo **o que falhou**, **qual invariante de negócio (`RN-001..RN-016`) foi acionada** e **qual a ação corretiva determinística**:
     * `DomainViolation::SpecSealBroken { file_path, expected_blake3, actual_blake3, invariant: "RN-001" }`
     * `DomainViolation::TestIntegrityCompromised { test_file, line, removed_assertion, invariant: "RN-004" }`
     * `DomainViolation::CircuitBreakerTripped { attempt: 5, max_attempts: 5, invariant: "RN-005" }`
     * `DomainViolation::ForbiddenCommandIntercepted { command, reason, invariant: "RN-006" }`
3. **Handler Global Único na Entrada CLI/MCP (`CliBoundaryErrorHandler`):**
   * Nenhum módulo interno escreve em `stderr` ou chama `std::process::exit()`. Apenas o `CliBoundaryErrorHandler` em `main.rs` intercepta o `Result<DomainOutcome, DomainViolation>`, sanitiza segredos, formata a saída (JSON sem `null` para o agente ou tabela ASCII para o humano) e mapeia para códigos de saída POSIX semânticos:

| Código de Saída (`ExitCode`) | Categoria de `DomainViolation` | Significado Determinístico para o Agente / Harness |
| :---: | :--- | :--- |
| **`0`** | `CleanPass` / `Success` | Operação concluída com 100% de conformidade. |
| **`1`** | `InfrastructureBoundaryFault` | Falha técnica de I/O local (disco cheio, permissão negada) traduzida na borda. |
| **`2`** | `SpecOrTraceabilityIncomplete` | Spec V2 sem Mermaid/Gherkin ou Matriz $\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E}$ aberta (`RN-001`). |
| **`3`** | `TddLifecycleViolation` | Tentativa de concluir tarefa sem prova prévia de falha na fase `RED` (`RN-002`). |
| **`4`** | `AstIntegrityOrInvariantViolation` | Camada 0 reprovada: *test dilution*, emoji, trailer de IA, `raw.githubusercontent` ou escopo violado (`RN-004`, `RN-007`, `RN-008`, `RN-013`). |
| **`5`** | `CriticSwarmOrPlaywrightFailed` | Camada 1 do Gauntlet reprovada (Lint/Build, Playwright E2E ou Críticos LLM); permite retry até o limite. |
| **`6`** | `CircuitBreakerTripped` | 5ª falha consecutiva atingida (`RN-005`). Execução autônoma bloqueada; requer intervenção humana. |

### 4.4. Matriz de Avaliação de Simplicidade Arquitetural — As 4 Regras de Kent Beck (Capítulo 12)

| Ordem de Prioridade | Regra de Projeto Simples (Kent Beck) | Como a Arquitetura do `TRustKit` Atende Rigorosamente à Regra |
| :---: | :--- | :--- |
| **1º** | **Passa em todos os testes (Princípios F.I.R.S.T.)** | - **Fast:** O núcleo `trustkit-core` não possui dependências de I/O; 100% das regras dos 7 Gates e 16 Invariantes rodam em memória em `< 5ms` via `cargo test`.<br>- **Independent & Repeatable:** Os Adapters concretos (`SqliteWalRepositoryAdapter` e `GitWorktreeReflinkAdapter`) são testados em diretórios temporários isolados (`tempfile::TempDir`) e bancos SQLite `:memory:`, garantindo zero estado compartilhado entre testes.<br>- **Self-Validating:** Todo teste valida invariantes tipadas sem inspeção manual. |
| **2º** | **Elimina a duplicação (DRY)** | - **Fonte Única de Regras (`trustkit-core::invariants`):** A mesma regra que valida commits/arquivos no `trustkit hook pre-tool` é reutilizada na Camada 0 do Gauntlet (`trustkit gauntlet ast-audit`) e na preparação do PR (`trustkit pr`), eliminando duplicação entre hooks, CLI, servidor MCP e templates Markdown.<br>- **Renderização Única a partir do Event Log:** O relatório `gauntlet-report.md` e o resumo técnico da issue são projetados a partir da mesma tabela `execution_evidences` do SQLite. |
| **3º** | **Expressa a intenção do desenvolvedor** | - **Linguagem Ubíqua e Typestate:** Os nomes das structs e métodos refletem exatamente o domínio SDD (`SealSpecification`, `VerifyTestIntegrity`, `TripCircuitBreaker`, `RestoreForeignEvidence`), eliminando nomes obscuros ou sufixos vazios como `Manager`/`Processor`.<br>- As transições de Gate são expressas na assinatura de tipos (`Gate2SealedSpec -> Gate3WaveSchedule -> Gate5ClearedGauntlet`). |
| **4º** | **Minimiza o número de entidades e componentes** | - **Binário Único Sem Daemons:** Em vez de microsserviços, filas externas (Redis/RabbitMQ), banco cliente-servidor (PostgreSQL) ou interfaces TUI/Web supérfluas, toda a solução compila em **1 único binário estático (`trustkit`)** apoiado por **1 arquivo SQLite local (`.trustkit/state.db`)** e **5 tabelas estritamente necessárias**. |

---

## 5. Consequências

### 5.1. Positivas
* **[POSITIVO] Redução Drástica de Latência e Custo de Tokens:** A Camada 0 do Gauntlet em Rust barra instantaneamente specs incompletas, matrizes de rastreabilidade quebradas, adulteração de testes e emojis em `< 5ms` com **zero tokens gastos**.
* **[POSITIVO] Isolamento Físico de Builders:** Nenhuma tarefa de uma onda interfere nos arquivos não commitados de outra tarefa paralela graças às *Git Worktrees* efêmeras providas pelo `GitWorktreeReflinkAdapter`.
* **[POSITIVO] Fiscalização Real de TDD (`RED -> GREEN`):** Um Builder não consegue declarar tarefa pronta sem antes ter registrado evidência de falha (`RedVerified`) e posterior aprovação (`GreenVerified`) sob supervisão do binário `trustkit`.
* **[POSITIVO] Manutenibilidade e Testabilidade Puras:** O isolamento via Ports & Adapters permite evoluir ou substituir o parser AST, o cliente GitHub ou o banco embarcado sem tocar no domínio.

### 5.2. Negativas
* **[NEGATIVO] Manutenção das Gramáticas `tree-sitter`:** Novas linguagens adotadas pela organização no futuro exigirão a adição da respectiva crate de gramática no `Cargo.toml` de `trustkit-ast` e recompilação do binário.
* **[NEGATIVO] Gerenciamento de Ciclo de Vida de Worktrees:** Falhas abruptas de energia ou `SIGKILL` durante o Gate 4 podem deixar diretórios de worktrees órfãos em `.trustkit/worktrees/`.

### 5.3. Mitigações

| Risco / Consequência Negativa | Estratégia de Mitigação Arquitetural |
| :--- | :--- |
| **Worktrees órfãs após `SIGKILL` ou falha de processo** | O subcomando `trustkit init` e um *Drop Guard* no `GitWorktreeReflinkAdapter` executam `git worktree prune` e limpam diretórios órfãos registrados como inativos no `.trustkit/state.db`. |
| **Multiplicação de espaço em disco por `node_modules` nas Worktrees** | Ao provisionar cada worktree em `.trustkit/worktrees/task-NNN`, o `GitWorktreeReflinkAdapter` cria automaticamente *symlinks* (ou cópia `reflink` CoW) dos diretórios pesados de dependências (`node_modules`, `.venv`, `target`) a partir da raiz do repositório. |
| **Linguagem sem gramática AST específica** | O `TreeSitterAstAdapter` possui *fallback* em dois níveis: (1) AST completa para TS/JS/Rust/Python/Go/SQL; (2) `InvariantLexicalScanner` baseado em autômatos finitos determinísticos (`regex` crate) para arquivos de configuração ou linguagens secundárias. |

---

## 6. Arquitetura Técnica Detalhada

### 6.1. Diagrama de Componentes (C4 Model — Nível 2 & 3 com Ports & Adapters)

```text
+-----------------------------------------------------------------------------------+
|                   AMBIENTE HOSPEDEIRO (Dev / Agente Principal)                    |
|  [Desenvolvedor Humano]        [Thin Skill: /trustkit] (Claude Code / Antigravity)|
+------------+-----------------------------------------+----------------------------+
             | CLI (stdout/stderr)                     | CLI (--json) / MCP (stdio)
             v                                         v
+-----------------------------------------------------------------------------------+
|                         BINARIO ESTATICO: `trustkit` (Rust)                       |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  | Camada de Entrada & Composition Root (`trustkit-cli`)                       |  |
|  |  - `bootstrap.rs` (DI Wiring: Fast Sync Path vs Concurrent Tokio Path)      |  |
|  |  - `CliBoundaryErrorHandler` (DomainViolation -> POSIX ExitCode + JSON)     |  |
|  |  - CLI Router (`clap` v4) & MCP Server (`stdio` JSON-RPC)                   |  |
|  +--------------------------------------+--------------------------------------+  |
|                                         | Invoca Use Cases                        |
|  +--------------------------------------v--------------------------------------+  |
|  | Nucleo de Dominio Puro (`trustkit-core` - Zero External Crates / Zero NULL) |  |
|  |  - Entidades PORS: `SddCycle`, `ArtifactSeal`, `AtomicTask`, `TraceLink`    |  |
|  |  - Typestate FSM: `Gate0Discovery` -> `Gate2SpecSealed` -> `Gate6Certified` |  |
|  |  - Regras: 16 Hard Invariants + Circuit Breaker (Max 5 Loops)               |  |
|  |  - Ports (Traits): `CycleEvidenceRepositoryPort`, `SyntaxTreeAuditorPort`,  |  |
|  |    `WaveTopologyPlannerPort`, `WorktreeIsolationPort`, `ProjectBoardPort`   |  |
|  +------+-------------------+-------------------+--------------------+---------+  |
|         ^                   ^                   ^                    ^            |
|         | Implementa Port   | Implementa Port   | Implementa Port    | Impl. Port |
|  +------+-------+   +-------+-------+   +-------+-------+   +--------+---------+  |
|  | `trustkit-db`|   | `trustkit-ast`|   |`trustkit-orch`|   |  `trustkit-gh`   |  |
|  | SqliteWal    |   | TreeSitterAst |   | GitWorktree   |   | GitHubCliGraphQl |  |
|  | Repository   |   | Adapter +     |   | ReflinkAdapter|   | Adapter          |  |
|  | Adapter +    |   | PetgraphWave  |   | + TddPhase    |   | (9-Col Projects, |  |
|  | Blake3Sealer |   | Adapter       |   | RunnerAdapter |   | Issue & PR)      |  |
|  +------+-------+   +-------+-------+   +-------+-------+   +--------+---------+  |
+---------|-------------------|-------------------|--------------------|------------+
          |                   |                   |                    |
          v                   v                   v                    v
   [.trustkit/state.db] [Codigo-Fonte]  [.trustkit/worktrees/]  [GitHub GraphQL API]
```

### 6.2. Fluxo Principal de Execução (7 Gates + Two-Tier Gauntlet)

```text
[Gate 0: trustkit init]
   |-- Paralelo (tokio::join! via Adapters injetados):
   |     1. `GitHubIdentityGuard`: confirma conta ativa no gh
   |     2. `GitWorktreeReflinkAdapter`: inspeciona `git status --porcelain` (isola `PreExistingDirtyFiles`)
   |     3. Detecta `.specify/` (seleciona `SpecKitIntegrated` ou `StandardGauntletV2`)
   |     4. `GitHubProjectsGraphQlAdapter`: move card para `In progress` (ou usa `UnlinkedLocalCycle`)
   |     5. Cria/reabre `docs/specs/<issue-id>-AAAAMMDDHHMM/` e grava `SddCycle` no `.trustkit/state.db`
   v
[Gate 0a/1: trustkit spec init [--from-prd] & trustkit spec verify]
   |-- Se `--from-prd`: `PrdSpecificationIngestor` extrai RN, RF e jornadas e gera `<topico>-spec.md`
   |-- Verifica presenca de: Visao Product Trio + Mermaid `flowchart LR` + Gherkin (`Dado/Quando/Entao`) + Matriz RN->RF->TASK->E2E
   v
[Gate 2: trustkit spec lock (SealSpecification)]
   |-- `Blake3ParallelSealerAdapter` calcula hash BLAKE3 da Spec e de todos os testes existentes
   |-- `TreeSitterAstAdapter` conta nos de assercao (`expect`/`assert`) de cada teste existente
   |-- Persiste `ArtifactSeal` na tabela `artifact_seals` -> Transiciona para `SpecLockState::Frozen`
   v
[Gate 3: trustkit atomize & trustkit wave plan]
   |-- Le `TASK-NNN.md` e `INDEX.md` (ou `tasks.md` no perfil `SpecKitIntegrated`)
   |-- `PetgraphTopologyAdapter` monta DAG: valida aciclicidade, cobertura de 100% das RNs e separa tarefas com arquivos conflitantes
   |-- Agrupa tarefas em `ExecutionWaves` (Onda 1: Banco/Setup, Onda 2: Dominio/Servicos, Onda 3: API, Onda 4: E2E)
   v
[Gate 4: trustkit wave run --wave <W>]
   |-- Para cada `AtomicTask` da Onda W em paralelo:
   |     1. `GitWorktreeReflinkAdapter` provisiona `.trustkit/worktrees/TASK-NNN` + symlink `node_modules`
   |     2. `AstContextSlicer` extrai pacote enxuto de assinaturas publicas (`SliceAstContext`)
   |     3. `TddPhaseGuard` exige `trustkit task test --phase red` (Exit != 0) ANTES de `--phase green` (Exit == 0)
   |     4. Valida commit `<type>: #<id> <msg>` (sem emojis, sem Co-Authored-By) e faz merge na branch `feat/<id>-<slug>`
   v
[Gate 5: trustkit gauntlet run (Loop max 5x - Circuit Breaker)]
   |-- CAMADA 0 (Rust Nativo < 5ms - Zero Tokens):
   |     1. Verifica `ArtifactSeal` BLAKE3 da Spec congelada (Spec Tamper Detection)
   |     2. `TestIntegrityAuditor` analisa AST dos testes: falha se algum `expect`/`assert` antigo foi removido/comentado ou recebeu `.skip`
   |     3. `InvariantLexicalScanner` escaneia diff: falha se encontrar Emoji Unicode, `Co-Authored-By`, `raw.githubusercontent.com` ou `@ts-ignore`/`any`
   |     4. Verifica se apenas arquivos permitidos pelo escopo da Spec/Tasks foram alterados
   |-- CAMADA 1 (Swarm Concorrente + Playwright E2E):
   |     1. Static & Security Critic: executa `npm run lint && npm run build` nas apps tocadas (`Exit Code: 0`)
   |     2. Playwright Visual Critic: executa `npm run test:e2e`, confere screenshots da Jornada Mermaid em `tests/evidence/` e chama `RestoreForeignEvidence`
   |     3. Domain & Technical Adversarial Critic: valida concorrencia, idempotencia e 100% das RNs
   |     4. Contract Compliance Critic: audita aderencia semantica aos blocos Gherkin
   |-- Se qualquer camada == FAIL: incrementa `gauntlet_attempt`. Se == 5 -> `TripCircuitBreaker`. Senao, realimenta Builders.
   v
[Gate 6: trustkit done -> trustkit merge / trustkit pr]
   |-- Renderiza `gauntlet-report.md` diretamente da tabela `execution_evidences` do SQLite
   |-- Faz commit das screenshots e do relatorio na branch
   |-- Publica comentario oficial na Issue (Nota de Negocios + Matriz RN->RF->TASK->E2E->Screenshot + `<img ...?raw=true>` + Resumo Tecnico)
   |-- Pergunta destino de merge ao usuario; apos merge em `develop`, move card para `Await Staging`
   |-- Se usuario executar `trustkit pr`: abre PR (`Ref #<id>`, revisores e assignees configurados) e move card para `In review`
```

### 6.3. Modelagem do Banco de Dados Embarcado Livre de `NULL` (Visão do DBA — SQLite WAL)

Aplicando o Capítulo 7 de *Clean Code* (Proibição de `null` e uso de *Special Case Values*) diretamente ao schema relacional em `.trustkit/state.db`, **100% das colunas possuem restrição `NOT NULL`** e valores explícitos de Caso Especial:

```sql
-- Inicializacao Obrigatoria de Concorrencia e Integridade
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

-- 1. Tabela de Ciclos SDD (100% NOT NULL com Special Case Values)
CREATE TABLE IF NOT EXISTS sdd_cycles (
    cycle_id                TEXT PRIMARY KEY NOT NULL,      -- ex: 'issue-45-202610271230' ou 'no-issue-202610271230'
    issue_linkage_kind      TEXT NOT NULL CHECK (issue_linkage_kind IN ('LINKED_ISSUE', 'UNLINKED_LOCAL_CYCLE')),
    issue_number            INTEGER NOT NULL DEFAULT 0,     -- 0 quando issue_linkage_kind = 'UNLINKED_LOCAL_CYCLE'
    issue_url               TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_board_id        TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_item_id         TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    board_column            TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    methodology_profile     TEXT NOT NULL CHECK (methodology_profile IN ('STANDARD_GAUNTLET_V2', 'SPECKIT_INTEGRATED')),
    current_gate            INTEGER NOT NULL DEFAULT 0 CHECK (current_gate BETWEEN 0 AND 6),
    lifecycle_status        TEXT NOT NULL CHECK (lifecycle_status IN ('ACTIVE', 'PAUSED', 'CIRCUIT_BREAKER_TRIPPED', 'CERTIFIED', 'MERGED')),
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0 CHECK (gauntlet_attempt BETWEEN 0 AND 5),
    spec_dir                TEXT NOT NULL,                  -- ex: 'docs/specs/issue-45-202610271230'
    spec_file_path          TEXT NOT NULL,
    spec_lock_state         TEXT NOT NULL DEFAULT 'DRAFT_UNVERIFIED' CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN')),
    spec_blake3_digest      TEXT NOT NULL DEFAULT 'UNSEALED_DRAFT',
    work_branch             TEXT NOT NULL,                  -- ex: 'feat/45-validacao-unicidade'
    base_commit_sha         TEXT NOT NULL,
    pre_existing_dirty_json TEXT NOT NULL DEFAULT '[]',     -- Lista vazia '[]' como Caso Especial
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 2. Tabela de Selos Criptograficos (Spec & Anti-Cheat de Testes Pre-existentes)
CREATE TABLE IF NOT EXISTS artifact_seals (
    seal_id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    file_path               TEXT NOT NULL,
    artifact_category       TEXT NOT NULL CHECK (artifact_category IN ('SPEC', 'CONSTITUTION', 'PRE_EXISTING_TEST', 'FOREIGN_ISSUE_EVIDENCE')),
    blake3_digest           TEXT NOT NULL,
    ast_assertion_count     INTEGER NOT NULL DEFAULT 0,     -- Contagem de nos AST expect/assert no Gate 2
    contract_unlock_state   TEXT NOT NULL DEFAULT 'SEALED' CHECK (contract_unlock_state IN ('SEALED', 'UNLOCKED_BY_HUMAN')),
    unlock_justification    TEXT NOT NULL DEFAULT 'NONE',
    UNIQUE(cycle_id, file_path)
);

-- 3. Matriz de Rastreabilidade Ponta a Ponta (RN -> RF -> TASK -> E2E -> Screenshot)
CREATE TABLE IF NOT EXISTS traceability_links (
    trace_id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    business_rule_id        TEXT NOT NULL,                  -- ex: 'RN-001'
    functional_req_id       TEXT NOT NULL,                  -- ex: 'RF-001'
    task_id                 TEXT NOT NULL,                  -- ex: 'TASK-001'
    mermaid_journey_node    TEXT NOT NULL,                  -- ex: 'D[Estado de Sucesso Confirmado]'
    playwright_spec_path    TEXT NOT NULL,                  -- ex: 'loggiq/tests/e2e/issue-45.spec.ts'
    evidence_screenshot     TEXT NOT NULL,                  -- ex: '01-cadastro-sucesso.png'
    verification_state      TEXT NOT NULL DEFAULT 'PENDING_GATE5' CHECK (verification_state IN ('PENDING_GATE5', 'VERIFIED_PASS'))
);

-- 4. Tarefas Atomicas e Fiscalizacao TDD (Red -> Green -> Refactor)
CREATE TABLE IF NOT EXISTS atomic_tasks (
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    task_id                 TEXT NOT NULL,                  -- ex: 'TASK-001'
    title                   TEXT NOT NULL,
    architectural_layer     TEXT NOT NULL,
    wave_index              INTEGER NOT NULL CHECK (wave_index >= 1),
    depends_on_json         TEXT NOT NULL DEFAULT '[]',     -- Lista vazia '[]' quando sem dependencias
    expected_files_json     TEXT NOT NULL DEFAULT '[]',
    target_test_file        TEXT NOT NULL,
    tdd_lifecycle_state     TEXT NOT NULL DEFAULT 'AWAITING_RED_PHASE' CHECK (tdd_lifecycle_state IN ('AWAITING_RED_PHASE', 'RED_VERIFIED', 'GREEN_VERIFIED', 'COMMITTED')),
    red_evidence_id         INTEGER NOT NULL DEFAULT 0,     -- 0 quando ainda em AWAITING_RED_PHASE
    green_evidence_id       INTEGER NOT NULL DEFAULT 0,     -- 0 ate GREEN_VERIFIED
    worktree_allocation     TEXT NOT NULL DEFAULT 'NOT_PROVISIONED',
    commit_sha              TEXT NOT NULL DEFAULT 'UNCOMMITTED',
    PRIMARY KEY (cycle_id, task_id)
);

-- 5. Evidencias de Execucao Imutaveis (Append-Only Audit Trail: "O log de eventos prova")
CREATE TABLE IF NOT EXISTS execution_evidences (
    evidence_id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    gate_stage              INTEGER NOT NULL,
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0,
    actor_role              TEXT NOT NULL,                  -- 'ORCHESTRATOR', 'BUILDER:TASK-001', 'AUDITOR:AST', 'CRITIC:DOMAIN', etc.
    evidence_kind           TEXT NOT NULL,                  -- 'CMD_EXEC', 'AST_AUDIT', 'BOARD_SYNC', 'SPEC_SEAL', 'CRITIC_VERDICT'
    executed_command        TEXT NOT NULL DEFAULT 'INTERNAL_AST_EVALUATION',
    exit_code               INTEGER NOT NULL DEFAULT 0,
    duration_ms             INTEGER NOT NULL,
    sanitized_stdout_tail   TEXT NOT NULL DEFAULT '',
    sanitized_stderr_tail   TEXT NOT NULL DEFAULT '',
    output_blake3_digest    TEXT NOT NULL,
    verdict                 TEXT NOT NULL CHECK (verdict IN ('PASS', 'FAIL', 'INFO', 'BLOCKED')),
    recorded_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_evidences_cycle_gate ON execution_evidences(cycle_id, gate_stage, verdict);
CREATE INDEX IF NOT EXISTS idx_traceability_cycle ON traceability_links(cycle_id, business_rule_id);
```

### 6.4. Estrutura de Pastas do Workspace Rust (`TRustKit`)

Seguindo a diretriz de Código Limpo (nomes que revelam intenção, sem sufixos `Manager`/`Processor` e arquivos modulares com menos de 250 linhas):

```text
trustkit/
|-- Cargo.toml                        # Workspace Cargo (resolver = "2", LTO thin, strip = true)
|-- crates/
|   |-- trustkit-cli/                 # Camada de Borda CLI/MCP & Composition Root (< 3ms cold-start)
|   |   |-- src/
|   |       |-- main.rs               # Entrypoint enxuto: aciona bootstrap e CliBoundaryErrorHandler
|   |       |-- bootstrap.rs          # Composition Root unico (Injecao de Dependencia dos Adapters)
|   |       |-- error_handler.rs      # Global Handler: traduz DomainViolation -> ExitCode + JSON/ASCII
|   |       |-- cli_contract.rs       # Definicao declarativa `clap` v4 (--json global flag)
|   |       |-- presenter.rs          # Formatador Dual livre de `null`: JSON compacto vs ASCII Tabelado
|   |       |-- mcp_stdio_server.rs   # Servidor MCP JSON-RPC sobre stdio
|   |       `-- commands/             # 1 arquivo por subcomando (init, spec, wave, gauntlet, done, pr, hook)
|   |
|   |-- trustkit-core/                # Dominio Puro (Zero crates externas, 100% PORS, Zero `null`)
|   |   |-- src/
|   |       |-- lib.rs
|   |       |-- entities.rs           # SddCycle, ArtifactSeal, TraceabilityLink, AtomicTask, ExecutionEvidence
|   |       |-- special_cases.rs      # IssueLinkage, SpecLockState, TddLifecycleState, AstAuditVerdict
|   |       |-- ports.rs              # Contratos de Limite (Traits): Repository, AstAuditor, Worktree, Board
|   |       |-- errors.rs             # Hierarquia rica `DomainViolation` (respeitando OCP)
|   |       |-- gates_fsm.rs          # Typestate Machine (Gate0Discovery..Gate6Certification)
|   |       |-- invariants_policy.rs  # Regras de negocio das 16 Hard Invariants (RN-001..RN-016)
|   |       `-- use_cases/            # Servicos de aplicacao puros recebendo `&dyn Port`
|   |
|   |-- trustkit-ast/                 # Adapters de AST (`tree-sitter`) e Topologia (`petgraph`)
|   |   |-- src/
|   |       |-- lib.rs
|   |       |-- tree_sitter_adapter.rs# Implementa `SyntaxTreeAuditorPort` + Boundary Error Translation
|   |       |-- test_integrity.rs     # VerifyTestIntegrity: detecta remocao de expect/assert e .skip
|   |       |-- lexical_scanner.rs    # InvariantLexicalScanner: bloqueia Emojis, Co-Authored-By, raw.github
|   |       |-- spec_validator.rs     # Valida Mermaid (`flowchart LR`), Gherkin e Matriz de Rastreabilidade
|   |       |-- prd_ingestor.rs       # PrdSpecificationIngestor (`--from-prd`)
|   |       |-- context_slicer.rs     # AstContextSlicer: extrai assinaturas publicas para Builders
|   |       `-- petgraph_adapter.rs   # Implementa `WaveTopologyPlannerPort`
|   |
|   |-- trustkit-db/                  # Adapters de Persistencia SQLite WAL & Hashing BLAKE3
|   |   |-- src/
|   |       |-- lib.rs
|   |       |-- sqlite_adapter.rs     # Implementa `CycleEvidenceRepositoryPort` (zero NULL)
|   |       |-- migrations.rs         # DDL embutido e pragmas WAL
|   |       `-- blake3_adapter.rs     # Implementa `ArtifactHasherPort` com paralelismo `rayon`
|   |
|   |-- trustkit-orch/                # Adapters de Isolamento de Worktrees, TDD Guard & Critic Swarm
|   |   |-- src/
|   |       |-- lib.rs
|   |       |-- worktree_adapter.rs   # Implementa `WorktreeIsolationPort` (`gix`/`git2` + symlinks CoW)
|   |       |-- tdd_phase_guard.rs    # Executor supervisionado RED (fail) -> GREEN (pass)
|   |       |-- swarm_dispatcher.rs   # Dispatcher assincrono `tokio` para Builders e 4 Critics headless
|   |       `-- evidence_restorer.rs  # RestoreForeignEvidence: protege screenshots de outras issues
|   |
|   `-- trustkit-gh/                  # Adapters do GitHub Projects v2, Issues e PRs
|       |-- src/
|           |-- lib.rs
|           |-- github_adapter.rs     # Implementa `ProjectBoardGatewayPort` + Boundary Error Translation
|           |-- identity_guard.rs     # GitHubIdentityGuard: garante conta ativa no gh
|           |-- board_synchronizer.rs # Mutacoes GraphQL das 9 Colunas Oficiais (sem regressao silenciosa)
|           `-- report_renderer.rs    # Renderizador deterministico do comentario de issue e PR
|
`-- skill/
    `-- trustkit-sdd/
        `-- SKILL.md                  # Thin Skill (< 90 linhas) que instrui o agente a usar o binario `trustkit`
```

### 6.5. Contrato de Subcomandos CLI do `trustkit`

| Comando CLI | Gate | Caso de Uso / Port Acionado pelo Binário Rust |
| :--- | :---: | :--- |
| `trustkit init [url\|id] [--json]` | **Gate 0** | Aciona `InitializeCycleUseCase` $\rightarrow$ `ProjectBoardGatewayPort` (conta ativa + `In progress`), `WorktreeIsolationPort` (isola `PreExistingDirtyFiles`) e grava `SddCycle` no `CycleEvidenceRepositoryPort`. |
| `trustkit spec init [--from-prd <p>]` | **Gate 0a/1** | Aciona `PrdSpecificationIngestor` para gerar `<topico>-spec.md` (ou `gauntlet-rubric.md` no perfil `SpecKitIntegrated`). |
| `trustkit spec lock [--json]` | **Gate 1 & 2** | Aciona `SealSpecificationUseCase` $\rightarrow$ valida Mermaid `flowchart LR`, Gherkin e Matriz $\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E}$ via `SyntaxTreeAuditorPort` e grava `ArtifactSeal` via `ArtifactHasherPort`. |
| `trustkit wave plan [--json]` | **Gate 3** | Aciona `PlanExecutionWavesUseCase` $\rightarrow$ `WaveTopologyPlannerPort` constrói o DAG, previne colisão de arquivos e retorna `WaveSchedule`. |
| `trustkit wave run --wave <N>` | **Gate 4** | Aciona `DispatchWaveUseCase` $\rightarrow$ provisiona worktrees via `WorktreeIsolationPort`, fatia contexto via `AstContextSlicer` e executa Builders. |
| `trustkit task test <TASK-ID> --phase <red\|green>` | **Gate 4** | Aciona `TddPhaseGuard`, grava `ExecutionEvidence` no SQLite e transiciona `TddLifecycleState` (`RedVerified` $\rightarrow$ `GreenVerified`). |
| `trustkit gauntlet ast-audit [--json]` | **Gate 5 (L0)** | Aciona `VerifyTestIntegrity` e `InvariantLexicalScanner` em `< 5ms` (Camada 0 Zero-Token). |
| `trustkit gauntlet run [--json]` | **Gate 5 (L0+L1)** | Executa Camada 0 + Camada 1 (Lint/Build, Playwright E2E com `RestoreForeignEvidence` e 4 Críticos) + controle do disjuntor (`TripCircuitBreaker` no 5º loop). |
| `trustkit done [--json]` | **Gate 6** | Renderiza `gauntlet-report.md` a partir de `execution_evidences`, commita evidências, publica comentário na issue com Matriz de Rastreabilidade e pergunta destino de merge. |
| `trustkit merge --target <branch>` | **Gate 6+** | Verifica `CycleStatus::Certified`, realiza merge e move card para `Await Staging` via `ProjectBoardGatewayPort`. |
| `trustkit pr [--json]` | **Sob Demanda** | Abre PR para `develop` (revisores e assignees, `Ref #<id>`, Matriz e imagens HTML) e move card para `In review`. |
| `trustkit hook <pre-tool\|post-tool>` | **Contínuo** | Rota síncrona `< 3ms` que bloqueia comandos proibidos (`RN-006`, `RN-012`, `RN-013`). |

---

## 7. Plano de Implementação em Fases

### Fase 1: Domínio Puro (`trustkit-core`), SQLite Adapter (`trustkit-db`) & Fast Hooks (Semana 1)
* Implementar todas as entidades PORS, Casos Especiais (zero `null`), `DomainViolation`, `gates_fsm` e os 6 traits em `trustkit-core::ports` (com 100% de cobertura unitária F.I.R.S.T. usando Fake Adapters em memória).
* Implementar `SqliteWalRepositoryAdapter` e `Blake3ParallelSealerAdapter` em `trustkit-db`.
* Implementar o Composition Root (`bootstrap.rs`), o `CliBoundaryErrorHandler`, `trustkit hook pre-tool` (`< 3ms`) e `trustkit init` com `GitHubCliGraphQlAdapter`.

### Fase 2: Motor de Especificação V2, `--from-prd`, Spec Seal & Topology Adapter (Semana 2)
* Implementar `PrdSpecificationIngestor` e o validador de Mermaid/Gherkin/Matriz de Rastreabilidade em `trustkit-ast`.
* Implementar `SealSpecificationUseCase` (`trustkit spec lock`) e `PetgraphTopologyAdapter` (`trustkit wave plan`).

### Fase 3: Camada 0 do Gauntlet (`TreeSitterAstAdapter`) & `AstContextSlicer` (Semana 3)
* Integrar `tree-sitter` em `trustkit-ast` para `TestIntegrityAuditor` (contagem e comparação de nós de asserção `expect`/`assert` antes e depois do diff).
* Implementar `InvariantLexicalScanner` (zero emojis Unicode, zero `Co-Authored-By`, zero `raw.githubusercontent.com`, zero `@ts-ignore`/`any` novo) e `AstContextSlicer`.

### Fase 4: Isolamento de Worktrees, TDD Guard, Gauntlet Camada 1 & Thin Skill (Semana 4)
* Implementar `GitWorktreeReflinkAdapter` (criação/poda de worktrees em `.trustkit/worktrees/` com symlinks de `node_modules`) e `TddPhaseGuard` (`RED -> GREEN`).
* Implementar `trustkit gauntlet run` (Camada 0 + Camada 1 com `RestoreForeignEvidence` e `TripCircuitBreaker`), `trustkit done`, `trustkit merge`, `trustkit pr`, servidor `trustkit mcp` e a nova **Thin Skill** (`SKILL.md` com menos de 90 linhas).

---

## 8. Métricas de Sucesso para esta Arquitetura

| Métrica Arquitetural | Alvo | Como Medir |
| :--- | :--- | :--- |
| **Pureza do Domínio (`trustkit-core`)** | `0` crates de I/O ou terceiros na árvore do `trustkit-core` | Auditoria automatizada com `cargo tree -p trustkit-core` no CI. |
| **Execução da Suíte Unitária F.I.R.S.T. (`trustkit-core`)** | `< 50ms` para toda a suíte de domínio | `cargo test -p trustkit-core`. |
| **Latência de Cold-Start (`trustkit hook` / `status`)** | `< 3ms` (p99) | Benchmark automatizado com `hyperfine --warmup 10 'trustkit status --json'`. |
| **Latência da Camada 0 (`trustkit gauntlet ast-audit`)** | `< 10ms` (p99) | Coluna `duration_ms` registrada na tabela `execution_evidences` do `.trustkit/state.db`. |
| **Ausência Total de `null` nos Contratos** | `0` ocorrências de `: null` nos JSONs de saída e `0` colunas nullable no SQLite | Testes de contrato validando o schema JSON de saída e inspeção de `PRAGMA table_info()` no SQLite. |

---

## 9. Alternativas Consideradas e Rejeitadas

### 9.1. Manter Apenas a Skill Declarativa em Markdown (`SKILL.md` + Bash Scripts)
**Rejeitado porque:**
* Scripts Bash encadeando `jq`, `sed`, `grep` e chamadas sequenciais `gh` não possuem parser AST real (incapazes de detectar *test dilution* com precisão), violam as 4 Regras de Projeto Simples (duplicação de regras e baixa testabilidade F.I.R.S.T.) e não oferecem transações ACID para o Event Log.

### 9.2. Construir o Harness em Python (`asyncio` + `pydantic`) ou TypeScript (`Bun` / `Node.js`)
**Rejeitado porque:**
* Mesmo em runtimes otimizados, o tempo de inicialização e o consumo de memória em invocações repetidas de hooks (`PreToolUse`) são de uma a duas ordens de grandeza maiores que um binário estático em Rust. Além disso, a integração nativa com `tree-sitter`, `gix` e `BLAKE3` em Rust opera sem sobrecarga de *bindings* dinâmicos.

### 9.3. Acoplar `rusqlite` e `tree-sitter` Diretamente nos Casos de Uso sem Ports (Sem Arquitetura Hexagonal)
**Rejeitado porque:**
* Violaria o Capítulo 8 (*Isolamento de Limites*) e o Capítulo 11 (*Separação entre Construção e Execução*) de *Clean Code*, obrigando todos os testes unitários da máquina de estados dos 7 Gates a subirem banco SQLite e arquivos reais em disco, degradando a velocidade e o isolamento F.I.R.S.T. da suíte de testes.

---

## 10. Próximos Passos

1. **Revisar e Aprovar este PRD v1.1 e ADR v1.1:** Confirmar os contratos dos 6 *Ports* (`trustkit-core::ports`) e dos Objetos de Caso Especial.
2. **Scaffolding do Workspace Rust (`Fase 1`):** Inicializar o repositório `Cargo.toml` em `/home/mmc/00_code/00_tools/20261027-novo-harness` começando pelo domínio puro `trustkit-core` e seus testes unitários F.I.R.S.T.
3. **Decomposição em Tarefas Atômicas:** Utilizar o fluxo de atomização para quebrar a Fase 1 do `TRustKit` em tarefas `TASK-001..TASK-NNN` orientadas a TDD em Rust (`cargo test`).

---

## 11. Referências

* Martin, Robert C. *Clean Code: A Handbook of Agile Software Craftsmanship* (Capítulos 1, 2, 7, 8, 9, 11 e 12).
* Especificação Base V2: `/home/mmc/00_code/00_tools/_skills/mmc-gauntlet-sdd-v2/SKILL.md`
* Especificação Base V1: `/home/mmc/00_code/00_tools/_skills/mmc-gauntlet-sdd/SKILL.md`
* Racional de Nomenclatura: `/home/mmc/00_code/00_tools/20261027-novo-harness/docs/name-defense.md`
* Documentação do Produto: `/home/mmc/00_code/00_tools/20261027-novo-harness/PRD_TRustKit.md`
