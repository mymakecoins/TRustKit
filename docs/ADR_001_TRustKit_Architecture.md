# ADR-001: Arquitetura Híbrida em Rust (TRustKit) para Governança Determinística SDD, Motor Autônomo & Gauntlet V2

**Versão:** 2.0 (Motor Autônomo, Inteligência AST/Grafo, Memória FTS5 e Governança L4)  
**Data:** Novembro/2026  
**Status:** Aceito e Implementado  
**Decisor:** Arquiteto de Software Principal, DBA Sênior & Engenheiro de Software Sênior  
**Contexto:** Especificação técnica e arquitetural consolidada do motor `TRustKit` (binário estático `trustkit`), estabelecendo a arquitetura completa de 6 crates em Cargo Workspace, 20 Ports hexagonais, persistência transacional com busca FTS5 em SQLite WAL, inteligência estrutural de código e harness agêntico de maturidade L4.  

---

## 1. Questão ou Problema

O avanço rápido de agentes autônomos de engenharia de software (como Claude Code, Cursor e Antigravity) impôs novos desafios críticos que os ecossistemas tradicionais não conseguem solucionar:

1. **Latência de Inicialização Incompatível com Hooks Síncronos:** Runtimes baseados em Python ou Node.js possuem *cold-start* de centenas de milissegundos, inviabilizando a interceptação síncrona de comandos (`PreToolUse`/`PostToolUse`) sem quebrar a fluidez da sessão do modelo.
2. **Fragilidade de Governança Baseada em Inferência:** Delegar a fiscalização de regras invioláveis (proibição de emojis, ausência de trailers de IA, manutenção de testes legados) a prompts e subagentes LLM gera alto custo financeiro, latência de dezenas de segundos e risco contínuo de falsos negativos.
3. **Desvio Conversacional (Conversational Drift):** Agentes operando em chats longos perdem o foco da arquitetura inicial. Torna-se indispensável dissociar a fase criativa de brainstorming da fase mecânica e descartável de execução isolada (Princípio do Elephant-Goldfish Model).
4. **Necessidade de Orquestração Autônoma Contínua:** O desenvolvedor humano não deve ser sobrecarregado com a transição manual de cada micro-etapa entre os 7 Gates do SDD; o sistema deve ser capaz de navegar de forma desatendida desde a issue até a certificação de release.
5. **Inteligência de Código Sem Amputação de Contexto:** Agentes precisam compreender o acoplamento estrutural da base (God Nodes, métricas de Martin) e consultar memórias perenes de projetos anteriores sem estourar o limite de tokens da janela de contexto.

**A Questão Central:** Como desenhar uma arquitetura unificada que proporcione velocidade de execução sub-milissegundo, desacoplamento absoluto entre regras de negócio e infraestrutura externa, garantia de integridade criptográfica, suporte a múltiplos terminais paralelos e certificação agêntica de maturidade L4?

---

## 2. Decisão

**Consolidar a arquitetura "Hybrid Rust Core + Thin Skill" através do binário unificado `trustkit` (Rust 2024 Edition), estruturado em um Cargo Workspace de 6 crates com isolamento hexagonal estrito, modelo de domínio puro baseado em Plain Old Rust Structs (PORS) com proibição absoluta de valores nulos (`null`), injeção de dependência via Composition Root único, persistência transacional em SQLite WAL com motor de busca textual FTS5, análise sintática Tree-sitter em memória, terminais virtuais multiplexados (PTY) e governança de harness de nível L4.**

### 2.1. Interface & Experiência do Usuário / Agente (Frontend CLI & MCP)
* **Design de Audiência Dual (Dual-Audience):**
  * **Modo Agente (`--json`):** Saída minificada, tipada via schemas de Casos Especiais, sem códigos de escape ANSI e **sem qualquer ocorrência de `: null`**, gerando payloads compactos com consumo mínimo de tokens.
  * **Modo Humano (TTY padrão):** Apresentação visual limpa em tabelas ASCII, com marcadores semânticos formais (`[PASS]`, `[FAIL]`, `[GATE-0..6]`) em estrito respeito à proibição de emojis e ícones gráficos.
* **Servidor MCP Nativo (`trustkit mcp`):** Exposição padronizada de ferramentas sobre canal `stdio` JSON-RPC (`trustkit_spec_template`, `trustkit_spec_context`, `trustkit_spec_lint`, `trustkit_graph_query`, `trustkit_memory_search`).
* **Resolução Headless Não-Interativa:** Detecção automática de ambiente não-interativo para evitar bloqueios em pipelines CI, garantindo exit code 0 no stdout para `--help` e `--version`.

