# PRD/ERS: TRustKit — Motor Determinístico em Rust para Spec-Driven Development & Gauntlet Loop

**Versão:** 2.0 (Motor Autônomo, Inteligência AST/Grafo, Memória FTS5 e Governança L4)  
**Data:** Novembro/2026  
**Autor:** Equipe Multidisciplinar de Produto & Engenharia (Analista de Requisitos + Gerente de Projetos + UX/UI CLI + QA Sênior + Arquiteto de Software)  
**Status:** Implementado e Operacional em Produção  

---

## 1. Visão do Produto

### 1.1. Propósito
Fornecer um motor determinístico de altíssimo desempenho compilado nativamente em Rust (`trustkit`) que governa o ciclo de vida completo do desenvolvimento guiado por especificações (Spec-Driven Development — SDD) e a suíte adversarial multi-agente Gauntlet Loop V2, combinando orquestração autônoma ponta a ponta (`trustkit auto`), inteligência sintática via AST (`tree-sitter`) em microssegundos, isolamento físico de agentes em Git Worktrees com terminais multiplexados (PTY), memória perene indexada em SQLite FTS5 e governança de harness agêntico de maturidade máxima (Nível L4).

### 1.2. Nome Oficial e Identidade
* **Nome do Produto:** `TRustKit`
* **Binário CLI:** `trustkit`
* **Slogan:** *"True Rust performance for trusted, spec-driven agentic execution."*
* **Paradigma Central:** *Hybrid Rust Core + Thin Skill* (Skill declarativa ultraleve no contexto do agente + Binário estático nativo para mutações e auditoria).

### 1.3. Problema que Resolve
1. **Latência Excessiva e Cold-Start de Runtimes Interpretados:** Harnesses em Python ou Node.js impõem de 300ms a 2.000ms de *cold-start* a cada invocação de subprocesso ou hook (`PreToolUse`/`PostToolUse`), degradando a experiência e invalidando o *Prompt Cache* (KV-Cache) do LLM.
2. **Fragilidade Probabilística de Skills Exclusivamente Declarativas:** Skills baseadas apenas em prompts Markdown dependem da obediência volátil do modelo, permitindo que agentes pulem etapas do TDD, enfraqueçam asserções de testes (*test dilution*), alucinem conformidade na Matriz de Rastreabilidade ou insiram artefatos proibidos (emojis, trailers de IA).
3. **Desperdício Massivo de Tokens em Auditorias Mecânicas:** Gastar turnos inteiros de inferência de LLM (10s a 40s) para validar sintaxe Mermaid, blocos Gherkin, remoção de `expect()` ou linting estático consome orçamento e desacelera o time-to-delivery.
4. **Colisão de Recursos em Execuções Paralelas:** Quando múltiplos subagentes de implementação (*Builders*) operam concorrentemente na mesma árvore de arquivos, ocorrem condições de corrida no índice do Git, corrupção de caches de dependências e sobrescrita de evidências E2E.
5. **Degradação Conversacional (Conversational Drift):** Sessões longas de chat provocam perda de atenção e desvios de arquitetura. O TRustKit resolve esse problema separando estruturalmente a formulação (The Elephant) da execução e revisão fria descartável (The Goldfish).
6. **Ausência de Rastreabilidade Criptográfica e Event Sourcing:** Textos e relatórios convencionais podem ser editados pelo próprio agente gerador. O TRustKit impõe a verdade matemática com assinaturas BLAKE3 e log transacional imutável em SQLite WAL.

### 1.4. Público-Alvo
* **Engenheiros de Software Sênior & Arquitetos AI-First:** Desenvolvedores que operam fluxos de SDD em repositórios corporativos de alta criticidade e exigem velocidade máxima sem abdicar de governança estrita.
* **Agentes Codificadores Autônomos (Claude Code, Antigravity, Codex):** Consumidores primários da interface CLI em modo `--json` e do servidor nativo MCP, que delegam ao binário `trustkit` toda validação mecânica, gestão de worktrees e sincronização de boards.
* **Tech Leaders & Revisores de Arquitetura:** Stakeholders que auditam Pull Requests, relatórios de certificação (`gauntlet-report.md`) e árvores de rastreabilidade ponta a ponta ($	ext{RN} ightarrow 	ext{RF} ightarrow 	ext{TASK} ightarrow 	ext{E2E} ightarrow 	ext{Screenshot}$).
* **Especialistas em Segurança de Aplicação (AppSec):** Auditores que utilizam a suíte pré-release de segurança em 6 fases para barrar vulnerabilidades CWE/OWASP antes da integração contínua.

### 1.5. Proposta de Valor
O TRustKit entrega uma infraestrutura mecatrônica e determinística em Rust nativo:
* **Tempo de Inicialização Sub-3ms:** Hooks síncronos e verificações de linter executam sem latência perceptível no terminal.
* **Auditoria AST Camada 0 em < 5ms:** Barramento instantâneo de trapaças em testes, violações léxicas e quebra de escopo antes de acionar turnos caros de LLM.
* **Orquestração 100% Autônoma (`trustkit auto`):** Capacidade de receber uma issue remota ou arquivo local e transitar sozinho por todos os 7 gates, despachando waves e executando TDD rigoroso até a certificação final.
* **Memória Perene de Projeto e Grafo Estrutural:** Recuperação imediata de contexto histórico via FTS5 BM25 e análise topológica de acoplamento diretamente no binário.

---

## 2. Contexto Atual

### 2.1. Processo Atual
Historicamente, o desenvolvimento de software assistido por IA oscilava entre a lentidão de frameworks pesados (LangGraph, CrewAI) e a fragilidade de prompts em Markdown. Com o TRustKit V1, os 7 Gates foram consolidados em um binário estático. Agora, no TRustKit V2, o ecossistema evoluiu para maturidade agêntica de nível L4: suporte a execução autônoma headless, independência completa de runtimes externos para especificação, inteligência de grafo de código e memória semântica perene.

### 2.2. Principais Dores e Impactos

