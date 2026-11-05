# PRD/ERS: TRustKit — Motor Determinístico em Rust para Spec-Driven Development & Gauntlet Loop

**Versão:** 1.1 (Enriquecido com Diretrizes Clean Code — Robert C. Martin)  
**Data:** Outubro/2026  
**Autor:** Equipe de Produto & Engenharia (Analista de Requisitos + PM + UX/UI de CLI + QA Sênior)  
**Status:** Pronto para Desenvolvimento  

---

## 1. Visão do Produto

### 1.1. Propósito
Fornecer um motor determinístico de altíssima velocidade compilado nativamente em Rust (`trustkit`) que governa os 7 Gates e as 16 Invariantes do framework **MMC Gauntlet SDD V2**, combinando a fluidez de uma Skill enxuta com validação sintática via AST em microssegundos, isolamento paralelo de agentes em Git Worktrees e auditoria imutável em SQLite WAL.

### 1.2. Nome Oficial e Identidade
* **Nome do Produto:** `TRustKit`
* **Binário CLI:** `trustkit`
* **Slogan:** *"True Rust performance for trusted, spec-driven agentic execution."*

### 1.3. Problema que Resolve
1. **Latência Crítica de Harnesses Tradicionais (Python/Node.js):** Harnesses externos escritos em linguagens interpretadas pagam de `300ms` a `2.000ms` de *cold-start* a cada invocação de subprocesso ou *hook* (`PreToolUse`/`PostToolUse`), além de invalidarem o *Prompt Cache (KV-Cache)* do LLM ao reconstruírem sessões a cada transição de estado.
2. **Fragilidade Probabilística de Skills Puramente Declarativas (`SKILL.md`):** Embora rápidas por rodarem dentro da sessão quente do agente, skills em Markdown dependem 100% da obediência do LLM ao prompt. O modelo ainda pode alucinar conformidade, enfraquecer asserções de testes (*test dilution*), esquecer linhas na Matriz de Rastreabilidade ou inserir emojis e trailers proibidos (`Co-Authored-By`).
3. **Desperdício de Tokens e Tempo com Auditorias Mecânicas via LLM:** Acionar subagentes de IA (10s a 40s por turno) apenas para verificar se uma Spec possui blocos Mermaid/Gherkin, se um teste perdeu um `expect()` ou se arquivos fora de escopo foram tocados consome orçamento de tokens e degrada drasticamente o tempo de entrega.
4. **Colisão na Árvore de Trabalho em Execuções Paralelas:** Quando múltiplos *Builder Subagents* tentam executar tarefas de uma mesma onda simultaneamente na mesma árvore Git, ocorrem condições de corrida, corrupção de índice e sobrescrita de evidências do Playwright.
5. **Ausência de Prova Criptográfica e Event Sourcing:** Sem um banco de eventos transacional, o lema *"O log de eventos prova"* fica restrito a relatórios em texto que o próprio agente gerador poderia editar manualmente.

### 1.4. Público-Alvo
* **Engenheiros de Software Sênior & Arquitetos AI-First:** Desenvolvedores que operam fluxos de *Spec-Driven Development* (SDD) e *SDD* em repositórios corporativos e exigem velocidade máxima sem abrir mão de governança rígida.
* **Agentes Codificadores Hospedeiros (Claude Code, Antigravity, Codex):** Consumidores primários da interface CLI/MCP em modo `--json`, que delegam ao binário `trustkit` toda validação determinística, manipulação de Git/GitHub Projects e despacho de ondas paralelas.
* **Tech Leaders & QA / Homologadores:** Stakeholders que auditam Pull Requests, relatórios de certificação (`gauntlet-report.md`), comentários de encerramento em issues e matrizes de rastreabilidade ponta a ponta.

### 1.5. Proposta de Valor
O **TRustKit** estabelece a arquitetura **"Hybrid Rust Core + Thin Skill"**: mantém uma Skill declarativa mínima (para preservar o KV-Cache quente e guiar o raciocínio criativo de produto e código) enquanto transfere 100% do controle de estado, auditoria de AST (`tree-sitter`), validação de especificação, isolamento de *Git Worktrees* (`gix`/`git2`) e sincronização com o GitHub Projects v2 para um binário estático em Rust com tempo de inicialização de **1 a 3 milissegundos**.

---

## 2. Contexto Atual

### 2.1. Processo Atual
Atualmente, a equipe utiliza as skills declarativas `mmc-gauntlet-sdd` (v2.2.0) e `mmc-gauntlet-sdd-v2` (v3.0.0) compostas exclusivamente por arquivos Markdown (`SKILL.md` e `references/*.md`). O agente principal lê o manual, executa múltiplos comandos sequenciais de shell (`git`, `gh`, `npm`) para descobrir a stack e mover cards no GitHub Projects, redige documentos de especificação e tarefas, despacha subagentes sequencialmente ou em ondas na mesma árvore de diretórios e invoca 4 subagentes críticos (LLM) para inspecionar diffs em busca de violações.

### 2.2. Principais Dores

| Dor | Impacto |
| :--- | :--- |
| **Excesso de Turnos de LLM para Tarefas Mecânicas** | Operações como checar conta do GitHub, mover card no GraphQL do GitHub Projects, criar diretórios `docs/specs/<issue-id>-AAAAMMDDHHMM/` e validar templates exigem de 4 a 8 turnos sequenciais de ferramentas (20s a 45s perdidos apenas no Gate 0 e Gate 1). |
| **Auditoria de Invariantes via Inferência** | Verificar se o Builder comentou um teste, usou `any`, adicionou emojis, usou `raw.githubusercontent.com` ou incluiu `Co-Authored-By` depende de um Crítico LLM ler o diff — lento, caro e sujeito a falsos negativos. |
| **Conflito de Concorrência nas Ondas TDD (Gate 4)** | Sem isolamento físico de *worktrees* gerenciado por um motor nativo, Builders paralelos disputam o mesmo diretório `node_modules`/`.git/index`, forçando execução semi-sequencial. |
| **Falta de Trava Criptográfica na Spec (Gate 2)** | O congelamento da Spec no Gate 2 é apenas conceitual; não há validação de hash (`BLAKE3`) em nível de binário que impeça ou detecte instantaneamente alterações não autorizadas na Spec ou em testes pré-existentes. |
| **Ausência de Verificação Mecânica do Ciclo RED -> GREEN** | O agente Builder é instruído a rodar o teste falhando (RED) antes de implementar (GREEN), mas nenhum mecanismo externo impede que ele escreva o código e o teste juntos e declare conformidade. |