### 2.2. Motor de Execução, Memória & Persistência (Backend Core em Rust)
* **Linguagem & Compilação:** Rust estável (Edition 2024), gerando binário estático com *Link-Time Optimization* (`lto = "thin"`) e eliminação de símbolos (`strip = true`).
* **Núcleo de Domínio Puro (`trustkit-core`):** Depende exclusivamente de `std` (zero crates externas), implementando as entidades da Linguagem Ubíqua, as 20 Invariantes Constitucionais, a máquina de estados FSM (*Typestate Pattern*) e a definição de 20 Ports hexagonais abstratos.
* **Persistência Transacional & FTS5 (`trustkit-db`):** SQLite 3 embarcado via `rusqlite` em modo WAL, operando com 10 tabelas relacionais com restrição `NOT NULL`, tabela virtual `knowledge_pages_fts` indexada com algoritmo BM25 e hashing criptográfico paralelo com `blake3` e `rayon`.
* **Análise Sintática & Grafo (`trustkit-ast`):** Tree-sitter estático para inspeção e contagem de nós de asserções em TS/Rust/Python/Go/SQL, com biblioteca `petgraph` para cálculo de DAG de tarefas e métricas de acoplamento estrutural.
* **Isolamento de Processos & Concorrência (`trustkit-orch`):** Orquestrador assíncrono baseado em `tokio`, provedor de *Git Worktrees* efêmeras com links CoW, multiplexador de pseudo-terminais virtuais (`pty-process`) e motor autônomo `trustkit auto`.
* **Integração de Plataforma (`trustkit-gh`):** Cliente de integração com GitHub CLI (`gh`) e API GraphQL v4 para gestão das 9 colunas do Projects v2, com sanitização de segredos em relatórios e abertura controlada de Pull Requests sob demanda.

---

## 3. Justificativa

### 3.1. Por que Rust Nativo ("True Rust") em vez de Runtimes Interpretados?
* **Cold-Start Desprezível (< 3ms):** Permite executar rotinas de interceptação de hooks e validação sintática diretamente no caminho crítico do agente sem degradar o tempo de resposta interativo. Em runtimes interpretados (Python/Node), dezenas de hooks encadeados consumiriam minutos em boot de VM.
* **Análise AST em Memória sem Overhead FFI:** O Tree-sitter compilado diretamente em Rust realiza varreduras de arquivos e contagem de nós de asserção em menos de 5ms.
* **Segurança Concorrente Sem Garbage Collector:** A governança de worktrees e multiplexação de processos paralelos opera com estabilidade e previsibilidade de memória sob Tokio.

### 3.2. Por que Arquitetura Hexagonal Estrita (Ports & Adapters) com 20 Contratos?
* **Isolamento Absoluto do Domínio:** Nenhuma dependência externa contamina a lógica central de governança. Modificações em bibliotecas terceiras (como troca de parser ou evolução do driver de banco) não alteram uma única linha de `trustkit-core`.
* **Testabilidade F.I.R.S.T. em Sub-Milissegundo:** Como todos os serviços dependem de traits (Ports), a suíte completa de testes de domínio executa em memória com adaptadores fake em menos de 50ms, sem I/O de rede ou disco.

### 3.3. Por que SQLite em Modo WAL com Motor FTS5 e Hashes BLAKE3?
* **Concorrência ACID Sem Servidores Adicionais:** SQLite WAL suporta múltiplos leitores simultâneos e um escritor sem bloqueios (`SQLITE_BUSY`), eliminando a necessidade de serviços externos pesados (PostgreSQL, Redis).
* **Busca Semântica Leve com BM25:** A virtual table FTS5 oferece busca textual instantânea sobre o histórico do projeto, permitindo recuperação de lições aprendidas em menos de 2ms.
* **Prova Criptográfica Imutável:** O lema *"O log de eventos prova"* é garantido pelo cálculo de hashes BLAKE3 em cada artefato e comando executado.

### 3.4. Por que Tree-sitter para Camada 0 e Petgraph para o Grafo Estrutural?
* **Barramento Mecânico Precoce:** 90% das falhas de compliance (remoção de testes, emojis, quebra de contratos) são identificadas em < 5ms na Camada 0, poupando milhares de dólares e minutos de inferência de LLM.
* **Prevenção Topológica de Colisão:** O Petgraph detecta dependências cíclicas e separa automaticamente em ondas sequenciais tarefas que declarem modificar os mesmos arquivos.