| Dor Identificada | Impacto no Processo | Como o TRustKit V2 Elimina |
| :--- | :--- | :--- |
| **Interrupções manuais frequentes entre Gates** | Desenvolvedor precisa digitar comandos intermediários (`wave run`, `task test`, `gauntlet run`). | Comando unificado `trustkit auto` executa os 7 gates em sequência contínua com circuit breaker defensivo. |
| **Dependência de ecossistemas externos de prompt/spec** | Necessidade de frameworks externos para gerar esqueletos de especificações. | Subcomandos nativos `trustkit spec prompt` e `trustkit spec lint` com fatiamento de contexto AST e FTS5. |
| **Amnésia entre sessões de desenvolvimento** | Agentes recém-instanciados repetem erros de arquitetura ou esquecem lições aprendidas. | Módulo `trustkit memory` indexa lições aprendidas, incidentes e specs com busca textual FTS5 BM25 e links wiki. |
| **Acoplamento oculto e God Nodes no código** | Tarefas paralelas modificam módulos hiper-conectados, gerando conflitos de merge e quebras em cascata. | Módulo `trustkit graph` mapeia dependências estruturais, calcula métricas de acoplamento (Ca/Ce, Instabilidade) e alerta god nodes. |
| **Vulnerabilidades de segurança introduzidas por IA** | Injeções SQL, caminhos inseguros e permissões frouxas passam despercebidos por linters convencionais. | Módulo `trustkit audit security` audita diffs em 6 fases de segurança defensiva com filtro de confiança >= 8/10. |
| **Travamento de processos interativos em CI/Headless** | Prompts de terminal bloqueiam aguardando stdin quando executados em pipelines ou subprocessos de agentes. | Tratamento estrito de exit codes (help/version com status 0 no stdout) e resolução não-interativa por política em modo headless. |

### 2.3. Oportunidades
* **Governança Agêntica L4 Autocertificada:** Garantir nota máxima (100%) em avaliações de harness agêntico (`harness-score`), unindo regras de contexto, interceptores síncronos e verificação contínua em CI.
* **Execução em Swarm com Terminais PTY Reais:** Permitir que agentes builders operem em pseudo-terminais isolados com streaming de logs, socket IPC e controle de timeout individual.
* **Auditoria de Segurança Integrada ao Ciclo de Release:** Bloquear a promoção de branches caso a análise de segurança identifique riscos de severidade alta/crítica.

---

## 3. Usuários e Personas

### 3.1. Persona 1: Engenheiro de Software Sênior / Arquiteto AI-First
* **Idade:** 36 anos | **Experiência:** 14+ anos em sistemas distribuídos e engenharia de software.
* **Objetivo:** Orquestrar agentes autônomos para construir features complexas mantendo TDD estrito e zero degradação de arquitetura.
* **Dor Principal:** Ter que policiar manualmente agentes que esquecem de rodar testes em falha (RED) ou alteram testes existentes para forçar aprovação.
* **Necessidade:** Um motor mecânico que bloqueie mutações inválidas em tempo de compilação e exija comprovação matemática de cada fase do ciclo.

### 3.2. Persona 2: Agente Orquestrador Hospedeiro (Claude Code, Antigravity, Cursor)
* **Idade:** N/A (Agente de IA) | **Experiência:** Opera com janelas de contexto limitadas e sensíveis a ruído.
* **Objetivo:** Percorrer a jornada dos 7 gates com economia máxima de tokens, sem alucinar passos e sem ser bloqueado por interfaces interativas.
* **Dor Principal:** Receber stacktraces gigantescos ou payloads JSON contendo campos `null` e dados redundantes que poluem o KV-Cache.
* **Necessidade:** Modos de saída puramente determinísticos (`--json`), códigos de saída POSIX semânticos e ferramentas MCP de alta granularidade.

### 3.3. Persona 3: Tech Leader & Revisor de Governança
* **Idade:** 38 anos | **Experiência:** 15 anos em liderança técnica e qualidade corporativa.
* **Objetivo:** Aprovar Pull Requests com velocidade sabendo que as Regras de Negócio e contratos foram 100% verificados.
* **Dor Principal:** Diffs densos gerados por IA onde a intenção do requisito original se perdeu na implementação.
* **Necessidade:** Relatórios oficiais imutáveis (`gauntlet-report.md`) gerados diretamente do banco de eventos, com links auditáveis para cada evidência.

### 3.4. Persona 4: Especialista em Segurança de Aplicação (AppSec)
* **Idade:** 34 anos | **Experiência:** 10 anos em testes de invasão e auditoria de código seguro.
* **Objetivo:** Impedir que códigos gerados por IA introduzam vulnerabilidades OWASP ou quebrem o princípio do menor privilégio.
* **Dor Principal:** Linters de segurança geram excesso de falsos positivos ou exigem varreduras lentas fora da esteira do desenvolvedor.
* **Necessidade:** Portões de segurança nativos rápidos que avaliem o diff da branch contra vetores conhecidos de ataque com alta precisão.

---

## 4. Jornada do Usuário

### 4.1. Jornada Manual Supervisionada (7 Gates Tradicionais)
```text
1. Inicialização (Gate 0):
   trustkit init --issue 45 [--from-prd docs/PRD.md] --json
   -> Valida ambiente, branches, conta GitHub e registra ciclo no SQLite WAL em < 250ms.

2. Especificação e Linting (Gate 1):
   trustkit spec prompt --idea "Feature de Conciliação" --json
   trustkit spec lint --file specs/010-feature/spec.md --json
   -> Valida Product Trio, Gherkin, Mermaid e Matriz de Rastreabilidade.

3. Congelamento Criptográfico (Gate 2):
   trustkit spec lock --json
   -> Sela o arquivo com hash BLAKE3 e captura nós de asserção AST dos testes existentes.

4. Planejamento Topológico de Ondas (Gate 3):
   trustkit wave plan --json
   -> Petgraph constrói o DAG de tarefas atomicas, detecta ciclos e previne colisões de arquivos.

5. Execução em Worktrees e TDD Supervisionado (Gate 4):
   trustkit wave run --wave 1 --runtime pty --json
   trustkit task test TASK-001 --phase red --json
   trustkit task test TASK-001 --phase green --json
   trustkit task review TASK-001 --effort MEDIUM --json
   trustkit task complete TASK-001 --json

6. Certificação Gauntlet Loop V2 (Gate 5):
   trustkit gauntlet ast-audit --json      # Camada 0: AST nativa < 5ms (Zero Tokens)
   trustkit audit security --strict --json # Auditoria global de segurança em 6 fases
   trustkit gauntlet run --json            # Camada 1: Parallel Swarm de Críticos + Playwright

7. Finalização e Integração (Gate 6):
   trustkit done --json                    # Renderiza gauntlet-report.md e publica comentário
   trustkit merge --target develop         # Transiciona card para Await Staging
   trustkit pr --json                      # Opcional: abre PR exclusivamente sob demanda
```