### 2.3. Oportunidades
* **Zero-Token Gatekeeping (Camada 0 do Gauntlet):** Barrar 90% das violações estruturais, sintáticas e de contrato em `< 5ms` via análise estática nativa (`tree-sitter`) antes de gastar um único token com os 4 Críticos LLM.
* **Execução Paralela Real por Ondas:** Usar *Git Worktrees* com *Copy-on-Write* (`reflink` em sistemas de arquivos Linux modernos) orquestradas via `tokio` para executar tarefas atômicas simultaneamente com isolamento total.
* **Context Slicing Cirúrgico via AST:** Extrair apenas os símbolos, tipos e assinaturas relevantes para cada `TASK-NNN.md`, reduzindo o tamanho do prompt enviado aos Builders e acelerando o *Time-To-First-Token (TTFT)*.
* **Auditoria Imutável em SQLite WAL:** Registrar cada transição de Gate, saída de comando (`Exit Code`), hash de arquivo e parecer de crítico em `.trustkit/state.db`, gerando o `gauntlet-report.md` e o comentário da issue de forma 100% determinística.

---

## 3. Usuários e Personas

### 3.1. Persona 1: Engenheiro de Software Sênior / Arquiteto de Sistemas AI-First
* **Idade:** 36 anos
* **Experiência:** 14+ anos em engenharia de software, arquitetura distribuída e automação agêntica.
* **Objetivo:** Entregar funcionalidades complexas com qualidade impecável (TDD estrito, 100% das RNs cobertas, Playwright E2E com evidências visuais) no menor tempo de relógio possível.
* **Dor Principal:** Harnesses tradicionais são lentos e burocráticos, enquanto skills puras em Markdown exigem vigilância constante para evitar que o modelo pule etapas ou viole invariantes.
* **Necessidade:** Um binário CLI ultrarrápido (`trustkit`) que funcione em simbiose com uma Skill enxuta, impondo todas as regras de governança automaticamente em milissegundos.

### 3.2. Persona 2: Agente Orquestrador Hospedeiro (Claude Code / Antigravity / Codex)
* **Idade:** N/A (Agente de IA)
* **Experiência:** Opera com janela de contexto limitada e custo proporcional ao volume de tokens lidos/gerados.
* **Objetivo:** Cumprir os 7 Gates sem erros de ferramenta, sem poluir a janela de contexto com logs verbosos e sem desperdiçar turnos com operações repetitivas.
* **Dor Principal:** Receber saídas de terminal gigantescas (milhares de linhas de stacktrace ou JSON bruto do GitHub) que degradam sua atenção e consomem o KV-Cache.
* **Necessidade:** Subcomandos determinísticos (`trustkit init --json`, `trustkit gauntlet ast-audit --json`) que retornem payloads JSON enxutos, acionáveis, sem valores nulos (`null`) e com códigos de saída semânticos.

### 3.3. Persona 3: Tech Leader & Revisor de Arquitetura
* **Idade:** 38 anos
* **Experiência:** 15 anos em liderança técnica e governança de engenharia.
* **Objetivo:** Revisar Pull Requests com confiança absoluta de que as Regras de Negócio (`RN`), os Contratos, os testes E2E e o quadro do GitHub Projects foram respeitados sem atalhos.
* **Dor Principal:** Revisar código gerado por IA onde os testes foram sutilmente ajustados para passar ou onde falta rastreabilidade clara entre a regra de negócio e a tela testada.
* **Necessidade:** Relatórios de certificação gerados a partir de um Event Log imutável, com Matriz de Rastreabilidade Ponta a Ponta ($\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$) e garantia anti-cheat verificada por AST.

---

## 4. Jornada do Usuário (e Ergonomia de CLI)

### 4.1. Jornada Atual (Com Skill Pura em Markdown)
```text
1. Dev invoca `/mmc-gauntlet-sdd-v2 auto https://github.com/org/repo/issues/45 --from-prd docs/PRD.md`.
2. O LLM lê 420 linhas do SKILL.md + templates em references/.
3. O LLM executa 5 a 7 comandos shell separados (`gh auth status`, `git status`, queries GraphQL `gh api` para achar o card no Projects e movê-lo para "In progress"). [~25s]
4. O LLM redige a Spec V2 e as Tasks TASK-NNN.md manualmente. Se esquecer uma coluna na Matriz de Rastreabilidade, ninguém percebe até o Gate 5.
5. Os Builders rodam na mesma árvore Git (risco de conflito) e o LLM avalia o próprio ciclo TDD.
6. No Gate 5, 4 subagentes LLM são invocados para ler diffs procurando emojis, adulteração de testes e falhas de contrato. [~60s-120s]
7. O LLM monta manualmente o relatório e posta o comentário na issue.
```

### 4.2. Jornada Futura (Com o TRustKit — Hybrid Rust Core + Thin Skill)
```text
1. Dev invoca `/trustkit auto https://github.com/org/repo/issues/45 --from-prd docs/PRD.md`.
2. A Thin Skill executa `trustkit init --issue 45 --from-prd docs/PRD.md --json`.
   -> Em < 180ms (I/O de rede concorrente em Rust): valida conta GitHub ativa, checa árvore Git, cria `docs/specs/issue-45-AAAAMMDDHHMM/`, move card no GitHub Projects para `In progress`, pré-processa o PRD upstream e abre a sessão no `.trustkit/state.db`.
3. O Agente preenche a Spec V2 e roda `trustkit spec lock`.
   -> Em 2ms: o Rust valida a presença do flowchart Mermaid, a sintaxe dos blocos Gherkin, o fechamento matemático da Matriz (RN -> RF -> TASK -> E2E) e grava o hash BLAKE3 da Spec congelada no SQLite.
4. O Agente gera as tarefas e roda `trustkit wave plan`.
   -> Em 1ms: o Rust valida o grafo acíclico (DAG) de dependências e organiza as Ondas de Execução.
5. O Agente dispara `trustkit wave run --wave 1`.
   -> O Rust cria Git Worktrees efêmeras via CoW, fatia o contexto via AST (`tree-sitter`), despacha os Builders headless em paralelo (`tokio`), exige prova de falha (RED) seguida de sucesso (GREEN) no SQLite e faz o merge limpo na branch de trabalho da feature (`feat/45/...`).
6. O Agente dispara `trustkit gauntlet run`.
   -> Camada 0 (Rust Nativo, < 5ms): Audita AST contra test-cheating, `any`, emojis, `Co-Authored-By`, `raw.githubusercontent` e Spec Tamper. Se falhar, aborta em 5ms com diagnóstico exato sem gastar tokens!
   -> Camada 1 (Parallel Swarm + Playwright): Dispara lint/build, Playwright E2E (restaurando screenshots de outras issues automaticamente) e os Críticos LLM de Domínio/Contrato em paralelo.