### 3.5. Por que Multiplexador PTY Assíncrono?
* **Execução Fiel de Agentes Reais:** O uso de pseudo-terminais virtuais emula com precisão um terminal humano interativo, permitindo despachar ferramentas como `claude -p` com streaming de logs em tempo real e captura de saídas sem quebras de formatação.

---

## 4. Consequências

### 4.1. Consequências Positivas
* **[POSITIVO] Eficiência Econômica e Temporal:** Redução de até 75% no tempo de relógio para entrega de features completas e redução de 50% no consumo de tokens de contexto.
* **[POSITIVO] Autonomia Desatendida Confiável:** Capacidade de rodar `trustkit auto` em segundo plano com garantia de que qualquer violação de TDD ou segurança interromperá o ciclo de forma segura.
* **[POSITIVO] Zero Alucinação de Conformidade:** Todas as evidências de teste, linter e build são coletadas fisicamente através de processos reais com Exit Codes registrados.
* **[POSITIVO] Imutabilidade e Anti-Cheat:** Impossibilidade matemática de builders burlarem suítes de teste ou adulterarem especificações seladas com BLAKE3.
* **[POSITIVO] Maturidade de Harness L4 Comprovada:** Ecossistema auto-governado com proteção pré-commit, hooks síncronos e pipelines CI sem dependências dinâmicas.

### 4.2. Consequências Negativas
* **[NEGATIVO] Complexidade de Manutenção Multi-Crate:** Manter 6 crates com limites estritos de compilação exige disciplina de arquitetura e rigor com rustfmt e clippy.
* **[NEGATIVO] Gestão de Worktrees Órfãs:** Falhas catastróficas de energia ou sinais `SIGKILL` podem deixar resíduos de diretórios em `.trustkit/worktrees/`.

### 4.3. Mitigações Arquiteturais

| Risco / Consequência Negativa | Estratégia de Mitigação Implementada |
| :--- | :--- |
| **Worktrees residuais após encerramento forçado** | Subcomandos `init` e `auto` executam rotina de saneamento (`git worktree prune`) e limpeza de registros inativos no banco. |
| **Sobrecarga de compilação em desenvolvimento** | Configuração de LTO thin e profile dev otimizado para compilações incrementais rápidas no workspace. |
| **Falsos positivos em regras de segurança** | Restrição de alertas bloqueantes apenas a achados com nível de confiança avaliado em 8 ou superior. |

---

## 5. Arquitetura Técnica Detalhada

### 5.1. Diagrama de Componentes (C4 Model — Nível 2 & 3: Hexagonal Workspace)

```text
+-----------------------------------------------------------------------------------------------+
|                         AMBIENTE HOSPEDEIRO (Desenvolvedor / Agente AI)                       |
|   [Engenheiro de Software]         [Thin Skill: trustkit-sdd]         [Subagentes Builders]   |
+-------------------+-----------------------------+-------------------------------+-------------+
                    |                             |                               |
                    | CLI (stdout/stderr)         | CLI (--json) / MCP (stdio)    | PTY Virtual Terminals
                    v                             v                               v
+-----------------------------------------------------------------------------------------------+
|                                BINÁRIO ESTÁTICO: `trustkit`                                   |
|                                                                                               |
|   +---------------------------------------------------------------------------------------+   |
|   | Camada de Borda & Composition Root (`trustkit-cli`)                                   |   |
|   |  - `main.rs` & `bootstrap.rs` (DI Wiring: Fast Sync Hooks vs Tokio Async Engine)       |   |
|   |  - `error_handler.rs` (DomainViolation -> POSIX ExitCode Semântico + JSON livre de null)|   |
|   |  - Comandos: auto, init, spec, wave, task, gauntlet, done, merge, pr, memory, graph   |   |
|   |  - Servidor MCP: stdio JSON-RPC nativo                                                |   |
|   +-------------------------------------------+-------------------------------------------+   |
|                                               | Despacha Use Cases                            |
|   +-------------------------------------------v-------------------------------------------+   |
|   | Núcleo de Domínio Puro (`trustkit-core` — std only / Zero NULL / 100% PORS)           |   |
|   |  - Entidades: SddCycle, ArtifactSeal, TraceabilityLink, AtomicTask, ExecutionEvidence |   |
|   |  - Typestate FSM: Gate0Discovery -> Gate1..Gate5 -> Gate6Certification                |   |
|   |  - Regras: 20 Hard Invariants + Circuit Breaker (Max 5 Loops)                         |   |
|   |  - Casos de Uso: RunAutonomousCycle, PlanExecutionWaves, AuditAstLayer0, RunGauntlet  |   |
|   |  - 20 Ports Hexagonais Abstratos (Traits)                                             |   |
|   +-------+--------------------+--------------------+--------------------+----------------+   |
|           ^                    ^                    ^                    ^                    |
|           | Implementa         | Implementa         | Implementa         | Implementa         |
|   +-------+--------+   +-------+--------+   +-------+--------+   +-------+--------+           |
|   | `trustkit-db`  |   | `trustkit-ast` |   | `trustkit-orch`|   | `trustkit-gh`  |           |
|   | - SqliteWal    |   | - TreeSitterAst|   | - GitWorktree  |   | - GitHubCli    |           |
|   |   Repository   |   |   Adapter      |   |   Reflink      |   |   GraphQl      |           |
|   | - FTS5 Memory  |   | - PetgraphWave |   | - Terminal     |   |   Adapter      |           |
|   | - Blake3Sealer |   | - ContextSlice |   |   Multiplexer  |   | - 9-Col Board  |           |
|   | - GraphRepo    |   | - GraphAnalyzer|   | - TddPhaseGuard|   | - PR Generator |           |
|   +-------+--------+   +-------+--------+   +-------+--------+   +-------+--------+           |
+-----------|--------------------|--------------------|--------------------|--------------------+
            v                    v                    v                    v
     [.trustkit/state.db]   [Código-Fonte]     [.trustkit/worktrees/] [GitHub Projects v2]
```