### 4.2. Jornada Autônoma Ponta a Ponta (`trustkit auto`)
```text
1. Disparo Único pelo Desenvolvedor ou Agente:
   trustkit auto --issue 45 [--target develop] [--runtime auto] --json
   (ou via arquivo local: trustkit auto --spec docs/specs/010-feature/spec.md)

2. Execução Autônoma pelo Motor:
   - Inicializa sessão e valida árvore Git limpa (Gate 0).
   - Audita especificação via spec lint nativo (Gate 1).
   - Aplica selo BLAKE3 congelando a especificação (Gate 2).
   - Gera o grafo DAG e organiza as ondas de execução (Gate 3).
   - Para cada onda (1..N): provisiona worktrees, orquestra builders, exige prova RED antes de GREEN,
     executa revisão semântica de diff e faz merge na branch de trabalho (Gate 4).
   - Executa Camada 0 AST e dispara críticos do Gauntlet V2 (Gate 5).
     * Se falhar: realimenta os builders (até 5 tentativas; aciona Circuit Breaker na 5ª).
     * Se passar: prossegue para certificação.
   - Gera gauntlet-report.md imutável a partir do log SQLite e realiza o merge final (Gate 6).

3. Conclusão:
   Relatório emitido no terminal e branch pronta para homologação sem intervenção manual intermediária.
```

### 4.3. Diretrizes de UX/UI para CLI Ultraleve (Dual-Audience Design)
* **Modo Agente (`--json`):** Saída minificada, estritamente tipada, livre de códigos de escape ANSI, **livre de valores `null`** (uso de Objetos de Caso Especial e arrays vazios `[]`) e enriquecida em todos os comandos com o array determinístico `next_steps` (`priority`, `target_gate`, `command` e `description`), minimizando consumo de tokens e eliminando dúvidas de navegação.
* **Modo Humano (TTY Padrão):** Saída estruturada em tabelas ASCII, com status claros `[PASS]`, `[FAIL]`, `[WARN]`, `[GATE-N]` e rodapé padronizado `NEXT STEP:` ao final de cada comando e no `trustkit status`, sem emojis ou caracteres gráficos extravagantes, garantindo legibilidade corporativa.
* **Compatibilidade Headless:** Detecção automática de ausência de TTY (`atty::isnt`) para suprimir prompts interativos, adotando comportamentos seguros por política padrão.

---

## 5. Escopo do Produto (Funcionalidades Implementadas V1 & V2)

### 5.1. Itens no Escopo
#### 5.1.1. Core de Domínio, FSM de Gates & Repositório SQLite WAL (`trustkit-core` & `trustkit-db`)
* Máquina de estados determinística dos 7 Gates com padrão *Typestate* em Rust puro (`std` only).
* Banco embarcado SQLite 3 em modo WAL com 10 tabelas relacionais e 100% de colunas com restrição `NOT NULL`.
* Controle do disjuntor de segurança (*Circuit Breaker*) limitando loops de correção a no máximo 5 tentativas.
* Registro em log de eventos transacional (*Event Sourcing*) com hashes BLAKE3 de cada evidência de comando.

#### 5.1.2. Especificação Nativa, Spec Lint & Validador de DAG (`trustkit-spec` & `trustkit-ast`)
* Subcomando `trustkit spec prompt`: montagem nativa de prompts com retroalimentação de contexto via AST e busca FTS5.
* Subcomando `trustkit spec lint`: validador estático (< 3ms) de seções do Product Trio, sintaxe Mermaid, blocos Gherkin e integridade da Matriz de Rastreabilidade.
* Subcomando `trustkit spec lock`: congelamento criptográfico da especificação via selo BLAKE3.
* Validador topológico via `petgraph` com detecção de ciclos e agrupamento em ondas independentes.

#### 5.1.3. Auditoria Sintática AST Camada 0 & Context Slicer (`trustkit-ast`)
* Motor Tree-sitter estático (suportando Rust, TypeScript, TSX, Python, Go e SQL):
  * `TestIntegrityAuditor`: fiscaliza remoção de asserções (`expect`, `assert`, `assert_eq!`) e anotações `@ignore` / `.skip`.
  * `InvariantLexicalScanner`: bloqueia emojis, trailers de autoria IA (`Co-Authored-By`), URLs `raw.githubusercontent` e `any`/`@ts-ignore`.
  * `AstContextSlicer`: fatiamento cirúrgico de código extraindo assinaturas públicas e contratos para reduzir o consumo de tokens dos builders.

#### 5.1.4. Isolador de Git Worktrees & Multiplexador PTY (`trustkit-orch`)
* Provedor de worktrees efêmeras (`.trustkit/worktrees/TASK-NNN`) com compartilhamento reflink CoW e symlinks automáticos de pastas pesadas (`node_modules`, `target`).
* Multiplexador de terminais virtuais (`pty-process`) para despacho concorrente de subagentes com streaming assíncrono e controle de I/O em tempo real.
* Fiscalizador `TddPhaseGuard` exigindo prova física da fase RED (`Exit Code != 0`) antes de aceitar a fase GREEN (`Exit Code == 0`).

#### 5.1.5. Memória Perene de Projeto & Busca FTS5 (`trustkit-db` & `trustkit-cli`)
* Tabela virtual SQLite FTS5 (`knowledge_pages_fts`) com tokenizer Porter e BM25 ranking.
* Gestão de links wiki bidirecionais (`knowledge_wiki_links`) com resolução automática de referências `[[slug]]`.
* Subcomandos `trustkit memory index`, `trustkit memory search`, `trustkit memory show` e `trustkit memory link`.

#### 5.1.6. Inteligência de Grafo Estrutural (`trustkit-ast` & `trustkit-db`)
* Extração de nós e arestas arquiteturais a partir do código-fonte via Tree-sitter.
* Persistência do grafo relacional (`structural_graph_nodes`, `structural_graph_edges`, `structural_graph_manifests`).
* Análise de saúde topológica: identificação de *God Nodes* (grau de acoplamento excessivo), cálculo de acoplamento eferente/aferente (Ce/Ca) e instabilidade de Martin ($I = Ce / (Ca + Ce)$).
* Subcomandos `trustkit graph build`, `trustkit graph query`, `trustkit graph god-nodes` e `trustkit graph coupling`.