7. Com 100% PASS, `trustkit done` compila `gauntlet-report.md` a partir do Event Log imutável, faz commit das evidências e publica o comentário oficial na issue com Matriz de Rastreabilidade e imagens autenticadas.
```

### 4.3. Diretrizes de UX/UI para CLI Ultraleve (Dual-Audience Design)
O `trustkit` **não possui interface TUI nem servidor Web**, concentrando 100% da ergonomia em uma CLI ultraleve com dois modos de saída:
* **Modo Agente (`--json` ou detecção automática via `TRUSTKIT_AGENT_MODE=1`):** Saída JSON estritamente tipada, minificada, sem códigos ANSI e **sem campos `null`** (utilizando objetos de Caso Especial e listas vazias `[]`), projetada para ocupar o mínimo de tokens na janela de contexto do LLM.
* **Modo Humano (TTY padrão):** Saída limpa em `stdout`/`stderr`, puramente textual e estruturada (respeitando a **Invariante #7 de proibição estrita de emojis e ícones gráficos**), usando alinhamento tabular ASCII e prefixos formais `[PASS]`, `[FAIL]`, `[GATE-0]..[GATE-6]`.

---

## 5. Escopo do MVP

### 5.1. Itens no Escopo
#### 5.1.1. Core de Domínio & Repositório de Eventos (`trustkit-core` & `trustkit-db`)
* Máquina de estados determinística dos **7 Gates** (`Gate 0` a `Gate 6` + etapas de Comentário, Decisão de Merge e PR sob demanda).
* Repositório transacional **SQLite Embarcado em modo WAL** (`.trustkit/state.db`) para *Event Sourcing* imutável, controle do *Circuit Breaker* (limite de 5 loops), assinaturas `BLAKE3` de arquivos congelados e auditoria de ciclos `RED -> GREEN -> REFACTOR`.
* Detecção nativa e suporte completo ao **Modo SDD Estruturado** (`.specify/memory/constitution.md`, `specs/<NNN>-<nome>/spec.md`, `plan.md`, `tasks.md`).

#### 5.1.2. Ingestor Upstream, Validador de Spec V2 & Planejador de Ondas (`trustkit-spec`)
* Ingestor `--from-prd <path>` que extrai Regras de Negócio (`RN`), Requisitos Funcionais (`RF`), entidades e jornadas de documentos PRD/ADR em Markdown, gerando o esqueleto pré-populado da Spec V2.
* Validador estrutural instantâneo da Spec V2: verifica seções obrigatórias do Product Trio, sintaxe do diagrama Mermaid (`flowchart LR`), blocos formais Gherkin (`Dado / Quando / Então / E`) e integridade referencial da Matriz de Rastreabilidade ($\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$).
* Planejador de Ondas (`WaveExecutionPlanner`): valida tarefas atômicas (`TASK-NNN.md` e `INDEX.md`) com construção de Grafo Acíclico Direcionado (DAG) para cálculo determinístico das Ondas de Paralelismo.

#### 5.1.3. Auditor Sintático via AST & Context Slicer (`trustkit-ast`)
* Motor baseado em `tree-sitter` (suportando TypeScript/TSX, JavaScript, Python, Rust, SQL e Go) que executa a **Camada 0 do Gauntlet** em `< 5ms`:
  * **Auditor de Integridade de Testes (`TestIntegrityAuditor` — Invariante #4):** Compara a AST dos arquivos de teste antes e depois do Builder; bloqueia se asserções (`expect`, `assert`, `assert_eq!`) foram removidas/enfraquecidas ou se modificadores `.skip` / `.only` / `@ignore` foram introduzidos.
  * **Scanner de Invariantes Léxicos (`InvariantLexicalScanner`):** Bloqueia emojis/ícones Unicode em qualquer artefato/commit (Invariante #7), links `raw.githubusercontent.com` (Invariante #8), atribuições a IA como `Co-Authored-By: Claude` (Invariante #13) e uso não autorizado de `any` ou `@ts-ignore`.
  * **Fatiador de Contexto (`AstContextSlicer`):** Extrai assinaturas de tipos, interfaces e funções públicas dos arquivos-alvo de uma `TASK-NNN` para enxugar o prompt enviado aos subagentes.

#### 5.1.4. Isolador de Worktrees & Despachante de Agentes (`trustkit-orch`)
* Provisor de **Git Worktrees efêmeras** (`WorktreeIsolationProvider` em `.trustkit/worktrees/task-NNN`) com suporte a cópia rápida e isolamento de execução por onda.
* Despachante assíncrono (`tokio`) capaz de operar em **Modo Híbrido**:
  * Expor ferramentas via **MCP (Model Context Protocol)** sobre `stdio` e subcomandos CLI para uma *Thin Skill* hospedeira.
  * Despachar processos de agentes headless (`claude -p`, `codex exec`, ou comando customizado via config) em paralelo para executar as tarefas de uma Onda TDD (Gate 4) ou o Swarm de 4 Críticos Independentes (Gate 5).
* Guardião de Evidências Visuais do Playwright (`VisualEvidenceGuard`): verifica a gravação das capturas de tela em `docs/specs/<issue-id>-AAAAMMDDHHMM/tests/evidence/` e **restaura automaticamente** via Git qualquer screenshot de outra issue que tenha sido regravada durante a bateria E2E (Invariante #8).

#### 5.1.5. Sincronizador Git & GitHub Projects v2 (`trustkit-gh`)
* Guardião de Identidade GitHub (`GitHubIdentityGuard`): verifica conta ativa autenticada no `gh` (ou perfil configurado via `TRUSTKIT_GITHUB_USER`) antes de qualquer operação de rede (Invariante #14).
* Sincronizador do Quadro GitHub Projects v2 (`ProjectBoardSynchronizer`): movimentação de cards nas **9 Colunas Oficiais**, criação de issues tipadas pelo campo nativo `Type` (sem sub-issues de tarefa — Invariante #15), publicação do comentário de encerramento e abertura de Pull Request exclusivamente sob demanda (`trustkit pr`).

#### 5.1.6. Interceptador de Hooks de Ciclo de Vida (`trustkit hook`)
* Subcomandos `trustkit hook pre-tool` e `trustkit hook post-tool` com tempo de execução `< 3ms` para bloquear em tempo real comandos proibidos pelo agente hospedeiro (`git stash`, `git add .`, `git add -A`, `git reset --hard`, commit direto em `main`/`staging`/`develop` sem flag de autorização explícita, ou `gh pr create` sem comando explícito do usuário).

### 5.2. Itens Fora do Escopo (V2 ou Futuro)
* Interface gráfica TUI (`ratatui`) ou Dashboard Web local (`axum`).
* Gerenciamento de faturamento direto de chaves de API de provedores de LLM.
* Criação de sub-issues no GitHub para tarefas atômicas (expressamente proibido pela Invariante #15).
* Abertura automática de Pull Requests ao fim do Gate 6 (expressamente proibido pela Invariante #12; PR é sempre sob demanda).

---

## 6. Requisitos Funcionais (Especificação BDD Executável — Build / Operate / Check)

> **Diretriz Clean Code (Capítulos 1 e 9):** Todos os requisitos funcionais abaixo seguem estritamente a estrutura *Given / When / Then* (`Dado / Quando / Então`), contemplando tanto o caminho nominal quanto os cenários de falha determinística e ausência de dados (*Special Cases*).

| ID | Requisito de Domínio | Critérios de Aceite Executáveis (BDD — Dado / Quando / Então) |
| :--- | :--- | :--- |
| **RF-001** | **Inicialização Concorrente e Sincronização de Board (`Gate 0`)** | **Cenário 1.1 (Com Issue Vinculada):** **Dado** que o repositório possui árvore Git acessível e o desenvolvedor executa `trustkit init --issue 45`, **quando** o comando inicia, **então** o binário valida a conta GitHub ativa, preserva arquivos sujos pré-existentes na lista `PreExistingDirtyFiles` (sem executar `git stash`), move o card `#45` para `In progress`, cria `docs/specs/issue-45-AAAAMMDDHHMM/` e registra o ciclo `SddCycle` no SQLite com `Exit Code: 0` em `< 250ms`.<br>**Cenário 1.2 (Caso Especial Sem Issue):** **Dado** que `trustkit init` é invocado sem argumento `--issue` e fora de uma branch de feature (`feat/<id>-...`), **quando** o comando inicia, **então** o binário instancia o Caso Especial `IssueTarget::UnlinkedCycle`, cria o diretório `docs/specs/no-issue-AAAAMMDDHHMM/`, não move nenhum card no GitHub Projects e solicita confirmação explícita ao usuário. |
| **RF-002** | **Ingestão Upstream de PRD/ADR (`Gate 0a`)** | **Cenário 2.1 (PRD Válido):** **Dado** um arquivo PRD/ADR contendo tabelas de `RN` e `RF`, **quando** `trustkit spec init --from-prd docs/PRD.md` é executado, **então** o ingestor gera `<topico>-spec.md` com as seções do Product Trio, Gherkin e Matriz de Rastreabilidade pré-populadas.<br>**Cenário 2.2 (Arquivo PRD Ausente ou Malformado):** **Dado** que o caminho informado em `--from-prd` não existe ou não contém tabelas `RN`/`RF`, **quando** o comando é executado, **então** o binário aborta com `Exit Code: 2` (`SpecIngestionError::MissingBusinessRules`), sem criar arquivo parcial em disco. |
| **RF-003** | **Validação e Congelamento Criptográfico da Spec V2 (`Gate 1 & Gate 2`)** | **Cenário 3.1 (Spec Completa):** **Dado** um arquivo `<topico>-spec.md` contendo diagrama Mermaid `flowchart LR`, critérios em Gherkin e Matriz $\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$ 100% fechada, **quando** `trustkit spec lock` é executado, **então** o binário grava o estado `SpecLockState::Frozen { blake3_digest }` no SQLite em `< 5ms` e avança para `GATE_2_LOCKED`.<br>**Cenário 3.2 (Regra de Negócio Órfã na Matriz):** **Dado** uma Spec onde a `RN-002` não possui `TASK` ou `Screenshot` correspondente na Matriz de Rastreabilidade, **quando** `trustkit spec lock` é executado, **então** o binário rejeita o congelamento (`Exit Code: 3`) e retorna o objeto `TraceabilityGap { orphan_rule: "RN-002" }`. |
| **RF-004** | **Planejamento de Ondas via DAG (`Gate 3`)** | **Cenário 4.1 (Grafo Acíclico Válido):** **Dado** um conjunto de arquivos `TASK-NNN.md` sem dependências circulares, **quando** `trustkit wave plan` é executado, **então** o planejador retorna a estrutura `WaveSchedule` dividida em ondas topológicas e separa automaticamente em ondas distintas quaisquer tarefas que declarem tocar o mesmo arquivo.<br>**Cenário 4.2 (Dependência Circular Detectada):** **Dado** que `TASK-001` depende de `TASK-002` e `TASK-002` depende de `TASK-001`, **quando** `trustkit wave plan` é executado, **então** o planejador aborta em `< 2ms` com `DagValidationError::CycleDetected { cycle_path: ["TASK-001", "TASK-002"] }`. |
| **RF-005** | **Fiscalização Determinística do Ciclo TDD (`Gate 4`)** | **Cenário 5.1 (Ciclo RED -> GREEN Válido):** **Dado** que o Builder registrou uma execução `trustkit task test TASK-001 --phase red` com falha comprovada (`Exit Code != 0`) e posteriormente uma execução `--phase green` passando (`Exit Code == 0`), **quando** `trustkit task complete TASK-001` é invocado, **então** a tarefa transiciona para `TddLifecycleState::GreenVerified`.<br>**Cenário 5.2 (Violação de Ordem TDD — Sem Fase RED):** **Dado** que o Builder tenta executar `--phase green` ou `task complete` sem registro prévio de `TddLifecycleState::RedVerified` no SQLite, **quando** o comando é chamado, **então** o binário bloqueia a transição com `TddViolationError::MissingRedPhaseProof`. |
| **RF-006** | **Isolamento e Merge de Worktrees por Onda (`Gate 4`)** | **Dado** uma onda contendo $N$ tarefas independentes, **quando** `trustkit wave run --wave <W>` é disparado, **então** o `WorktreeIsolationProvider` instancia $N$ worktrees efêmeras em `.trustkit/worktrees/`, executa os Builders em paralelo via `tokio`, valida os commits no padrão `<type>: #<id> <descricao em pt-BR>` e realiza o merge determinístico na branch de trabalho (`feat/<id>-<slug>`). |
| **RF-007** | **Auditoria AST Anti-Cheat e Invariantes — Camada 0 (`Gate 5`)** | **Cenário 7.1 (Código Limpo e Íntegro):** **Dado** que o diff da branch não alterou a Spec congelada, manteve todas as asserções pré-existentes intactas e não introduziu emojis, `any` ou trailers de IA, **quando** `trustkit gauntlet ast-audit` é executado, **então** retorna o Caso Especial `AstAuditVerdict::CleanPass { violations: [] }` em `< 5ms`.<br>**Cenário 7.2 (Detecção de Test Dilution / Anti-Cheat):** **Dado** que um Builder comentou um `expect()` ou adicionou `.skip` em um teste listado em `artifact_locks`, **quando** `trustkit gauntlet ast-audit` é executado, **então** retorna `AstAuditVerdict::Rejected` com `TestIntegrityViolation { file, line, removed_assertions }` e `Exit Code: 4`, impedindo o disparo dos Críticos LLM. |
| **RF-008** | **Orquestração do Swarm de Críticos e Playwright E2E — Camada 1 (`Gate 5`)** | **Dado** que a Camada 0 retornou `CleanPass`, **quando** `trustkit gauntlet run` inicia a Camada 1, **então** o motor dispara concorrentemente os 4 Críticos (Adversarial de Domínio/Técnico, Contrato, Estático/Build e E2E Playwright), confirma a gravação das screenshots da Jornada Mermaid em `docs/specs/<issue-id>-AAAAMMDDHHMM/tests/evidence/` e executa `git checkout --` em qualquer screenshot de outra issue tocada durante a bateria. |
| **RF-009** | **Disjuntor de Segurança / Circuit Breaker (`Gate 5`)** | **Dado** que o ciclo acumulou 4 reprovações no Gauntlet, **quando** a 5ª iteração consecutiva resulta em `FAIL`, **então** o motor transiciona o ciclo para `CycleStatus::CircuitBreakerTripped`, bloqueia novas execuções de `wave run` ou `gauntlet run` e gera automaticamente o relatório de diagnóstico (`trustkit doctor`). |
| **RF-010** | **Certificação Imutável e Comentário na Issue (`Gate 6`)** | **Dado** que a Camada 0, os 4 Críticos, o lint/build e o Playwright E2E possuem eventos `PASS` (`Exit Code: 0`) no SQLite, **quando** `trustkit done` é executado, **então** o binário renderiza `gauntlet-report.md` a partir dos registros do banco, commita as evidências na branch, publica o comentário na issue com a Matriz de Rastreabilidade e tags `<img src=".../blob/...?raw=true" />`, e aguarda a decisão de destino de merge do usuário. |
| **RF-011** | **Gate de Merge e Transição para `Await Staging`** | **Dado** um ciclo em estado `CycleStatus::Certified` e confirmação do usuário para mergear em `develop`, **quando** `trustkit merge --target develop` é executado, **então** o binário realiza o merge e move o card no GitHub Projects para `Await Staging` mantendo a issue aberta. |
| **RF-012** | **Abertura de Pull Request Exclusivamente Sob Demanda** | **Dado** que o usuário invocou explicitamente `trustkit pr`, **quando** o comando executa, **então** o binário abre o PR para `develop` com revisores e assignees configurados, referência `Ref #<id>` (rejeitando `Closes`/`Fixes`/`Resolves`), Matriz de Rastreabilidade e imagens HTML, movendo o card para `In review`. |
| **RF-013** | **Interceptação Preventiva via Hooks (`PreToolUse`)** | **Dado** que o agente hospedeiro tenta invocar uma ferramenta shell contendo `git stash`, `git add .`, `git add -A`, commit direto em branch protegida ou `gh pr create` fora do fluxo, **quando** `trustkit hook pre-tool` inspeciona o comando, **então** o binário nega a execução em `< 3ms` retornando a invariante violada. |