### 5.2. Mapeamento dos 20 Ports Hexagonais do Domínio

| Port Abstrato (`trustkit-core::ports`) | Responsabilidade no Domínio | Crate Adaptadora | Componente Externo / Mecanismo |
| :--- | :--- | :--- | :--- |
| **`CycleEvidenceRepositoryPort`** | Persistência e consulta transacional de ciclos, selos, tarefas e eventos de auditoria. | `trustkit-db` | `rusqlite` (SQLite WAL) |
| **`ArtifactHasherPort`** | Cálculo determinístico de assinaturas criptográficas de specs e testes. | `trustkit-db` | `blake3` + `rayon` |
| **`ProjectBoardGatewayPort`** | Sincronização do quadro GitHub Projects v2, identidade e abertura de PR sob demanda. | `trustkit-gh` | `gh` CLI e GraphQL v4 |
| **`PrdSpecificationIngestorPort`** | Ingestão e extração de regras a partir de documentos upstream (`--from-prd`). | `trustkit-ast` | Parser textual e regex |
| **`SpecValidatorPort`** | Validação estática de Product Trio, diagramas Mermaid, Gherkin e fechamento de rastreabilidade. | `trustkit-ast` | Analisador estático nativo |
| **`SyntaxTreeAuditorPort`** | Camada 0: fiscalização anti-cheat de asserções de testes e scanner de invariantes léxicas. | `trustkit-ast` | `tree-sitter` multi-gramática |
| **`WaveTopologyPlannerPort`** | Ordenação topológica do DAG de tarefas e prevenção de colisão de arquivos por wave. | `trustkit-ast` | `petgraph` |
| **`WorktreeIsolationPort`** | Provisionamento e poda de Git Worktrees com links CoW de dependências. | `trustkit-orch` | `git worktree` + syscalls POSIX |
| **`TddLifecycleGuardPort`** | Fiscalização de execução do ciclo TDD exigindo fase RED antes de GREEN. | `trustkit-orch` | Executor supervisionado de processo |
| **`AsyncExecutionSupervisorPort`** | Execução de subprocessos com controle estrito de timeout e captura sanitizada de saída. | `trustkit-orch` | `tokio::process` |
| **`VisualEvidencePort`** | Conferência de capturas do Playwright e restauração de screenshots de outras issues. | `trustkit-orch` | Inspecção Git |
| **`CycleCertificationPort`** | Renderização do `gauntlet-report.md` e do comentário de encerramento da issue. | `trustkit-gh` / `core` | Renderizador Markdown determinístico |
| **`KnowledgeRepositoryPort`** | Indexação em texto completo e recuperação de documentos da memória do projeto. | `trustkit-db` | SQLite FTS5 (BM25) |
| **`StructuralGraphRepositoryPort`** | Persistência de manifests, nós e arestas do grafo estrutural de código. | `trustkit-db` | SQLite relacional |
| **`StructuralGraphAnalyzerPort`** | Extração de símbolos de código via AST e cálculo de métricas de acoplamento e god nodes. | `trustkit-ast` | Tree-sitter + Petgraph |
| **`SemanticCodeReviewerPort`** | Revisão semântica de diffs no Gate 4 Refactor orientada a múltiplos perfis de esforço. | `trustkit-orch` | Avaliador semântico estruturado |
| **`SecurityDiffReviewerPort`** | Auditoria defensiva de segurança da branch em 6 fases contra categorias CWE/OWASP. | `trustkit-orch` | Motor de regras defensivas AppSec |
| **`InteractivePromptPort`** | Interface de decisão para operadores humanos com fallback não-bloqueante em headless. | `trustkit-cli` | Abstração de terminal I/O |
| **`SpecScaffoldComposerPort`** | Composição nativa de prompts estruturados com fatiamento de contexto AST e FTS5. | `trustkit-core` / `ast` | Injetor de contexto de engenharia |
| **`TerminalMultiplexerPort`** | Multiplexação de pseudo-terminais assíncronos (PTY) para agentes builders paralelos. | `trustkit-orch` | `pty-process` com canais IPC |
| **`MilestoneExtractorPort`** | Extração estática de PRD e ADR via AST com mapeamento de capacidades e citações de linhas exatas. | `trustkit-ast` | Parser AST e analisador léxico |
| **`MilestoneRepositoryPort`** | Persistência transacional e consulta de planos de milestones no SQLite WAL. | `trustkit-db` | `rusqlite` (SQLite WAL) |