#### 5.1.7. Revisão Semântica & Portões de Segurança Pré-Release (`trustkit-orch` & `trustkit-cli`)
* Subcomando `trustkit task review`: auditoria semântica de diffs no Gate 4 Refactor com níveis de esforço (`LOW`, `MEDIUM`, `HIGH`).
* Subcomando `trustkit audit security`: auditoria defensiva em 6 fases com categorização CWE/OWASP e filtro de confiança de criticidade (threshold >= 8/10).

#### 5.1.8. Motor Autônomo de Ponta a Ponta (`trustkit auto`)
* Subcomando `trustkit auto`: orquestração completa e desatendida percorrendo os Gates 0 a 6.
* Suporte nativo a issues remotas do GitHub (`--issue <id|url>`) ou arquivos locais de especificação (`--spec <path>`).
* Execução headless à prova de bloqueios interativos e tratamento estrito de exit codes em chamadas informativas (`--help`, `--version` com código 0 no stdout).

#### 5.1.9. Sincronizador Git & GitHub Projects v2 (`trustkit-gh`)
* Verificação estrita de identidade ativa no GitHub (`gh auth status`).
* Transição de cards nas 9 colunas oficiais do GitHub Projects v2.
* Abertura de Pull Request exclusivamente sob demanda explícita do desenvolvedor (`trustkit pr`).

#### 5.1.10. Interceptador de Hooks de Ciclo de Vida (`trustkit hook`)
* Subcomandos `trustkit hook pre-tool` e `trustkit hook post-tool` com cold-start < 3ms para bloquear comandos destrutivos (`git stash`, `git reset --hard`, commits diretos em branches protegidas).

#### 5.1.11. Agentic Harness & Governança L4 (100% no harness-score)
* Conjunto integrado de regras e personas em `.claude/` (`rules/`, `agents/`, `commands/`).
* Scaffolding de contexto centralizado em `AGENTS.md`.
* Validações contínuas via `clippy.toml`, `.pre-commit-config.yaml` e `.github/workflows/ci.yml`.

#### 5.1.12. Planejamento de Marcos a partir de PRD e ADR (`trustkit milestones`)
* Subcomando `trustkit milestones plan`: extração estática combinada de PRD e ADR via AST, mapeamento de capacidades de negócio e restrições técnicas, decomposição em DAG acíclico, geração de artefatos Markdown individuais em `docs/milestones/MNN.md` com citação exata de linhas (`#LXX-LYY`) e persistência transacional em SQLite WAL.
* Subcomando `trustkit milestones list`: listagem ordenada de marcos, dependências topológicas e status.
* Subcomando `trustkit milestones inspect`: inspeção detalhada de metadados, capacidades e citações de um marco.
* Integração com `trustkit init --milestone`: inicialização direcionada de ciclos SDD a partir do marco planejado.

#### 5.1.13. Orientação Contextual Determinística de Próximo Passo (`NextStepResolver` & `NextStepPresenter`)
* Resolvedor puro de domínio (`NextStepResolver` em `trustkit-core`) que inspeciona o gate ativo, o estado de congelamento da spec, as tarefas pendentes na onda (`AwaitingRedPhase`, `RedVerified`, `GreenVerified`) e o estado do Circuit Breaker para recomendar exatamente o comando seguinte com argumentos reais preenchidos (ex.: `TASK-002`, `--wave 2`).
* Padronização em todos os comandos de ciclo e no `trustkit status` do array JSON `next_steps` (livre de `null`) e do rodapé ASCII `NEXT STEP:`.
* Protocolo obrigatório na skill declarativa `trustkit-sdd` (v4.2) para apresentar ao usuário o bloco `PROXIMO PASSO RECOMENDADO` ao final de cada turno.