---

## 7. Estratégia para Casos Especiais e Ausência de Dados (Clean Code — Capítulo 7)

> **Invariante de Modelagem (Proibição de `null`):** É estritamente proibido o uso de valores `null` / `nil` nos modelos de domínio, nos contratos JSON da CLI (`--json`) e nas regras de negócio do `TRustKit`. Toda ausência de dado ou condição atípica deve ser representada pelo **Padrão de Caso Especial (*Special Case / Null Object Pattern*)** através de *Tagged Unions* (Enums algébricos em Rust) ou coleções vazias imutáveis (`[]`).

| Cenário de Ausência ou Caso Especial | Representação Proibida (Anti-Pattern) | Representação Obrigatória no Domínio e JSON (`Special Case Pattern`) | Comportamento Determinístico do Sistema |
| :--- | :--- | :--- | :--- |
| **Ciclo iniciado sem Issue do GitHub vinculada** | `issue_number: null`, `issue_url: null`, `board_column: null` | `IssueLinkage::UnlinkedLocalCycle { directory_slug: "no-issue-202610271230" }` | Cria o diretório `docs/specs/no-issue-AAAAMMDDHHMM/`, opera todos os Gates locais normalmente e ignora silenciosamente mutações no GitHub Projects sem lançar erro de ponteiro nulo. |
| **Spec ainda em elaboração (antes do Gate 2)** | `spec_blake3_hash: null` | `SpecLockState::DraftUnverified` (transiciona para `SpecLockState::Frozen { blake3_digest }` no Gate 2) | Qualquer tentativa de iniciar o Gate 4 (`wave run`) recebe o estado `DraftUnverified` e recusa a execução com mensagem instrutiva: `"Execute 'trustkit spec lock' antes de iniciar os Builders"`. |
| **Tarefa Atômica ainda não iniciada em Worktree** | `worktree_path: null`, `commit_sha: null`, `red_exit_code: null` | `TaskExecutionContext::NotProvisioned` e `TddLifecycleState::AwaitingRedPhase` | Comandos de status e relatórios renderizam `"AWAITING_RED_PHASE"` em vez de campos vazios ou `N/A`. |
| **Árvore Git 100% limpa no Gate 0 ou Auditoria AST sem violações** | `pre_existing_dirty: null` ou `violations: null` | Lista imutável vazia `pre_existing_dirty: []` e objeto `AstAuditVerdict::CleanPass { violations: [] }` | Iteradores nos consumidores processam a lista vazia sem ramificações `if (x != null)`. |
| **Repositório sem SDD (`.specify/` ausente)** | `sdd_config: null` | `MethodologyProfile::StandardGauntletV2` (vs `MethodologyProfile::StructuredSdd { feature_dir }`) | Polimorfismo via enum despacha o validador da Spec V2 padrão sem checagens condicionais espalhadas pelo código. |
| **Falha de comunicação temporária com a API do GitHub** | Retornar erro genérico ou `board_status: null` | `BoardSyncOutcome::DeferredOffline { queued_transition: "In progress", retry_after_ms: 5000 }` | Registra a transição pendente no SQLite local e permite que o desenvolvedor continue trabalhando no Gate 1/2 sem bloqueio de rede. |