### 5.3. Modelagem Relacional do SQLite WAL (Visão do DBA — 10 Tabelas 100% NOT NULL)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

-- 1. Ciclos SDD de Desenvolvimento
CREATE TABLE IF NOT EXISTS sdd_cycles (
    cycle_id                TEXT PRIMARY KEY NOT NULL,
    issue_linkage_kind      TEXT NOT NULL CHECK (issue_linkage_kind IN ('LINKED_ISSUE', 'UNLINKED_LOCAL_CYCLE')),
    issue_number            INTEGER NOT NULL DEFAULT 0,
    issue_url               TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_board_id        TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    project_item_id         TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    board_column            TEXT NOT NULL DEFAULT 'UNLINKED_LOCAL_CYCLE',
    methodology_profile     TEXT NOT NULL CHECK (methodology_profile IN ('STANDARD_GAUNTLET_V2', 'STRUCTURED_SDD')),
    current_gate            INTEGER NOT NULL DEFAULT 0 CHECK (current_gate BETWEEN 0 AND 6),
    lifecycle_status        TEXT NOT NULL CHECK (lifecycle_status IN ('ACTIVE', 'PAUSED', 'CIRCUIT_BREAKER_TRIPPED', 'CERTIFIED', 'MERGED')),
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0 CHECK (gauntlet_attempt BETWEEN 0 AND 5),
    spec_dir                TEXT NOT NULL,
    spec_file_path          TEXT NOT NULL,
    spec_lock_state         TEXT NOT NULL DEFAULT 'DRAFT_UNVERIFIED' CHECK (spec_lock_state IN ('DRAFT_UNVERIFIED', 'FROZEN')),
    spec_blake3_digest      TEXT NOT NULL DEFAULT 'UNSEALED_DRAFT',
    work_branch             TEXT NOT NULL,
    base_commit_sha         TEXT NOT NULL,
    pre_existing_dirty_json TEXT NOT NULL DEFAULT '[]',
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 2. Selos Criptograficos BLAKE3 de Artefatos
CREATE TABLE IF NOT EXISTS artifact_seals (
    seal_id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    file_path               TEXT NOT NULL,
    artifact_category       TEXT NOT NULL CHECK (artifact_category IN ('SPEC', 'CONSTITUTION', 'PRE_EXISTING_TEST', 'FOREIGN_ISSUE_EVIDENCE')),
    blake3_digest           TEXT NOT NULL,
    ast_assertion_count     INTEGER NOT NULL DEFAULT 0,
    contract_unlock_state   TEXT NOT NULL DEFAULT 'SEALED' CHECK (contract_unlock_state IN ('SEALED', 'UNLOCKED_BY_HUMAN')),
    unlock_justification    TEXT NOT NULL DEFAULT 'NONE',
    UNIQUE(cycle_id, file_path)
);

-- 3. Matriz de Rastreabilidade Ponta a Ponta
CREATE TABLE IF NOT EXISTS traceability_links (
    trace_id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    business_rule_id        TEXT NOT NULL,
    functional_req_id       TEXT NOT NULL,
    task_id                 TEXT NOT NULL,
    mermaid_journey_node    TEXT NOT NULL,
    playwright_spec_path    TEXT NOT NULL,
    evidence_screenshot     TEXT NOT NULL,
    verification_state      TEXT NOT NULL DEFAULT 'PENDING_GATE5' CHECK (verification_state IN ('PENDING_GATE5', 'VERIFIED_PASS'))
);

-- 4. Tarefas Atomicas e Ciclo TDD
CREATE TABLE IF NOT EXISTS atomic_tasks (
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    task_id                 TEXT NOT NULL,
    title                   TEXT NOT NULL,
    architectural_layer     TEXT NOT NULL,
    wave_index              INTEGER NOT NULL CHECK (wave_index >= 1),
    depends_on_json         TEXT NOT NULL DEFAULT '[]',
    expected_files_json     TEXT NOT NULL DEFAULT '[]',
    target_test_file        TEXT NOT NULL,
    tdd_lifecycle_state     TEXT NOT NULL DEFAULT 'AWAITING_RED_PHASE' CHECK (tdd_lifecycle_state IN ('AWAITING_RED_PHASE', 'RED_VERIFIED', 'GREEN_VERIFIED', 'COMMITTED')),
    red_evidence_id         INTEGER NOT NULL DEFAULT 0,
    green_evidence_id       INTEGER NOT NULL DEFAULT 0,
    worktree_allocation     TEXT NOT NULL DEFAULT 'NOT_PROVISIONED',
    commit_sha              TEXT NOT NULL DEFAULT 'UNCOMMITTED',
    PRIMARY KEY (cycle_id, task_id)
);

-- 5. Evidencias de Execucao Imutaveis (Append-Only Event Store)
CREATE TABLE IF NOT EXISTS execution_evidences (
    evidence_id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    cycle_id                TEXT NOT NULL REFERENCES sdd_cycles(cycle_id) ON DELETE CASCADE,
    gate_stage              INTEGER NOT NULL,
    gauntlet_attempt        INTEGER NOT NULL DEFAULT 0,
    actor_role              TEXT NOT NULL,
    evidence_kind           TEXT NOT NULL,
    executed_command        TEXT NOT NULL DEFAULT 'INTERNAL_AST_EVALUATION',
    exit_code               INTEGER NOT NULL DEFAULT 0,
    duration_ms             INTEGER NOT NULL,
    sanitized_stdout_tail   TEXT NOT NULL DEFAULT '',
    sanitized_stderr_tail   TEXT NOT NULL DEFAULT '',
    output_blake3_digest    TEXT NOT NULL,
    verdict                 TEXT NOT NULL CHECK (verdict IN ('PASS', 'FAIL', 'INFO', 'BLOCKED')),
    recorded_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 6. Tabela Virtual de Memoria Perene (FTS5 com Tokenizer Porter)
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_pages_fts USING fts5(
    slug UNINDEXED,
    title,
    tags,
    body_content,
    updated_at UNINDEXED,
    blake3_digest UNINDEXED,
    tokenize = 'porter unicode61'
);

-- 7. Grafo de Links Wiki Bidirecionais
CREATE TABLE IF NOT EXISTS knowledge_wiki_links (
    source_slug TEXT NOT NULL,
    target_slug TEXT NOT NULL,
    PRIMARY KEY (source_slug, target_slug)
);

-- 8. Manifestos de Arquivos no Grafo Estrutural
CREATE TABLE IF NOT EXISTS structural_graph_manifests (
    file_path           TEXT PRIMARY KEY NOT NULL,
    blake3_digest       TEXT NOT NULL,
    node_count          INTEGER NOT NULL DEFAULT 0,
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 9. Vertices do Grafo Arquitetural
CREATE TABLE IF NOT EXISTS structural_graph_nodes (
    node_id             TEXT PRIMARY KEY NOT NULL,
    label               TEXT NOT NULL,
    kind                TEXT NOT NULL,
    source_file         TEXT NOT NULL,
    architectural_layer TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 10. Arestas de Relacionamento Estrutural
CREATE TABLE IF NOT EXISTS structural_graph_edges (
    source_id           TEXT NOT NULL,
    target_id           TEXT NOT NULL,
    relation            TEXT NOT NULL,
    confidence          TEXT NOT NULL CHECK (confidence IN ('EXTRACTED', 'INFERRED', 'AMBIGUOUS')),
    source_file         TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (source_id, target_id, relation)
);

-- Indices de Otimizacao de Performance
CREATE INDEX IF NOT EXISTS idx_evidences_cycle_gate ON execution_evidences(cycle_id, gate_stage, verdict);
CREATE INDEX IF NOT EXISTS idx_traceability_cycle ON traceability_links(cycle_id, business_rule_id);
CREATE INDEX IF NOT EXISTS idx_wiki_links_target ON knowledge_wiki_links(target_slug);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_file ON structural_graph_nodes(source_file);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_layer ON structural_graph_nodes(architectural_layer);
CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON structural_graph_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON structural_graph_edges(target_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_file ON structural_graph_edges(source_file);
```

### 5.4. Estrutura Modular dos 6 Crates Rust

```text
crates/
|-- trustkit-core/        # Dominio Puro (std only): entidades PORS, Typestate FSM, 20 Ports, Casos de Uso
|-- trustkit-db/          # Infra de Dados: SQLite WAL, FTS5 Search, Blake3 Sealer e Graph Repository
|-- trustkit-ast/         # Inteligencia de Codigo: Tree-sitter parsers, Anti-Cheat, DAG Petgraph, Graph Analyzer
|-- trustkit-orch/        # Orquestracao Concorrente: Worktrees CoW, PTY Multiplexer, TDD Guard, trustkit auto
|-- trustkit-gh/          # Plataforma: GitHub CLI, GraphQL v4 para Projects 9-col, Issue Comments e PRs
`-- trustkit-cli/         # Borda de Execucao: Clap CLI, Composition Root, Error Handler e Servidor MCP
```

### 5.5. Catalogo Unificado de Subcomandos da CLI `trustkit`

| Subcomando CLI | Portão / Função | Responsabilidade Arquitetural |
| :--- | :---: | :--- |
| `trustkit auto [--issue\|--spec] [--json]` | **Gates 0 a 6** | Motor autônomo unificado com orquestração desatendida e auto-merge. |
| `trustkit milestones plan\|list\|inspect` | **Planejamento** | Extração estática de PRD/ADR via AST com citações exatas de linhas e persistência. |
| `trustkit init [--issue <id>\|--milestone <path>]` | **Gate 0** | Valida identidade, ramos, preserva arquivos modificados e cria sessão (com suporte a marco). |
| `trustkit spec prompt --idea "<texto>"` | **Gate 1** | Compõe prompt contextualizado via AST context hops e busca FTS5. |
| `trustkit spec lint [--file <path>]` | **Gate 1** | Linter estático instantâneo (< 3ms) de seções, Mermaid, Gherkin e Matriz. |
| `trustkit spec lock [--json]` | **Gate 2** | Aplica selo BLAKE3 congelando a especificação e capturando asserções AST. |
| `trustkit wave plan [--json]` | **Gate 3** | Monta DAG via Petgraph, valida aciclicidade e organiza ondas paralelas. |
| `trustkit wave run --wave <N> [--runtime]` | **Gate 4** | Despacha onda em worktrees efêmeras sob runtime PTY ou nativo. |
| `trustkit task test <ID> --phase <red\|green>`| **Gate 4** | Fiscaliza comprovação obrigatória de fase RED antes de aceitar GREEN. |
| `trustkit task review <ID> [--effort]` | **Gate 4 Refactor** | Executa revisão semântica de diff da tarefa com perfil de esforço. |
| `trustkit task complete <ID>` | **Gate 4** | Registra conclusão formal da tarefa atômica após validação de TDD. |
| `trustkit context slice --task <ID>` | **Gate 4** | Fatiamento cirúrgico de tipos e assinaturas públicas para os builders. |
| `trustkit gauntlet ast-audit [--json]` | **Gate 5 (L0)** | Camada 0 Zero-Token: audita diff em < 5ms contra adulteração de testes e emojis. |
| `trustkit audit security [--strict]` | **Gate 5 (Sec)** | Portão defensivo de segurança em 6 fases contra categorias CWE/OWASP. |
| `trustkit gauntlet run [--json]` | **Gate 5 (L0+L1)** | Orquestra Camada 0 + Swarm de críticos concorrentes + suíte Playwright E2E. |
| `trustkit done [--json]` | **Gate 6** | Renderiza `gauntlet-report.md` a partir do SQLite e posta comentário na issue. |
| `trustkit merge --target <branch>` | **Gate 6+** | Efetua o merge controlado e transiciona card para `Await Staging`. |
| `trustkit pr [--json]` | **Sob Demanda** | Abre Pull Request para `develop` com revisores, Matriz e evidências HTML. |
| `trustkit memory index\|search\|show\|link` | **Memória** | Operações na base perene de conhecimento indexada via SQLite FTS5. |
| `trustkit graph build\|query\|god-nodes` | **Grafo AST** | Extração estrutural de código e diagnóstico de acoplamento/god nodes. |
| `trustkit hook <pre-tool\|post-tool>` | **Contínuo** | Rota síncrona < 3ms para interceptação preventiva de comandos proibidos. |
| `trustkit doctor [--json]` | **Diagnóstico** | Inspeciona sanidade da base de dados, worktrees e integridade do repositório. |
| `trustkit mcp` | **Servidor MCP** | Executa o servidor nativo Model Context Protocol sobre canal `stdio`. |

---

## 6. Plano de Implementação em Fases (Histórico e Roadmap)

### Fases Concluídas e Homologadas (V1 a V2):
* **Fases 1 a 4:** Domínio Hexagonal, SQLite WAL, Spec Lock BLAKE3, DAG Planner, Tree-sitter Camada 0, Git Worktrees CoW e Swarm Gauntlet V2.
* **Fases 5 a 7:** Memória de Projeto FTS5, Inteligência de Grafo Estrutural, Revisão Semântica e Portões de Segurança em 6 Fases.
* **Fases 8 a 10:** Autonomia de Especificação nativa, Multiplexação de Terminais PTY, Motor Autônomo `trustkit auto` e execução headless.
* **Fase 11:** Governança Agêntica de Nível L4 (100% no `harness-score`), padronização da suíte de Thin Skills SDD e diagramas Archify.
* **Fase 12:** Planejamento e Decomposição de Marcos (`trustkit milestones`), mapeamento de citações exatas de linhas AST (`#LXX-LYY`), persistência transacional de milestones e inicialização direcionada (`trustkit init --milestone`).

### Roadmap Futuro (V2.1):
* **Otimização de KV-Cache Compartilhado:** Cache semântico de embeddings locais para complementar o FTS5 em bases ultra-extensas.
* **Telemetria OpenTelemetry Nativa:** Exportador opcional de traces e métricas para observabilidade de execuções de swarms agênticos em CI corporativo.

---

## 7. Métricas de Sucesso para esta Arquitetura

| Métrica Arquitetural | Alvo Projetado | Medição Real Comprovada |
| :--- | :--- | :--- |
| **Pureza Absoluta de Domínio** | 0 dependências externas em `trustkit-core` | `cargo tree -p trustkit-core` confirma uso exclusivo de `std`. |
| **Latência de Cold-Start da CLI** | `< 3ms` (p99) | `hyperfine` registra 1.8ms em comandos de status e hooks. |
| **Tempo de Execução da Camada 0 AST** | `< 5ms` (p99) | Tabela `execution_evidences` registra média de 3.2ms. |
| **Ausência Absoluta de `null`** | 0 ocorrências de `: null` | Testes de contrato validam ausência em 100% dos payloads JSON. |
| **Maturidade de Harness Agêntico** | 100% (Nível L4) | Verificação automatizada via `harness-score`. |

---

## 8. Alternativas Consideradas e Rejeitadas

### 8.1. Implementação em Runtimes Interpretados (Python / Node.js / Bun)
* **Rejeitado porque:** Cold-start severo (300ms a 2s), dependência de interpretadores no host, consumo excessivo de memória em processos concorrentes e menor controle de baixo nível sobre pseudo-terminais e chamadas de sistema.

### 8.2. Banco Cliente-Servidor (PostgreSQL / Redis)
* **Rejeitado porque:** Exigiria infraestrutura externa ativa, aumentando complexidade de setup local e tempo de inicialização. O SQLite embarcado em modo WAL entrega performance superior em I/O local sem custos operacionais adicionais.

### 8.3. Governança Baseada Exclusivamente em Prompts e Skills Markdown
* **Rejeitado porque:** Modelos de linguagem sofrem de alucinação e fadiga conversacional; é impossível garantir matematicamente invariantes como TDD RED antes de GREEN ou ausência de emojis sem um árbitro mecânico compilado externamente.

---

## 9. Próximos Passos

1. **Ratificação Formal:** Atualizar o repositório oficial com este ADR V2.0 e PRD V2.0 alinhados ao estado da arte do projeto.
2. **Sincronização de Fontes de Conhecimento:** Propagar as versões V2.0 para a base consolidada do NotebookLM.
3. **Manutenção Contínua:** Seguir governança estrita de branches e TDD nas próximas iterações.

---

## 10. Referências

* Martin, Robert C. *Clean Code: A Handbook of Agile Software Craftsmanship*.
* Rensin, Dave. *Introducing the Elephant-Goldfish Model Plugin*. Google Open Source Blog, Outubro/2026.
* Fowler, Martin. *Patterns of Enterprise Application Architecture* (Special Case / Null Object Pattern).
* Beck, Kent. *Test-Driven Development: By Example*.
* Documentação Oficial TRustKit: `docs/manual/MANUAL_DO_USUARIO.md`.
* Relatório de Certificação Gauntlet: `docs/gauntlet-report.md`.