### 5.2. Itens Fora do Escopo (V2)
* Interface gráfica TUI complexa (`ratatui`) ou Dashboard Web local — o foco permanece 100% na eficiência de terminal e MCP.
* Cobrança ou gestão direta de saldo/chaves de API de provedores de LLM.
* Criação de sub-issues no GitHub para tarefas atômicas (proibido pela Invariante Constitucional #15).
* Abertura automática de Pull Request ao término do ciclo (proibido pela Invariante Constitucional #12).

---

## 6. Requisitos Funcionais (Especificação BDD Executável)

| ID | Requisito de Domínio | Critérios de Aceite Executáveis (BDD — Dado / Quando / Então) |
| :--- | :--- | :--- |
| **RF-001** | **Inicialização Concorrente do Ciclo (`Gate 0`)** | **Dado** uma árvore de trabalho Git acessível, **quando** o comando `trustkit init --issue <ID>` for executado, **então** o binário valida a conta GitHub ativa, preserva arquivos modificados prévios na lista `PreExistingDirtyFiles` (sem executar `git stash`), move o card no GitHub Projects para `In progress`, cria o diretório `docs/specs/<issue-id>-AAAAMMDDHHMM/` e grava o registro `SddCycle` no SQLite WAL em < 250ms com Exit Code 0. |
| **RF-002** | **Caso Especial Sem Issue Vinculada (`Gate 0`)** | **Dado** que `trustkit init` foi invocado sem argumento de issue, **quando** o comando iniciar, **então** o binário instancia o Caso Especial `IssueLinkage::UnlinkedLocalCycle`, cria `docs/specs/no-issue-AAAAMMDDHHMM/` e opera localmente sem emitir mutações remotas no GitHub Projects. |
| **RF-003** | **Ingestão Upstream de PRD/ADR (`Gate 0a`)** | **Dado** um arquivo PRD ou ADR válido contendo tabelas de requisitos, **quando** `trustkit spec init --from-prd <path>` for acionado, **então** o motor extrai regras de negócio e requisitos funcionais e gera o esqueleto formal da especificação com as seções do Product Trio, Gherkin e Matriz de Rastreabilidade. |
| **RF-004** | **Geração de Prompt Contextualizado (`Gate 1`)** | **Dado** uma ideia ou requisito em linguagem natural, **quando** `trustkit spec prompt --idea "<ideia>"` for executado, **então** o motor recupera fatias AST de código relevante e artigos da memória FTS5, retornando um prompt estruturado dentro do orçamento de tokens solicitado. |
| **RF-005** | **Linting Estático da Especificação (`Gate 1`)** | **Dado** um arquivo `spec.md`, **quando** `trustkit spec lint --file <path>` for invocado, **então** o analisador verifica a presença do Product Trio, validade do Mermaid `flowchart LR`, blocos Gherkin e fechamento da Matriz de Rastreabilidade em < 5ms, retornando CleanPass ou lista tipada de diagnósticos. |
| **RF-006** | **Congelamento Criptográfico da Spec (`Gate 2`)** | **Dado** uma especificação aprovada pelo linter, **quando** `trustkit spec lock` for acionado, **então** o motor calcula o hash BLAKE3 da especificação e de todos os testes existentes, captura a contagem de asserções AST e persiste o estado `SpecLockState::Frozen` no SQLite WAL. |
| **RF-007** | **Planejamento de Ondas Topológicas (`Gate 3`)** | **Dado** um conjunto de tarefas atômicas `TASK-NNN.md`, **quando** `trustkit wave plan` for executado, **então** o algoritmo baseia-se em `petgraph` para validar aciclicidade (DAG), mapeia 100% das regras da Matriz e aloca em ondas distintas tarefas que declarem modificar os mesmos arquivos. |
| **RF-008** | **Provisionamento Isolado de Worktrees (`Gate 4`)** | **Dado** uma onda de execução contendo N tarefas, **quando** `trustkit wave run --wave <N>` for disparado, **então** o motor instancia N Git Worktrees efêmeras com isolamento de índice e links CoW de dependências pesadas, despachando builders em paralelo. |
| **RF-009** | **Supervisão do Ciclo TDD Estrito (`Gate 4`)** | **Dado** uma tarefa atômica em execução, **quando** o builder rodar `trustkit task test <ID> --phase red`, **então** o motor exige saída de falha comprovada (`Exit Code != 0`) antes de permitir a homologação de sucesso via `trustkit task test <ID> --phase green` (`Exit Code == 0`). |
| **RF-010** | **Revisão Semântica de Código (`Gate 4 Refactor`)** | **Dado** uma tarefa concluída na fase green, **quando** `trustkit task review <ID>` for acionado, **então** o motor inspeciona o diff em busca de regressões, violações de escopo e aderência arquitetural, emitindo parecer detalhado. |
| **RF-011** | **Auditoria AST Camada 0 do Gauntlet (`Gate 5`)** | **Dado** o diff da branch de trabalho antes dos críticos, **quando** `trustkit gauntlet ast-audit` for executado, **então** o motor valida via Tree-sitter em < 5ms a ausência de adulteração de testes (*test dilution*), emojis, trailers de IA e modificação de arquivos protegidos, bloqueando a execução antes de gastar tokens caso haja inconformidade. |
| **RF-012** | **Portão Defensivo Global de Segurança (`Gate 5`)** | **Dado** a solicitação de auditoria pré-release, **quando** `trustkit audit security` for disparado, **então** o motor executa varredura em 6 fases (AST, CWE/OWASP, injeção, segredos, permissões e dependências), reportando achados com score de confiança >= 8/10. |
| **RF-013** | **Swarm de Críticos e Evidências Playwright (`Gate 5`)** | **Dado** que a Camada 0 foi aprovada com CleanPass, **quando** `trustkit gauntlet run` for executado, **então** o motor orquestra em paralelo os 4 críticos adversariais, executa testes E2E Playwright, valida capturas de tela e restaura automaticamente qualquer screenshot alheia tocada acidentalmente. |
| **RF-014** | **Disjuntor de Segurança / Circuit Breaker (`Gate 5`)** | **Dado** que um ciclo acumulou 4 reprovações no Gauntlet, **quando** a 5ª iteração falhar, **então** o motor bloqueia novas execuções via `CycleStatus::CircuitBreakerTripped` e aciona o relatório de diagnóstico defensivo `trustkit doctor`. |
| **RF-015** | **Certificação Determinística de Release (`Gate 6`)** | **Dado** que todos os portões e críticos acumularam veredito PASS no SQLite, **quando** `trustkit done` for acionado, **então** o binário renderiza o `gauntlet-report.md` a partir do log transacional, realiza commit das evidências e publica comentário oficial na issue do GitHub. |
| **RF-016** | **Integração de Merge Controlada (`Gate 6+`)** | **Dado** um ciclo certificado, **quando** `trustkit merge --target develop` for executado com autorização do operador, **então** o motor realiza o merge na branch de destino e move o card no quadro para `Await Staging`. |
| **RF-017** | **Abertura de Pull Request Sob Demanda** | **Dado** solicitação explícita do desenvolvedor, **quando** `trustkit pr` for executado, **então** o motor abre o Pull Request com referência `Ref #<id>`, lista de revisores, matriz de rastreabilidade e imagens HTML, transicionando o card para `In review`. |
| **RF-018** | **Interceptação Preventiva via Hooks Síncronos** | **Dado** que um agente ou script tenta disparar comando proibido (`git stash`, `git reset --hard`, commit direto em branch protegida), **quando** o interceptor `trustkit hook pre-tool` analisar o comando, **então** o processo é abortado em < 3ms com código semântico e diagnóstico explicativo. |
| **RF-019** | **Indexação e Busca na Memória Perene (`trustkit memory`)** | **Dado** termos de pesquisa de arquitetura ou histórico, **quando** `trustkit memory search <query>` for acionado, **então** o motor consulta a tabela FTS5 com BM25 ranking e retorna trechos contextuais e links de wiki adjacentes. |
| **RF-020** | **Análise Estrutural e Detecção de God Nodes (`trustkit graph`)** | **Dado** uma base de código analisada pelo Tree-sitter, **quando** `trustkit graph god-nodes` for invocado, **então** o motor retorna os nós com maior acoplamento eferente/aferente e calcula a métrica de instabilidade para guiar a decomposição de tarefas. |
| **RF-021** | **Orquestração Autônoma Desatendida (`trustkit auto`)** | **Dado** um identificador de issue ou arquivo de spec local, **quando** o comando `trustkit auto` for acionado, **então** o motor executa de forma autônoma e contínua os Gates 0 a 6 com isolamento de worktree, TDD estrito e auto-merge final em caso de certificação unânime. |
| **RF-022** | **Planejamento de Marcos e Citações AST (`trustkit milestones`)** | **Dado** a presença de PRD e ADR no repositório, **quando** `trustkit milestones plan` for executado, **então** o motor decompõe os documentos em marcos acíclicos com citações de linha precisas, gera os arquivos `docs/milestones/` e persiste os registros na tabela transacional `milestones` do SQLite WAL, habilitando `trustkit init --milestone` para direcionar a execução do ciclo SDD. |
| **RF-022** | **Conformidade de Exit Codes e Execução Headless** | **Dado** um ambiente de automação sem terminal interativo (TTY), **quando** o binário for invocado com flags de ajuda (`--help`, `-h`) ou versão (`--version`, `-V`), **então** a saída informativa deve ser direcionada para `stdout` com código de saída 0, e rotinas de decisão devem resolver por política padrão sem bloquear aguardando `stdin`. |

---

## 7. Requisitos Não-Funcionais

| Atributo de Qualidade | Especificação Técnica do Requisito | Métrica Objetiva de Validação |
| :--- | :--- | :--- |
| **Latência de Cold-Start** | Inicialização ultrarrápida para hooks síncronos de ciclo de vida (`pre-tool`/`post-tool`). | `< 3ms` (p99) medido via `hyperfine`. |
| **Auditoria de AST Camada 0** | Varredura estática de integridade de testes e invariantes léxicas sobre o diff da branch. | `< 5ms` (p99) para diffs de até 5.000 linhas de código. |
| **Busca Textual na Memória** | Consulta em linguagem natural via SQLite FTS5 com ranqueamento BM25. | `< 2ms` para base de até 10.000 documentos de documentação/wiki. |
| **Análise Topológica do Grafo** | Construção do DAG de tarefas e cálculo de acoplamento estrutural via `petgraph`. | `< 10ms` para grafos de até 1.000 nós e dependências. |
| **Confiabilidade Transacional** | Integridade dos dados sob alta concorrência de múltiplos builders e críticos simultâneos. | SQLite em modo WAL com `busy_timeout = 5000ms`, `synchronous = NORMAL` e 100% de colunas com restrição `NOT NULL`. |
| **Portabilidade do Binário** | Distribuição de binário único totalmente estático sem dependências dinâmicas externas. | Compilação nativa para Linux (`x86_64`, `aarch64`) e macOS (`darwin`) com tamanho final stripado `< 30 MB`. |
| **Ergonomia e Consumo de Tokens** | Formatação das saídas no modo `--json` projetada especificamente para economizar contexto de LLM. | Ausência absoluta do literal `: null` (100% Casos Especiais), payloads de sucesso `< 300 tokens` e relatórios focados em coordenadas exatas (`arquivo:linha`). |
| **Segurança e Sanitização** | Proteção de dados sensíveis e credenciais de ambiente em relatórios públicos de issue/PR. | Sanitização automática via `SecretRedactor` mascarando tokens GitHub, senhas e variáveis privadas em saídas gravadas. |

---

## 8. Regras de Negócio (As 20 Hard Invariants do TRustKit)

| ID | Regra de Negócio | Descrição Formal da Invariante |
| :--- | :--- | :--- |
| **RN-001** | **Sem Código Sem Spec Congelada** | Nenhuma linha de código de produção pode ser criada ou commitada sem que a especificação formal correspondente esteja no estado `SpecLockState::Frozen` com hash BLAKE3 gravado no SQLite WAL. |
| **RN-002** | **Evidência sobre Asserção Textual** | Nenhuma etapa de execução (TDD RED/GREEN, Lint, Build, Testes E2E) é concluída por declaração em texto do agente; o próprio binário `trustkit` executa o comando e grava o Exit Code real e digest no banco. |
| **RN-003** | **Aprovação Unânime Cumulativa** | A certificação do Gate 6 exige cumulativamente: Camada 0 AST CleanPass, 100% dos críticos do Gauntlet aprovados (PASS), testes E2E aprovados e linter/build sem advertências em todas as aplicações tocadas. |
| **RN-004** | **Integridade Imutável de Testes (Anti-Cheat)** | Proibição absoluta de remover, comentar ou enfraquecer asserções de testes pré-existentes sem autorização humana formal registrada via `trustkit contract unlock`. |
| **RN-005** | **Circuit Breaker Estrito de 5 Loops** | O ciclo autônomo entre builders e gauntlet é interrompido imediatamente na 5ª reprovação consecutiva, congelando o ciclo em `CircuitBreakerTripped` e exigindo intervenção humana. |
| **RN-006** | **GitFlow Rigoroso e Árvore Compartilhada** | Branch base é `develop`. Branches de trabalho seguem o padrão `feat/<id>-<slug>` ou `fix/<id>-<slug>`. Commits seguem `<type>: #<id> <descricao em pt-BR>`. Proibido `git stash`, `git add .`, `git add -A` e commits diretos em branches protegidas. |
| **RN-007** | **Proibição Absoluta de Emojis e Ícones Gráficos** | Nenhum artefato, documento, código-fonte, relatório, mensagem de commit, comentário de issue ou saída de terminal pode conter caracteres de emojis, emoticons ou dingbats Unicode. |
| **RN-008** | **Evidências Visuais Autenticadas e Restauração Alheia** | Screenshots do Playwright devem ser salvas no diretório oficial da spec e referenciadas via URLs autenticadas. Capturas de tela de outras issues regravadas durante a suíte são restauradas automaticamente via Git. |
| **RN-009** | **Sincronização das 9 Colunas do GitHub Projects** | Cards transicionam ordenadamente entre as colunas oficiais: `Backlog`, `Refinement`, `Ready`, `In progress`, `In review`, `Await Staging`, `In Staging`, `Blocked` e `Done`, sem saltos arbitrários. |
| **RN-010** | **Comentário de Encerramento com Rastreabilidade Total** | A conclusão do Gate 6 gera comentário oficial contendo nota para o time de negócios, matriz de rastreabilidade completa, passo a passo de homologação e resumo técnico da entrega. |
| **RN-011** | **Decisão Explícita de Destino de Merge** | O motor jamais efetua merge silencioso em branches principais sem validação prévia de conformidade e autorização expressa do operador humano. |
| **RN-012** | **Pull Request Exclusivamente Sob Demanda** | A abertura de Pull Request ocorre única e exclusivamente mediante comando humano explícito (`trustkit pr`), rejeitando menções de fechamento automático (`Closes`/`Fixes`). |
| **RN-013** | **Zero Atribuição a Ferramentas de IA** | Proibição de inclusão de trailers de autoria como `Co-Authored-By: Claude`, `Generated-by` ou referências a modelos de linguagem em commits, PRs e documentações. |
| **RN-014** | **Verificação Obrigatória de Conta GitHub** | Toda operação de rede valida previamente se a conta ativa no `gh auth status` corresponde ao perfil autorizado no repositório. |
| **RN-015** | **Tipagem Nativa de Issues e Proibição de Sub-Issues** | Tarefas atômicas de implementação (`TASK-NNN`) jamais são publicadas como sub-issues poluindo o repositório remoto; pertencem exclusivamente ao DAG local. |
| **RN-016** | **Interoperabilidade Upstream (`--from-prd`)** | O motor aceita nativamente documentos PRD/ADR para pré-popular a especificação formal, mantendo rastreabilidade desde a concepção até o teste E2E. |
| **RN-017** | **Auto-Suficiência e Independência de Especificação** | A especificação e linting de features devem operar de forma autônoma no binário nativo, sem exigir ferramentas ou interpretadores externos no ambiente hospedeiro. |
| **RN-018** | **Integridade Bidirecional da Memória Perene** | Toda atualização na memória viva (`trustkit memory`) deve reindexar instantaneamente os links wiki e atualizar o índice FTS5 sob selo de hash BLAKE3. |
| **RN-019** | **Limiar Mínimo de Confiança em Segurança (AppSec >= 8/10)** | Alertas de segurança gerados na auditoria pré-release só bloqueiam a entrega se possuírem índice de confiança avaliado em 8 ou superior, evitando falsos positivos paralisantes. |
| **RN-020** | **Execução Headless Não-Bloqueante** | Em pipelines CI e ambientes sem TTY interativo, comandos de verificação devem resolver ambiguidades por política padrão segura sem travar aguardando entrada de usuário. |

---

## 9. Domínio e Dados

### 9.1. Glossário Ubíquo do Domínio
* **`SddCycle` (Ciclo SDD):** Instância de governança que encapsula o ciclo de vida de uma funcionalidade, associada a uma issue remota ou a um caso especial local.
* **`GateStage` (Estágio de Gate):** Enumeração dos 7 marcos do SDD (`Gate0Discovery`, `Gate1SpecPrompt`, `Gate2SpecLock`, `Gate3WavePlan`, `Gate4TasksImplemented`, `Gate5GauntletCleared`, `Gate6Certification`).
* **`ArtifactSeal` (Selo de Artefato):** Registro de hash BLAKE3 e contagem de nós de asserção AST que atesta a imutabilidade da especificação e dos testes.
* **`TraceabilityLink` (Elo de Rastreabilidade):** Tupla relacionando $	ext{RN} ightarrow 	ext{RF} ightarrow 	ext{TASK} ightarrow 	ext{E2E} ightarrow 	ext{Screenshot}$.
* **`AtomicTask` (Tarefa Atômica):** Unidade atômica de trabalho de uma única camada arquitetural, associada a um teste de destino e executada em worktree dedicada.
* **`ExecutionWave` (Onda de Execução):** Grupo de tarefas atômicas sem conflito mútuo de arquivos nem dependência topológica que podem ser executadas em paralelo.
* **`ExecutionEvidence` (Evidência de Execução):** Registro append-only de comando executado, exit code, duração, hash de saída e parecer determinístico.
* **`KnowledgePage` (Página de Conhecimento):** Documento persistente indexado no FTS5 com título, tags, corpo textual e links wiki.
* **`StructuralNode` / `StructuralEdge`:** Vértices e arestas que compõem o grafo arquitetural de dependências extraído via AST.
* **`TerminalSession` (Sessão de Terminal PTY):** Sessão de processo filho executada sob pseudo-terminal assíncrono com canal de controle IPC.

### 9.2. Entidades Principais e Modelagem Livre de `NULL`

**`SddCycle`**
* `cycle_id`: Identificador textual único (`issue-45-AAAAMMDDHHMM` ou `no-issue-AAAAMMDDHHMM`).
* `issue_linkage`: Objeto de Caso Especial (`LinkedIssue { number, url, board_column }` ou `UnlinkedLocalCycle { directory_slug }`).
* `methodology_profile`: Perfil do framework (`StandardGauntletV2` ou `StructuredSdd`).
* `current_gate`: Estágio corrente na máquina de estados (`0..=6`).
* `lifecycle_status`: Status (`Active`, `Paused`, `CircuitBreakerTripped`, `Certified`, `Merged`).
* `gauntlet_attempt`: Contador limitado (`0..=5`).
* `spec_seal_state`: Estado de selo (`DraftUnverified` ou `Frozen { spec_path, blake3_digest }`).
* `work_branch`: Nome da branch Git em trabalho.
* `pre_existing_dirty_files`: Array de caminhos (`[]` quando limpo).

**`ArtifactSeal`**
* `seal_id`: Inteiro sequencial primário.
* `cycle_id`: Chave estrangeira para o ciclo SDD.
* `file_path`: Caminho do arquivo protegido.
* `artifact_category`: Categoria (`Spec`, `Constitution`, `PreExistingTest`, `ForeignIssueEvidence`).
* `blake3_digest`: Hash criptográfico de 64 caracteres hexadecimais.
* `ast_assertion_count`: Total de nós de asserção AST contabilizados.
* `contract_unlock_state`: Estado (`Sealed` ou `UnlockedByHuman { justification }`).

**`TraceabilityLink`**
* `trace_id`: Inteiro sequencial primário.
* `cycle_id`: Chave estrangeira para o ciclo SDD.
* `business_rule_id`: Código da regra de negócio (`RN-NNN`).
* `functional_req_id`: Código do requisito funcional (`RF-NNN`).
* `task_id`: Identificador da tarefa atômica (`TASK-NNN`).
* `mermaid_journey_node`: Rótulo do nó no diagrama de fluxo.
* `playwright_spec_path`: Caminho relativo da suíte E2E.
* `evidence_screenshot`: Nome do arquivo de captura de tela de evidência.
* `verification_state`: Estado de homologação (`PENDING_GATE5` ou `VERIFIED_PASS`).

**`AtomicTask`**
* `task_id`: Identificador da tarefa (`TASK-NNN`).
* `cycle_id`: Chave estrangeira para o ciclo SDD.
* `wave_index`: Índice da onda topológica (`1..=N`).
* `architectural_layer`: Camada da tarefa (`Core`, `DB`, `AST`, `Orch`, `CLI`, `GH`).
* `tdd_lifecycle_state`: Estado de TDD (`AwaitingRedPhase`, `RedVerified`, `GreenVerified`, `Committed`).
* `worktree_allocation`: Alocação de worktree (`NotProvisioned`, `Provisioned { path }`, `MergedAndPruned`).

---

## 10. Métricas de Sucesso

### 10.1. Métricas de Negócio
* **Redução de 60% a 75% no Tempo Total de Ciclo (Wall-Clock Time):** Funcionalidades completas implementadas e certificadas em fração do tempo através do motor autônomo `trustkit auto` e paralelização por worktrees.
* **Redução de 50% no Consumo de Tokens de LLM por Feature:** Graças à Camada 0 de AST que barra inconformidades mecânicas em microssegundos com zero tokens, e ao fatiamento cirúrgico de contexto (`trustkit context slice`).

### 10.2. Métricas de Produto
* **100% de Aderência às 20 Hard Invariants:** Zero commits com emojis, zero trailers de IA, zero adulterações em testes e zero bypasses de governança registrados.
* **100% de Fechamento na Matriz de Rastreabilidade:** Nenhuma funcionalidade é integrada sem mapeamento bidirecional comprovado entre regras de negócio, testes e evidências visuais.
* **Nota Máxima (100%) em Maturidade de Harness Agêntico:** Certificação de Nível L4 auditada deterministamente via `harness-score`.

### 10.3. Métricas Operacionais
* **Latência de Cold-Start do binário `trustkit`:** `< 3ms` (p99).
* **Tempo de Execução da Camada 0 de AST (`gauntlet ast-audit`):** `< 5ms` (p99).
* **Tempo de Resposta em Consultas de Memória FTS5:** `< 2ms` (p99).
* **Tempo de Montagem do DAG e Detecção de Ciclos:** `< 10ms` (p99).

---

## 11. Riscos e Dependências

### 11.1. Matriz de Riscos

| Risco Mapeado | Probabilidade | Impacto | Estratégia de Mitigação Implementada |
| :--- | :---: | :---: | :--- |
| **Conflitos de merge entre tarefas da mesma onda** | Baixa | Médio | O planejador DAG (`WaveTopologyPlannerPort`) inspeciona a lista de arquivos afetados e força tarefas que tocam os mesmos arquivos para ondas subsequentes. |
| **Sobrecarga de espaço em disco por múltiplas worktrees** | Baixa | Baixo | Criação de worktrees via *reflinks* Copy-on-Write e symlinks compartilhados para pastas pesadas de dependências (`node_modules`, `target`). |
| **Falsos positivos em regras de segurança AppSec** | Média | Médio | Aplicação de filtro rígido de confiança (threshold >= 8/10), limitando os bloqueios apenas a vulnerabilidades de alta certeza e severidade crítica. |
| **Instabilidade de rede na sincronização com GitHub** | Baixa | Baixo | Uso do padrão de Caso Especial `BoardSyncOutcome::DeferredOffline`, permitindo operação local normal com sincronização postergada. |
| **Saturação de processos em sistemas com baixa memória** | Baixa | Médio | Limite configurável de concorrência máxima de workers paralelos no orquestrador Tokio. |

### 11.2. Dependências do Sistema
* **Compilador Rust:** Toolchain estável `rustc >= 1.82` e `cargo`.
* **Ferramental Git:** `git >= 2.38` com suporte a worktrees.
* **GitHub CLI:** `gh >= 2.40` autenticado localmente para operações remotas de quadro e PR.
* **Gramáticas Tree-sitter:** Gramáticas de Rust, TypeScript, TSX, Python, Go e SQL embutidas estaticamente no binário.

---

## 12. Cronograma e Fases Entregues (Status de Engenharia)

| Fase de Engenharia | Funcionalidades Entregues no Repositório | Status Atual |
| :--- | :--- | :---: |
| **Fase 1: Fundação, Domínio e CLI Base** | Pure Domain Models, SQLite WAL (100% NOT NULL), `trustkit init`, `trustkit hook`. | Concluído e Homologado |
| **Fase 2: Motor de Especificação e DAG** | Validador de Spec V2, Ingestor PRD, `trustkit spec lock` BLAKE3 e `trustkit wave plan`. | Concluído e Homologado |
| **Fase 3: AST Camada 0 e Context Slicer** | Tree-sitter parser, `TestIntegrityAuditor` (anti-cheat), lexical scanner e context slicer. | Concluído e Homologado |
| **Fase 4: Git Worktrees e Ciclo TDD** | `WorktreeIsolationProvider` CoW, `TddPhaseGuard` (RED -> GREEN), Swarm de 4 críticos. | Concluído e Homologado |
| **Fase 5: Memória Perene do Projeto** | SQLite FTS5 BM25 search, links wiki bidirecionais e subcomando `trustkit memory`. | Concluído e Homologado |
| **Fase 6: Inteligência de Grafo Estrutural** | Extração de nós/arestas AST, acoplamento afferent/efferent, detecção de god nodes (`trustkit graph`). | Concluído e Homologado |
| **Fase 7: Revisão Semântica e Segurança** | `trustkit task review` e auditoria global de segurança em 6 fases (`trustkit audit security`). | Concluído e Homologado |
| **Fase 8: Autonomia de Especificação** | Independência de ferramentas externas; `trustkit spec prompt` e `trustkit spec lint` nativos. | Concluído e Homologado |
| **Fase 9: Multiplexador de Terminais PTY** | Terminais virtuais PTY assíncronos para despacho paralelo de agentes reais. | Concluído e Homologado |
| **Fase 10: Motor Autônomo e Headless** | Orquestrador `trustkit auto` ponta a ponta, exit codes 0 para help/version e execução headless. | Concluído e Homologado |
| **Fase 11: Governança de Harness L4** | Regras e agentes `.claude/`, `AGENTS.md`, suite SDD thin skills e diagramas Archify. | Concluído e Homologado |
| **Fase 12: Planejamento de Marcos e Citações AST** | Subcomando `trustkit milestones` (plan/list/inspect), `trustkit init --milestone`, citações exatas de linhas PRD/ADR e tabela SQLite WAL. | Concluído e Homologado |

---

## Apêndice: Racional de Nomenclatura e Identidade Arquitetural (TRustKit)

O nome **TRustKit** materializa a convergência entre:
1. **True Rust:** Desempenho nativo com tempos de inicialização da ordem de microssegundos, ausência de coletor de lixo e segurança garantida pelo sistema de tipos e propriedade (*ownership*).
2. **Trust (Confiança Determinística):** Eliminação da volatilidade em sistemas agênticos através de comprovação matemática, assinaturas BLAKE3, rastreabilidade fechada e integridade inviolável de testes.