---

## 8. Requisitos Não-Funcionais

| Atributo | Requisito | Métrica Objetiva |
| :--- | :--- | :--- |
| **Performance (Cold-Start)** | Inicialização instantânea do binário para uso em hooks síncronos e chamadas CLI frequentes. | `< 3ms` (p99) para subcomandos locais (`hook`, `spec verify`, `status`). |
| **Performance (Auditoria AST)** | Análise sintática incremental via `tree-sitter` e verificação das 16 invariantes sobre o diff da branch. | `< 10ms` para diffs de até 5.000 linhas modificadas. |
| **Performance (Context Slicing)** | Extração de assinaturas e tipos públicos dos arquivos-alvo de uma tarefa. | `< 15ms` por tarefa atômica; redução mínima de `60%` no volume de tokens de código enviados ao Builder. |
| **Confiabilidade & Integridade** | Persistência transacional à prova de corrupção sob múltiplos processos Builders/Critics simultâneos. | SQLite em modo `WAL` com `synchronous = NORMAL`, `busy_timeout = 5000ms` e `NOT NULL` em todas as colunas de estado; 0% de corrupção sob concorrência. |
| **Portabilidade & Distribuição** | Binário único estático sem dependência de runtime Python, Node.js ou bibliotecas dinâmicas externas. | Binário único compilado para Linux (`x86_64-unknown-linux-gnu` / `musl`) e macOS (`aarch64-apple-darwin`) com tamanho `< 25 MB`. |
| **Ergonomia de Tokens (AI UX)** | Respostas no modo `--json` otimizadas para consumo mínimo de janela de contexto do LLM. | Payloads de sucesso `< 300 tokens`; zero campos `null`; payloads de falha contendo apenas erros acionáveis e coordenadas exatas (`arquivo:linha:coluna`). |
| **Segurança** | Proteção de credenciais e proibição de vazamento de segredos em logs de evidência. | Uso nativo do token já autenticado no `gh auth`; sanitização automática (`SecretRedactor`) antes de gravar saídas no `state.db` e `gauntlet-report.md`. |

