# Analise: Alinhamento do TRustKit com o Elephant-Goldfish Model (EGM)

Data da Analise: 2026-11-03  
Status: Aprovado para Pesquisa, Arquitetura e Pitch  
Referencia Externa: [Introducing the Elephant-Goldfish Model Plugin (Google Open Source Blog, Out/2026)](https://opensource.googleblog.com/2026/10/introducing-elephant-goldfish-model-plugin.html)  
Escopo: Avaliacao comparativa e arquitetural do nivel de alinhamento entre o motor deterministico de governanca TRustKit e a tese do Elephant-Goldfish Model (EGM) formulada por Dave Rensin (Google).

---

## 1. Visao Geral e Tese Central

O TRustKit esta **profundamente alinhado (praticamente 1:1)** com a filosofia central do **Elephant-Goldfish Model (EGM)**, conceito formulado por Dave Rensin (VP na Google) e lancado como plugin open-source no Google Open Source Blog em outubro de 2026.

Na pratica, o TRustKit nao apenas adota a tese do EGM, mas a eleva de um **acordo comportamental via prompts** para uma **infraestrutura deterministica e mecatronica implementada em Rust nativo**.

---

## 2. O que propoe o Elephant-Goldfish Model (EGM)?

O artigo e o plugin do Google foram criados para combater a principal causa de degradacao em codificacao assistida por IA: o **conversational drift** (desvio conversacional por vies de confirmacao, degradacao do KV-cache e saturacao de contexto quando um agente comeca a escrever codigo diretamente em threads longas com mais de 100 mensagens).

O modelo separa o processo em duas personas e um mecanismo de checagem:

1. **The Elephant (`/egm-brainstorm`, `/egm-draft`)**: Sessao conversacional de contexto amplo e longo horizonte. Entrevista o engenheiro, explora restricoes arquiteturais e redige uma especificacao formal em Markdown (`spec.md`).  
   **Regra petrea:** O Elefante escreve **zero linhas de codigo de producao**.
2. **The Goldfish (`/egm-review`)**: Subagentes descartaveis, instanciados com **zero memoria** da conversa anterior. Eles realizam uma leitura fria (*cold-read*) da especificacao e do codigo de forma adversaria, expondo premissas implicitas, ambiguidades e alucinacoes que o Elephant e o desenvolvedor ignoraram.
3. **Drift Check (`/egm-drift-report`)**: Ferramentas mecanicas que verificam se a implementacao final nao se desviou da especificacao aprovada.

---

## 3. Mapeamento Direto: EGM vs. TRustKit

| Conceito EGM (Google) | Implementacao no TRustKit | Como Funciona no TRustKit |
| :--- | :--- | :--- |
| **The Elephant**<br>*(Contexto longo, especificacao formal, zero codigo)* | **Gates 1 & 2**<br>`trustkit spec prompt`<br>`trustkit spec lint`<br>`trustkit spec lock` | O agente arquiteto interage para levantar requisitos usando fatiamento de contexto AST e busca semantica FTS5 na memoria perene. Nenhuma linha de codigo de producao e autorizada ate o Gate 4. A spec e congelada com selo criptografico **BLAKE3**, tornando-se imutavel. |
| **The Goldfish (Executor)**<br>*(Zero memoria, sem vies da sessao de brainstorm)* | **Gates 3 & 4**<br>`trustkit wave plan`<br>`trustkit wave run`<br>*(Git Worktrees Isoladas)* | A especificacao e decomposta em tarefas atomicas (`TASK-NNN.md`) em um DAG. Cada subagente executor (*builder*) roda isolado em sua propria **Git Worktree** (reflink Copy-on-Write) recebendo apenas o escopo cirurgico da tarefa (`trustkit context slice`), com **zero contexto** da discussao original. |
| **The Goldfish (Revisor Adversario)**<br>*(Leitura fria e critica independente)* | **Gate 4 Refactor & Gate 5**<br>`trustkit task review`<br>`trustkit gauntlet ast-audit`<br>`trustkit gauntlet run` | - **Refactor Code Review**: Avaliacao do diff sem premissas previas sob multiplos angulos.<br>- **Camada 0 (AST nativo)**: Auditoria sintatica e *anti-cheat* via Tree-Sitter (< 3ms).<br>- **Camada 1 (Swarm do Gauntlet)**: Subagentes independentes e concorrentes rodando linter, testes e validacao visual Playwright.<br>- **Security Review**: Auditoria AppSec com filtro de confianca `>= 8/10`. |
| **Drift Check**<br>*(Verificacao de conformidade codigo vs. spec)* | **Gates 4, 5 & 6**<br>`trustkit task test`<br>`trustkit done` | - **TDD Mecanico**: O binario exige registro previo comprovado da fase `RED` antes de autorizar a fase `GREEN`.<br>- **Matriz de Rastreabilidade**: Cada `RN-*` e `RF-*` da spec precisa ter evidencia de execucao auditavel gravada em SQLite WAL (`.trustkit/state.db`).<br>- **Certificacao**: O `gauntlet-report.md` atesta que o entregavel e matematicamente conforme a spec travada no Gate 2. |

---

## 4. Onde o TRustKit vai ALEM do Elephant-Goldfish Model

O plugin oficial do EGM da Google opera principalmente no nivel de **prompts, skills e convencoes de chat** (via slash commands em Claude Code, Antigravity ou Codex). O TRustKit adota a mesma tese, mas substitui a dependencia da "boa vontade da LLM" por **garantias mecanicas em tempo de compilacao**:

1. **Arquitetura Hybrid Rust Core + Thin Skill**:
   - A camada de IA recebe apenas instrucoes minimas de orquestracao (`trustkit-sdd/SKILL.md`).
   - 100% das mutacoes de estado, bloqueios de portao e validacoes sao executados pelo binario nativo `trustkit` compilado em Rust (*Zero LLM calls, zero heavy regex, zero panic*).
2. **Isolamento de Sistema de Arquivos (Git Worktrees com CoW)**:
   - Enquanto o EGM isola apenas a *janela de contexto da IA* (limpando a thread), o TRustKit isola o **filesystem do repositorio**. Varios "Goldfish" constroem tarefas em paralelo sem risco de race conditions ou contaminacao cruzada de arquivos.
3. **Auditoria AST Anti-Cheat (Camada 0)**:
   - O TRustKit nao confia cegamente no reporte do agente. O analisador Tree-Sitter inspeciona a AST do codigo de teste para verificar se o Goldfish de fato escreveu assercoes validas ou se tentou burlar regras com mocks vazios ou testes tautologicos (`assert(true)`).
4. **Memoria Perene e Grafo Estrutural**:
   - Para que o Elephant formule boas specs sem alucinar a arquitetura existente, o TRustKit prove o motor de grafo estrutural e busca em texto completo com SQLite FTS5 e BM25, entregando fatias cirurgicas de codigo (*context slicing*).

---

## 5. Conclusao

O alinhamento e **estrutural e total**. O TRustKit pode ser considerado uma **materializacao de nivel de engenharia de sistemas do Elephant-Goldfish Model**: enquanto o EGM define a divisao cognitiva ideal (Elephant arquiteto sem codigo + Goldfish revisor/executor sem memoria), o TRustKit fornece o motor deterministico em Rust que torna essa separacao inviolavel.