---

## 9. Regras de Negócio (As 16 Hard Invariants do TRustKit)

| ID | Regra | Descrição Invariante |
| :--- | :--- | :--- |
| **RN-001** | **Sem Código Sem Spec Congelada (V2)** | Nenhum commit ou execução de Builder (Gate 4) é permitido se o ciclo não estiver com `SpecLockState::Frozen` no `.trustkit/state.db`. No perfil `StandardGauntletV2`, exige Spec com Product Trio, Mermaid, Gherkin e Matriz de Rastreabilidade. No perfil `StructuredSdd`, exige `spec.md`, `plan.md`, `tasks.md` e `gauntlet-rubric.md`. |
| **RN-002** | **Evidência sobre Asserção** | Nenhum Gate de execução (TDD RED/GREEN, Lint, Build, Playwright E2E) pode ser marcado como concluído por declaração textual do agente; o próprio binário `trustkit` deve capturar o comando, o `stdout`/`stderr`, a duração e o `Exit Code` real no banco de eventos. |
| **RN-003** | **Aprovação Unânime Cumulativa** | A certificação (Gate 6) exige, cumulativamente: Camada 0 (AST) `CleanPass`, todos os 4 Críticos do Gauntlet `PASS`, `npm run test:e2e` com `Exit Code: 0` e `npm run lint && npm run build` com `Exit Code: 0` em cada aplicação tocada. |
| **RN-004** | **Integridade Imutável de Testes (Anti-Cheat)** | É proibido remover, comentar ou enfraquecer asserções de testes pré-existentes sem autorização humana formal registrada via `trustkit contract unlock`. |
| **RN-005** | **Circuit Breaker Estrito de 5 Loops** | O ciclo autônomo entre Builders (Gate 4) e Gauntlet (Gate 5) possui limite máximo intransponível de 5 iterações. Na 5ª falha, o motor aciona o disjuntor (`CircuitBreakerTripped`) e exige intervenção humana. |
| **RN-006** | **GitFlow Rigoroso e Árvore Compartilhada** | Branch base é `develop` (`develop -> staging -> main`). Branches de trabalho seguem `feat/<task-id>-<titulo-curto-kebab>` (ou `bugfix/<task-id>-<titulo-curto-kebab>`). Commits seguem `<type>: #<task-id> <descricao em pt-BR>`. Proibido `git stash`, `git add .`, `git add -A` e reversão autônoma de commits. Arquivos sujos prévios ao ciclo permanecem intocados. |
| **RN-007** | **Proibição Absoluta de Emojis e Ícones Gráficos** | Nenhum arquivo de spec, task, relatório, commit, issue, comentário ou PR pode conter caracteres dos blocos Unicode de Emojis, Emoticons ou Dingbats. |
| **RN-008** | **Evidências Visuais Autenticadas e Restauração Alheia** | Capturas do Playwright devem residir em `docs/specs/<issue-id>-AAAAMMDDHHMM/tests/evidence/` e usar exclusivamente `<img src="https://github.com/<org>/<repo>/blob/<branch>/.../evidence/<arq>.png?raw=true" alt="..." width="100%" />`. Evidências de outras issues regravadas durante a suíte E2E são restauradas via `git checkout -- <path>` antes do commit. |
| **RN-009** | **Sincronização das 9 Colunas do GitHub Projects** | O card da issue transiciona estritamente entre `Backlog`, `Refinement`, `Ready`, `In progress`, `In review`, `Await Staging`, `In Staging`, `Blocked` e `Done`, sem regressão silenciosa de coluna. No caso especial `UnlinkedLocalCycle`, nenhum card é movido. |
| **RN-010** | **Comentário de Encerramento com Rastreabilidade Total** | Concluído o Gauntlet, publica-se na issue o comentário contendo Nota para o Time de Negócios, Matriz de Rastreabilidade Ponta a Ponta ($\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E} \rightarrow \text{Screenshot}$), Passo a Passo de Validação, Screenshots HTML e Resumo Técnico. |
| **RN-011** | **Decisão Explícita de Destino de Merge** | Ao final do Gate 6, o motor jamais faz merge silencioso: pergunta explicitamente ao usuário para qual branch deseja mergear (ou informa caso tenha sido autorizado commit direto na `develop`). |
| **RN-012** | **Pull Request Exclusivamente Sob Demanda** | Proibido abrir PR automaticamente ao fim da issue. Quando solicitado pelo usuário (`trustkit pr`), deve incluir revisores e assignees configurados, coluna `In review` e referência `Ref #<id>` (proibido `Closes`/`Fixes`/`Resolves`). |
| **RN-013** | **Zero Atribuição a Ferramentas de IA** | Bloqueio estrito de trailers `Co-Authored-By: Claude/Anthropic/...`, `Claude-Session`, rodapés `Generated with...` ou links `claude.ai/code/...` em commits, PRs, issues e comentários. |
| **RN-014** | **Verificação Obrigatória de Conta GitHub** | Toda operação GitHub verifica previamente se a conta ativa no `gh auth status` corresponde ao perfil autenticado (ou configurado via `TRUSTKIT_GITHUB_USER`), executando `gh auth switch` se necessário. |
| **RN-015** | **Tipagem Nativa de Issue e Proibição de Sub-Issues** | Issues utilizam o campo nativo `Type` da organização (`Bug`, `Feature`, `Test`, `Call`, `Administrative`, `Alignment`, `Deployment`, `Prototyping`). Tarefas atômicas (`TASK-NNN`) jamais viram sub-issues no GitHub. |
| **RN-016** | **Interoperabilidade Upstream (`--from-prd`)** | O motor aceita nativamente documentos PRD/ADR upstream para popular a Spec V2, mantendo rastreabilidade desde a concepção do produto até a evidência E2E. |

---

## 10. Domínio e Linguagem Ubíqua (Clean Code — Capítulo 2)

> **Regra de Nomenclatura do Domínio:** São proibidos sufixos vagos ou genéricos como `Manager`, `Processor`, `Data`, `Info`, `Helper` ou `Handler` nas entidades e serviços de domínio, bem como siglas obscuras de tela/sistema. Todos os substantivos e verbos abaixo compõem a **Linguagem Ubíqua** obrigatória tanto no PRD quanto nas structs e traits do código Rust.

### 10.1. Glossário Ubíquo de Substantivos e Verbos do Domínio

| Termo Ubíquo (Código / Produto) | Categoria | Definição Precisa no Domínio | Termos Genéricos Proibidos |
| :--- | :---: | :--- | :--- |
| **`SddCycle` (Ciclo SDD)** | Substantivo | Instância completa e isolada de entrega de uma funcionalidade, particionada em `docs/specs/<issue-id>-AAAAMMDDHHMM/`. | `SessionData`, `ProcessInfo`, `RunItem` |
| **`GateStage` (Estágio de Portão)** | Substantivo | Cada um dos 7 marcos determinísticos (`Gate0Discovery` a `Gate6Certification`) que governam a progressão do ciclo. | `StepData`, `PhaseNumber` |
| **`ArtifactSeal` (Selo Criptográfico)** | Substantivo | Assinatura `BLAKE3` mais contagem de nós de asserção AST capturadas no Gate 2 para garantir imutabilidade da Spec e dos testes pré-existentes. | `FileInfo`, `HashData`, `LockManager` |
| **`TraceabilityLink` (Elos de Rastreabilidade)** | Substantivo | Tupla verificável que amarra `BusinessRuleId (RN)` $\rightarrow$ `FunctionalRequirementId (RF)` $\rightarrow$ `TaskId (TASK)` $\rightarrow$ `PlaywrightSpecPath` $\rightarrow$ `ScreenshotFileName`. | `MappingData`, `MatrixRowInfo` |
| **`AtomicTask` (Tarefa Atômica)** | Substantivo | Unidade mínima de implementação TDD restrita a uma única camada arquitetural e executada em uma `IsolatedWorktree` própria. | `WorkItem`, `SubIssue`, `TaskData` |
| **`ExecutionWave` (Onda de Execução)** | Substantivo | Subconjunto de `AtomicTasks` sem dependência mútua nem colisão de arquivos, despachado simultaneamente no Gate 4. | `BatchGroup`, `TaskList` |
| **`ExecutionEvidence` (Evidência de Execução)** | Substantivo | Registro imutável de um comando real executado pelo harness, contendo comando literal, duração em milissegundos, `ExitCode`, resumo sanitizado da saída e hash `BLAKE3`. | `LogData`, `CommandInfo` |
| **`SealSpecification` (Selar Especificação)** | Verbo | Ação do Gate 2 (`trustkit spec lock`) que valida a completude da Spec V2 e grava seu `ArtifactSeal` no repositório. | `ProcessSpec`, `ManageSpec` |
| **`SliceAstContext` (Fatiar Contexto Sintático)** | Verbo | Ação de extrair da árvore sintática (`tree-sitter`) apenas as assinaturas públicas e tipos necessários para uma `AtomicTask`. | `GetFileData`, `PrepareInfo` |
| **`VerifyTestIntegrity` (Verificar Integridade de Testes)** | Verbo | Ação da Camada 0 do Gauntlet que compara a AST de testes antes e depois das mudanças do Builder para barrar *test dilution*. | `CheckTests`, `ProcessDiff` |
| **`TripCircuitBreaker` (Acionar Disjuntor)** | Verbo | Ação automática de travar o `SddCycle` após a 5ª reprovação consecutiva no Gauntlet, exigindo diagnóstico humano. | `StopLoop`, `HandleError` |
| **`RestoreForeignEvidence` (Restaurar Evidência Alheia)** | Verbo | Ação de reverter automaticamente capturas de tela de outras issues que tenham sido sobrescritas durante a suíte Playwright E2E. | `CleanFiles`, `FixScreenshots` |

### 10.2. Entidades Principais do Domínio (Livres de `null`)

**`SddCycle`**
* `cycle_id` (`CycleId` — ex.: `issue-45-202610271230`)
* `issue_linkage` (`IssueLinkage` — `LinkedIssue { number, url, board_column }` | `UnlinkedLocalCycle { directory_slug }`)
* `methodology_profile` (`MethodologyProfile` — `StandardGauntletV2` | `StructuredSdd { feature_dir }`)
* `current_gate` (`GateStage` — `Gate0Discovery` .. `Gate6Certification`)
* `lifecycle_status` (`CycleStatus` — `Active` | `Paused` | `CircuitBreakerTripped` | `Certified` | `Merged`)
* `gauntlet_attempt` (`GauntletAttemptCount` — inteiro limitado ao intervalo `0..=5`)
* `spec_seal_state` (`SpecLockState` — `DraftUnverified` | `Frozen { spec_path, blake3_digest }`)
* `work_branch` (`BranchName`)
* `pre_existing_dirty_files` (`Vec<RepoRelativePath>` — vazio `[]` quando árvore inicial está limpa)

**`TraceabilityLink`**
* `trace_id` (`TraceId`)
* `cycle_id` (`CycleId`)
* `business_rule_id` (`BusinessRuleId` — ex.: `RN-001`)
* `functional_req_id` (`FunctionalRequirementId` — ex.: `RF-001`)
* `task_id` (`TaskId` — ex.: `TASK-001`)
* `mermaid_journey_node` (`MermaidNodeId` — ex.: `D[Estado de Sucesso Confirmado]`)
* `playwright_spec_path` (`RepoRelativePath` — ex.: `loggiq/tests/e2e/issue-45.spec.ts`)
* `evidence_screenshot_name` (`ScreenshotFileName` — ex.: `01-sucesso.png`)

**`AtomicTask`**
* `task_id` (`TaskId` — ex.: `TASK-001`)
* `cycle_id` (`CycleId`)
* `wave_index` (`WaveNumber` — `1..=4`)
* `architectural_layer` (`ArchitecturalLayer`)
* `target_test_file` (`RepoRelativePath`)
* `tdd_lifecycle_state` (`TddLifecycleState` — `AwaitingRedPhase` | `RedVerified { failing_evidence_id }` | `GreenVerified { red_evidence_id, green_evidence_id }` | `Committed { commit_sha }`)
* `worktree_allocation` (`WorktreeAllocation` — `NotProvisioned` | `Provisioned { worktree_path }` | `MergedAndPruned { commit_sha }`)

**`ExecutionEvidence`**
* `evidence_id` (`EvidenceId`)
* `cycle_id` (`CycleId`)
* `gate_stage` (`GateStage`)
* `actor_role` (`ActorRole` — `Orchestrator` | `Builder(TaskId)` | `AstIntegrityAuditor` | `DomainAdversarialCritic` | `ContractComplianceCritic` | `StaticSecurityCritic` | `PlaywrightVisualCritic`)
* `executed_command` (`CommandLineString`)
* `exit_code` (`ProcessExitCode`)
* `duration_ms` (`DurationMillis`)
* `sanitized_output_digest` (`SanitizedOutput`)
* `verdict` (`GateVerdict` — `Pass` | `Fail` | `Blocked`)
* `recorded_at` (`UtcTimestamp`)

---

## 11. Métricas de Sucesso

### 11.1. Métricas de Negócio
* **Redução de 40% a 60% no Tempo Total de Ciclo (Wall-Clock Time)** de uma issue completa (do `init` ao `done`), eliminando turnos mecânicos de LLM e paralelizando ondas em worktrees.
* **Redução de 35% no Consumo de Tokens por Issue**, graças ao `AstContextSlicer` e ao bloqueio precoce de erros na Camada 0 (`< 5ms`) antes de acionar o Swarm de Críticos.

### 11.2. Métricas de Produto
* **100% de Conformidade nas 16 Invariantes:** Zero PRs ou commits gerados com emojis, trailers de IA, links `raw.githubusercontent` ou testes adulterados.
* **100% de Rastreabilidade Fechada:** Toda issue encerrada possui correspondência verificada matematicamente entre `RN -> RF -> TASK -> Playwright E2E -> Screenshot`.

### 11.3. Métricas Operacionais
* **Cold-start do binário `trustkit`:** `< 3ms` (medido via `hyperfine`).
* **Tempo de execução do Gate 0 completo (`trustkit init`):** `< 250ms` (operações locais + chamadas de API concorrentes).
* **Tempo de execução da Camada 0 do Gauntlet (`trustkit gauntlet ast-audit`):** `< 10ms`.

---

## 12. Riscos e Dependências

### 12.1. Riscos

| Risco | Probabilidade | Impacto | Mitigação |
| :--- | :---: | :---: | :--- |
| **Conflito de Merge entre Worktrees de uma mesma Onda** | Média | Médio | O `WaveExecutionPlanner` no Gate 3 (`trustkit wave plan`) analisa a seção `Arquivos Esperados` de cada `TASK-NNN.md`; se duas tarefas tocarem o mesmo arquivo, adiciona automaticamente uma aresta de dependência entre elas, alocando-as em ondas sequenciais. |
| **Falsos Positivos no `TestIntegrityAuditor` (AST)** | Baixa | Médio | Caso uma refatoração legítima de contrato exija alterar um teste existente, o desenvolvedor humano pode autorizar explicitamente via `trustkit contract unlock --file <test-file> --reason "..."`, registrando a justificativa no banco de eventos. |
| **Consumo de Disco por Múltiplas Worktrees (`node_modules`)** | Média | Médio | O `WorktreeIsolationProvider` utiliza *reflinks* Copy-on-Write (`cp --reflink=always`) ou *symlinks* automáticos da pasta `node_modules` da árvore principal para dentro das worktrees efêmeras, reduzindo o tempo de criação para `< 20ms` e custo de disco para ~0 MB. |
| **Indisponibilidade Temporária da API do GitHub (`gh`)** | Baixa | Baixo | Retornar o Caso Especial `BoardSyncOutcome::DeferredOffline`, gravando a transição pendente no SQLite para reenvio automático no próximo comando sem travar o trabalho local. |

### 12.2. Dependências
* Toolchain Rust estável (`rustc >= 1.82`, `cargo`).
* Git CLI (`>= 2.38`) e GitHub CLI (`gh >= 2.40`) autenticados na máquina hospedeira.
* Gramáticas oficiais do `tree-sitter` embarcadas estaticamente no binário (`tree-sitter-typescript`, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-json`).

---

## 13. Cronograma Estimado (MVP)

| Fase | Duração | Deliverables (Entregáveis) |
| :--- | :---: | :--- |
| **Fase 1: Fundação, CLI & SQLite WAL (`Gate 0`)** | Semana 1 | Workspace Cargo estruturado, banco `.trustkit/state.db` em modo WAL (zero `NULL`), `trustkit init`, integração concorrente com `gh` (`GitHubIdentityGuard` + `ProjectBoardSynchronizer`) e `trustkit hook` (`< 3ms`). |
| **Fase 2: Motor de Spec V2, `--from-prd` & DAG (`Gates 1, 2 e 3`)** | Semana 2 | Ingestor de PRD upstream (`PrdSpecificationIngestor`), validador de Mermaid/Gherkin/Matriz de Rastreabilidade, `trustkit spec lock` com `BLAKE3` (`ArtifactSeal`) e `WaveExecutionPlanner` (`trustkit wave plan`). |
| **Fase 3: Camada 0 AST (`tree-sitter`) & Context Slicer** | Semana 3 | Crate `trustkit-ast` com `TestIntegrityAuditor` (Anti-Cheat), `InvariantLexicalScanner` (zero emojis, zero IA trailers, zero `raw.githubusercontent`) e `AstContextSlicer` para os Builders. |
| **Fase 4: Isolador de Worktrees, TDD Guard & Gauntlet V2 (`Gates 4, 5 e 6`)** | Semana 4 | `WorktreeIsolationProvider`, fiscalizador `TddPhaseGuard` (`RED -> GREEN`), despachante paralelo (`tokio`) + servidor MCP `stdio`, `VisualEvidenceGuard`, `trustkit done`, `trustkit pr` e a *Thin Skill* (`SKILL.md` v4 enxuta). |

---

## Apêndice: Racional de Nomenclatura e Identidade Arquitetural (TRustKit)

A escolha do nome **TRustKit** consolida a fusão conceitual de **"True Rust"** com o ferramental (*Kit*) de governança de ciclo de vida para desenvolvimento assistido por agentes de IA. Longe de ser apenas um artifício fonético, o prefixo **T** define formalmente os pilares estruturais e operacionais que diferenciam este harness das implementações tradicionais de mercado:

### 1. "True Rust" (Nativo vs. Runtimes Interpretados)

A maioria dos harnesses de agentes no mercado opera sobre Python ou Node.js, arcando com o custo de inicialização de interpretadores (300ms a 2s por sub-processo), alto consumo de memória e latência severa em hooks síncronos de ciclo de vida (`PreToolUse` / `PostToolUse`).

O **TRustKit** implementa um binário estático nativo em Rust (*True Rust*), garantindo:

* **Cold-start desprezível (~1 a 3ms):** Hooks e validadores executam instantaneamente no terminal sem quebrar a fluidez da sessão interativa do LLM.
* **Orquestração de baixo nível concorrente:** Uso direto de bibliotecas nativas de sistema (`git2`/`gix` para isolamento de worktrees com reflink copy-on-write e `tokio` para paralelização real dos builders), eliminando o *overhead* de chamadas shell externas em cadeia.

### 2. Determinismo e Eliminação de Alucinações (*Trust*)

A sobreposição fonética intencional com o termo em inglês **"Trust"** (confiança) estabelece a fronteira entre a criatividade probabilística do LLM e a integridade matemática do código:

* **Invariantes determinísticos:** Validação de conformidade estrutural em microssegundos (fechamento formal da matriz $\text{RN} \rightarrow \text{RF} \rightarrow \text{TASK} \rightarrow \text{E2E}$, integridade de blocos Mermaid e critérios Gherkin).
* **Análise estática via AST (tree-sitter):** O binário audita alterações estruturais diretamente na árvore sintática, barrando quebra de contratos, adulteração de testes ou vazamento de escopo antes de acionar turnos caros de LLM para auditoria reflexiva.

### 3. Especificação do Nome e Mapeamento CLI

* **Nome Formal do Projeto:** `TRustKit`
* **Binário CLI:** `trustkit` (todo em minúsculas para conformidade ergonômica com ambientes POSIX)
* **Slogan / Tagline do Produto:** *"True Rust performance for trusted, spec-driven agentic execution."*
