# Analise Comparativa e Levantamento Tecnico: mattpocock/skills vs TRustKit

Data da Analise: 2026-11-11  
Versao do Levantamento: 20261111-V3  
Status: Consolidado e Desdobrado em 6 Especificacoes Executaveis  
Repositorio Analisado: `https://github.com/mattpocock/skills`  
Repositorio Alvo: `/home/mmc/00_code/00_tools/TRustKit`  

---

## 1. Resumo Executivo e Filosofia Arquitetural

No manifesto de abertura de `https://github.com/mattpocock/skills`, Matt Pocock critica frameworks declarativos pesados:

> "Developing real applications is hard. Approaches like GSD, BMAD, and Spec-Kit try to help by owning the process. But while doing so, they take away your control and make bugs in the process hard to resolve. These skills are designed to be small, easy to adapt, and composable."

O **TRustKit** ja superou estruturalmente as limitacoes de latencia, perda de controle e alucinacao de estado do Spec-Kit tradicional ao migrar a maquina de estados para um **Core Nativo em Rust (< 3ms)** sob arquitetura hibrida (`Hybrid Rust Core + Thin Skill`), apoiado em:
- Persistencia transacional SQLite WAL (`.trustkit/state.db`) com Event Sourcing e hashes BLAKE3 (`crates/trustkit-db/`).
- Isolamento fisico de execucao concorrente via Git Worktrees Copy-on-Write e multiplexacao de pseudoterminais PTY com telemetria OSC 7501 (`crates/trustkit-orch/`).
- Inteligencia de grafo estrutural e auditoria sintatica de Camada 0 via Tree-Sitter (`crates/trustkit-ast/`).
- Orientacao deterministica de proximo passo a cada turno via `NextStepResolver` (`crates/trustkit-core/src/next_step_guidance.rs`).

Contudo, enquanto o **TRustKit e imbativel em governanca mecanica, topologia AST e determinismo de estado**, o conjunto `mattpocock/skills` destila **heuristicas classicas de engenharia de software** em disciplinas acionaveis para reduzir a entropia arquitetural gerada por agentes de IA:
1. **John Ousterhout** (*A Philosophy of Software Design*): Modulos Profundos (*Deep Modules*) vs. Modulos Rasos (*Shallow Modules*) e o padrao *Design It Twice*.
2. **Michael Feathers** (*Working Effectively with Legacy Code*): Costuras de software (*Seams*) como fronteira exclusiva de testes e acoplamento.
3. **Eric Evans** (*Domain-Driven Design*): Construcao ativa de Linguagem Ubiqua (*Ubiquitous Language*) em glossarios vivos (`GLOSSARY.md`) livre de jargao de implementacao.
4. **Martin Fowler** (*Refactoring: Improving the Design of Existing Code*): Baseline de 12 Code Smells estruturais separado da verificacao de conformidade de especificacao (*Spec Compliance*).
5. **David Thomas & Andrew Hunt** (*The Pragmatic Programmer*): Fatias verticais (*Tracer Bullets*), refatoracao em ondas *Expand-Contract* e loops cientificos de depuracao por reproducao minima.

---

## 2. Diagrama de Posicionamento e Sinergia com os 7 Gates do TRustKit

```text
+----------------------------------------------------------------------------------------------------------+
|                         TRustKit: Governanca Deterministica SDD (Gates 0 a 6)                            |
+----------------------------------------------------------------------------------------------------------+
     |                        |                        |                        |                      |
     v (Gate 1 & Gate 2)      v (Gate 3: DAG/Waves)    v (Gate 4: TDD/Review)   v (Gate 5: Gauntlet)   v (Gate 6: Cert/PR)
[Ideia 01 & Ideia 02]    [Ideia 03]               [Ideia 04 & Ideia 05]    [Ideia 01 & Ideia 06]  [Ideia 06]
- Glossario Ubiquo       - Topologia de Ondas     - Guardiao de Seam e     - Auditoria AST de     - Calculo de Merge
  Ativo (.trustkit/wiki)   Expand-Contract para     Detector AST de          Modulos Rasos          Danger (One-Way vs
- Triagem Estrita de       Wide Refactors           Testes Tautologicos      (Ousterhout Depth)     Two-Way Door) +
  ADRs (3 criterios)     - Calculo de Blast       - Revisao Dual-Axis:     - Protocolo de Loop      Blast Radius no PR
- Padrao Design It         Radius no grafo AST      Standards/Fowler vs      Reprodutor no        - Retrospectiva de
  Twice para Portas                                 Spec Scope Creep         trustkit doctor        Harness pos-ciclo
```

---

## 3. Catalogo Completo de `mattpocock/skills` e Referencias Primarias

### 3.1. Skills de Engenharia (`skills/engineering/`)

| Skill | Tipo de Invocacao | Referencia no Repositorio `mattpocock/skills` | Conceito Central |
| :--- | :--- | :--- | :--- |
| `codebase-design` | Model-invoked | `skills/engineering/codebase-design/SKILL.md` e `DESIGN-IT-TWICE.md` | Vocabulario formal (*Module*, *Interface*, *Implementation*, *Depth*, *Seam*, *Adapter*, *Leverage*, *Locality*), *Deletion Test* e *Design It Twice* com subagentes paralelos. |
| `improve-codebase-architecture` | User-invoked | `skills/engineering/improve-codebase-architecture/SKILL.md` | Inspecao de hot spots no `git log` para encontrar modulos rasos (*shallow modules*) e oportunidades de aprofundamento (*deepening*), gerando relatorio visual Antes/Depois. |
| `domain-modeling` | Model-invoked | `skills/engineering/domain-modeling/SKILL.md` | Modelagem ativa de dominio: confrontar termos vagos contra `GLOSSARY.md`, testar limites com cenarios de borda, atualizar glossario inline e filtrar ADRs por 3 criterios estritos. |
| `grill-with-docs` | User-invoked | `skills/engineering/grill-with-docs/SKILL.md` | Entrevista implacavel de requisitos combinada com `domain-modeling` para alinhar expectativas e consolidar `GLOSSARY.md` e `docs/adr/` antes da especificacao. |
| `to-spec` | User-invoked | `skills/engineering/to-spec/SKILL.md` | Sintese de especificacao definindo explicitamente as costuras (*seams*) de mais alto nivel onde os testes serao acoplados. |
| `to-tickets` | User-invoked | `skills/engineering/to-tickets/SKILL.md` | Decomposicao em fatias verticais (*tracer bullets*) com arestas bloqueantes, exceto para *Wide Refactors*, que exigem sequenciamento *Expand -> Migrate -> Contract*. |
| `tdd` | Model-invoked | `skills/engineering/tdd/SKILL.md` | Ciclo Red-Green focado em interfaces publicas (*pre-agreed seams*) e proibicao de 3 anti-padroes: *Implementation-coupled*, *Tautological* e *Horizontal slicing*. |
| `code-review` | Model-invoked | `skills/engineering/code-review/SKILL.md` | Revisao paralela em dois eixos incomunicaveis: *Standards* (regras do repo + 12 smells de Fowler) e *Spec* (requisitos faltantes, implementacao incorreta e *scope creep*). |
| `diagnosing-bugs` | Model-invoked | `skills/engineering/diagnosing-bugs/SKILL.md` | Loop disciplinado em 6 fases: (1) construir feedback loop que fica vermelho, (2) minimizar repro, (3) formular 3-5 hipoteses falsificaveis, (4) instrumentar com `[DEBUG-xxxx]`, (5) corrigir na costura certa, (6) limpar. |
| `pr` | Model-invoked | `skills/engineering/pr/SKILL.md` | Padronizacao de Pull Request com diagrama minimo do diff, evidencia Antes/Depois e declaracao de *Merge Danger* (*One-Way Door* vs. *Two-Way Door* + *Blast Radius*). |
| `retro` | User-invoked | `skills/engineering/retro/SKILL.md` | Retrospectiva de sessao focada em evoluir o ambiente do agente: ponteiros de navegacao, conversao de regras em checagens mecanicas, economia de ferramentas e limpeza de no-ops. |
| `wayfinder` | User-invoked | `skills/engineering/wayfinder/SKILL.md` | Planejamento de escopos nebulosos multi-sessao separando *Decisions so far*, *Frontier tickets* e *Fog of war* (*Not yet specified* vs. *Out of scope*). |
| `implement` / `implement-spec` | User-invoked | `skills/engineering/implement-spec/SKILL.md` | Execucao concorrente da fronteira de tickets prontos em uma branch de integracao seguida de code review. |
| `setup-matt-pocock-skills` / `triage` | User-invoked | `skills/engineering/triage/SKILL.md` | Configuracao de issue tracker e maquina de estados baseada em labels (`ready-for-agent`). |
| `prototype` / `research` / `wizard` | Model-invoked | `skills/engineering/prototype/SKILL.md` | Prototipacao descartavel em arquivo unico, pesquisa em background com citacoes primarias e gerador de wizard bash para tarefas exclusivas de humanos (HITL). |

### 3.2. Skills de Produtividade (`skills/productivity/`)

| Skill | Referencia no Repositorio `mattpocock/skills` | Conceito Central |
| :--- | :--- | :--- |
| `grill-me` | `skills/productivity/grill-me/SKILL.md` | Entrevista em arvore de decisao ate resolver todas as ramificacoes abertas de um plano. |
| `handoff` | `skills/productivity/handoff/SKILL.md` | Compactacao estruturada de contexto entre sessoes de agentes. |
| `writing-for-agents` | `skills/productivity/writing-for-agents/` | Guia de estilo conciso para documentos consumidos por agentes (alta densidade semantica, ponteiros de navegacao, separacao entre contexto de implementacao e contexto de revisao). |

---

## 4. O Que o TRustKit Ja Executa de Forma Superior (Nao Importar)

Nao se deve instalar o pacote `mattpocock/skills` diretamente sobre um repositorio governado pelo TRustKit, pois as skills de fluxo dele sao baseadas em instrucoes de prompt sem garantias criptograficas ou isolamento de sistema de arquivos:

1. **Orquestracao Concorrente (`implement-spec` vs. `trustkit-orch`):**
   - `implement-spec` roda subagentes na mesma arvore de arquivos, sujeito a condicoes de corrida de indice Git e colisao de compilacao.
   - O TRustKit isola fisicamente cada tarefa em uma Git Worktree Copy-on-Write (`crates/trustkit-orch/src/worktree_provider.rs`), gerencia processos em pseudoterminais dedicados (`crates/trustkit-orch/src/pty_socket_adapter.rs`) e orquestra ondas completas via `crates/trustkit-orch/src/autonomous_wave_orchestrator.rs`.
2. **Fiscalizacao de Ciclo TDD (`tdd` prompt vs. `TestIntegrityAuditor` + SQLite WAL):**
   - A skill `tdd` de Matt Pocock apenas aconselha o modelo a rodar o teste antes.
   - O TRustKit exige a gravacao transacional da falha comprovada (`trustkit task test <ID> --phase red`) antes do sucesso (`--phase green`) em `crates/trustkit-db/src/task_ops.rs` e valida a integridade das assercoes por AST em `crates/trustkit-ast/src/test_integrity_auditor.rs`.
3. **Governanca de Tracker e Estado (`triage` / `handoff` vs. `trustkit-gh` / `NextStepResolver`):**
   - O TRustKit sincroniza deterministicamente as 9 colunas do GitHub Projects v2 (`crates/trustkit-gh/src/board_synchronizer.rs`), protege a identidade da conta Git (`crates/trustkit-gh/src/identity_guard.rs`) e elimina a necessidade de `handoff` manual gracas ao `NextStepResolver` (`crates/trustkit-core/src/next_step_guidance.rs`) e ao banco `.trustkit/state.db`.

---

## 5. Detalhamento das 6 Ideias Selecionadas para Evoluir o TRustKit

Abaixo estao detalhadas as 6 frentes de evolucao derivadas desta pesquisa, com mapeamento exato para os arquivos e crates do TRustKit e referencia para as respectivas especificacoes executaveis geradas nesta pasta.

### Ideia 01: Auditoria AST de Modulos Profundos (Ousterhout), Deletion Test e Design It Twice
- **Inspiracao:** `codebase-design/SKILL.md`, `codebase-design/DESIGN-IT-TWICE.md`, `improve-codebase-architecture/SKILL.md`.
- **Gap atual no TRustKit:** O `GraphHealthAuditor` (`crates/trustkit-ast/src/graph_health_auditor.rs`) detecta God Nodes (excesso de acoplamento) e violacoes de camadas hexagonais, mas nao avalia a **profundidade dos modulos (Module Depth)**. Agentes frequentemente criam dezenas de funcoes ou structs *pass-through* (modulos rasos onde a assinatura publica e quase tao complexa quanto o corpo), degradando a localidade de manutencao.
- **Solucao proposta no TRustKit:**
  1. Estender `trustkit-core/src/graph_entities.rs` e `trustkit-ast/src/graph_health_auditor.rs` para extrair metricas de **Module Depth Ratio** (razao entre superficie de interface publica exportada e volume/complexidade de implementacao oculta) e aplicar a heuristica mecanica do **Deletion Test** (identificar adaptadores ou funcoes puramente *pass-through* com um unico chamador ou delegacao direta).
  2. Adicionar ao `trustkit spec prompt` (`crates/trustkit-core/src/spec_scaffold_entities.rs`) o protocolo opcional **Design It Twice** (`--design-it-twice`), gerando briefings estruturados para avaliar 3 contrapontos de interface na camada de portas (`ports.rs`) antes do congelamento no Gate 2.
- **Especificacao vinculada:** `./spec-01-deep-modules-and-seam-analysis.md`

### Ideia 02: Glossario Ativo de Linguagem Ubiqua e Triagem Deterministica de ADRs
- **Inspiracao:** `domain-modeling/SKILL.md`, `grill-with-docs/SKILL.md`.
- **Gap atual no TRustKit:** Embora o Principio Constitucional VI exija Linguagem Ubiqua e o TRustKit possua a wiki FTS5 em `.trustkit/wiki/` (`crates/trustkit-db/src/knowledge_ops.rs`), nao existe um artefato canonico de glossario verificado pelo `SpecLinter` (`crates/trustkit-ast/src/spec_linter.rs`), nem um filtro formal para criacao de ADRs durante o ciclo SDD.
- **Solucao proposta no TRustKit:**
  1. Instituir o glossario vivo em `.trustkit/wiki/glossary.md` gerenciado por comandos dedicados (`trustkit memory glossary`) com termos canonicos, definicoes de dominio isentas de detalhes de implementacao e lista de sinonimos proibidos (*rejected terms / fuzzy terms*).
  2. Integrar a verificacao de vocabulario ao `SpecLinter` (`crates/trustkit-ast/src/spec_linter.rs`): se a `spec.md` utilizar um termo banido/ambiguo listado no glossario do projeto, o linter emite um diagnostico estruturado sugerindo o termo canonico.
  3. Adicionar validacao de triagem de ADRs (3 criterios obrigatorios: *Reversibility Cost*, *Context Surprise*, *Real Alternatives Trade-off*) antes de persistir decisoes arquiteturais na memoria viva.
- **Especificacao vinculada:** `./spec-02-ubiquitous-glossary-and-adr-triage.md`

### Ideia 03: Topologia de Ondas Expand-Contract para Wide Refactors e Calculo de Blast Radius
- **Inspiracao:** `to-tickets/SKILL.md` (secoes *Vertical Slices* vs. *Wide Refactors: Expand-Contract*).
- **Gap atual no TRustKit:** O `WaveCouplingAnalyzer` (`crates/trustkit-ast/src/wave_coupling_analyzer.rs`) e o `WaveExecutionPlanner` (`crates/trustkit-ast/src/wave_planner.rs`) agrupam tarefas em ondas aciclicas baseadas nas dependencias declaradas e alertam sobre sobreposicao de arquivos. Contudo, quando uma feature exige uma refatoracao ampla (*Wide Refactor* em um trait ou struct de alto `in_degree` em `trustkit-core`), fatiar horizontalmente ou tentar migrar tudo em uma unica tarefa paralela quebra a compilacao entre worktrees.
- **Solucao proposta no TRustKit:**
  1. Calcular o **Blast Radius Estrutural** de cada tarefa no Gate 3 cruzando os arquivos/simbolos alvo de `tasks/TASK-NNN.md` com o `in_degree` no grafo AST (`crates/trustkit-ast/src/wave_coupling_analyzer.rs`).
  2. Quando uma tarefa modificar um simbolo com `in_degree` acima do limiar critico (Wide Refactor), validar e orientar a decomposicao topologica em 3 ondas estritas: **Onda Expand** (introducao aditiva retrocompativel) -> **Onda Migrate** (lotes paralelos em Git Worktrees) -> **Onda Contract** (remocao do simbolo legado bloqueada por todas as tarefas de migracao).
- **Especificacao vinculada:** `./spec-03-expand-contract-wave-topology.md`

### Ideia 04: Deteccao AST de Testes Tautologicos e Guardiao de Costuras Publicas (Pre-Agreed Seams)
- **Inspiracao:** `tdd/SKILL.md`, `to-spec/SKILL.md`.
- **Gap atual no TRustKit:** O `TestIntegrityAuditor` (`crates/trustkit-ast/src/test_integrity_auditor.rs`), apoiado em `rust_assertion_counter.rs` e `ts_assertion_counter.rs`, impede que agentes apaguem assercoes ou usem `.skip`. Entretanto, um agente ainda pode burlar a qualidade do teste escrevendo uma **assercao tautologica** (ex: `assert_eq!(x, x)` ou `assert_eq!( compute(a, b), a + b )` replicando a formula de producao sem um valor esperado literal/fixture independente) ou acoplando testes diretamente a detalhes internos em vez de uma costura publica (*Seam*).
- **Solucao proposta no TRustKit:**
  1. Aprimorar `RustAssertionCounter` (`crates/trustkit-ast/src/rust_assertion_counter.rs`) e `TsAssertionCounter` (`crates/trustkit-ast/src/ts_assertion_counter.rs`) para inspecionar a arvore sintatica dos argumentos de `assert_eq!`, `assert!`, `expect().toBe()` e barrar assercoes identicas (`left == right` sintaticamente iguais) ou tautologias triviais na Camada 0 do Gauntlet.
  2. Evoluir o `SpecValidator` (`crates/trustkit-ast/src/spec_validator.rs`) e o `TestIntegrityAuditor` para validar que os testes listados na Matriz de Rastreabilidade exercitam simbolos/traits publicos (costuras validas / *Seams*) definidos na arquitetura hexagonal.
- **Especificacao vinculada:** `./spec-04-tautological-test-and-seam-guard.md`

### Ideia 05: Revisao Semantica em Dois Eixos Independentes (Standards & Fowler Smells vs. Spec Scope Creep)
- **Inspiracao:** `code-review/SKILL.md`.
- **Gap atual no TRustKit:** No Gate 4 (`trustkit task review`), as entidades em `crates/trustkit-core/src/semantic_review_entities.rs` modelam 8 angulos (`LogicCorrectness`, `RemovedBehavior`, `CrossFileContract`, `Reuse`, `Simplification`, `Efficiency`, `Altitude`, `ProjectConventions`) consolidados em um unico ranking. Isso permite que um codigo perfeitamente alinhado aos padroes do projeto mas que introduziu *Scope Creep* (comportamento nao solicitado na `spec.md`) ou *Code Smells* estruturais classicos de Fowler passe despercebido.
- **Solucao proposta no TRustKit:**
  1. Separar o resultado do `trustkit task review` (`crates/trustkit-core/src/semantic_review_entities.rs`) em **dois eixos independentes e nao-compensaveis** (`DualAxisReviewReport`):
     - **Eixo 1 (`StandardsAndSmellsAxis`):** Conformidade com `AGENTS.md` + baseline deterministico/semantico dos 12 smells de Fowler (*Primitive Obsession*, *Data Clumps*, *Feature Envy*, *Shotgun Surgery*, *Divergent Change*, *Speculative Generality*, *Middle Man*, *Message Chains*, etc.).
     - **Eixo 2 (`SpecComplianceAxis`):** Verificacao estrita contra a `spec.md` congelada no Gate 2, auditando (a) requisitos ausentes/incompletos, (b) implementacao divergente da regra `RN-XXX` e (c) **Scope Creep** (codigo ou endpoints adicionados sem respaldo na spec).
  2. Garantir que uma falha no eixo `SpecComplianceAxis` reprove a revisao mesmo que o eixo `StandardsAndSmellsAxis` tenha zero apontamentos, e vice-versa.
- **Especificacao vinculada:** `./spec-05-dual-axis-review-and-fowler-smells.md`

### Ideia 06: Loop de Reproducao Minima no Doctor, Calculo de Merge Danger no PR e Retrospectiva de Harness
- **Inspiracao:** `diagnosing-bugs/SKILL.md`, `pr/SKILL.md`, `retro/SKILL.md`.
- **Gap atual no TRustKit:**
  1. Quando o Circuit Breaker do Gauntlet desarma no Gate 5, o `trustkit doctor` diagnostica o estado do banco, mas nao exige um loop de reproducao isolado nem bloqueia marcadores de instrumentacao temporaria (`[DEBUG-xxxx]`) esquecidos no codigo.
  2. O `PullRequestGateway` (`crates/trustkit-gh/src/pr_gateway.rs`) valida `Ref #<id>` e bloqueia `Closes #<id>`, mas nao informa ao revisor humano o nivel de perigo do merge (*One-Way Door* vs. *Two-Way Door*) nem o *Blast Radius* calculado pelo grafo AST.
  3. O fechamento do ciclo em `trustkit done` (`crates/trustkit-orch/src/certification_reporter.rs`) gera o `gauntlet-report.md`, mas nao analisa os eventos de friccao ocorridos durante a sessao no SQLite WAL para retroalimentar o harness.
- **Solucao proposta no TRustKit:**
  1. No `InvariantLexicalScanner` (`crates/trustkit-ast/src/lexical_scanner.rs`), bloquear probes de depuracao residuais (`[DEBUG-`) antes de permitir avanco nos Gates 5 e 6, e estruturar no `trustkit doctor` o guia de reproducao minima em 6 fases.
  2. No `PullRequestGateway` (`crates/trustkit-gh/src/pr_gateway.rs`) e `CertificationReporter`, calcular automaticamente o **Merge Danger** (*One-Way Door* se alterar migracoes SQLite, schemas ou portas publicas; *Two-Way Door* se restrito a adaptadores internos) e o **Blast Radius** (`Isolated`, `ModuleLocal`, `CrossCrate`, `SystemWide`).
  3. No `trustkit done`, consolidar uma secao de **Retrospectiva Deterministica de Harness** a partir do historico de falhas do ciclo no SQLite WAL, recomendando quais falhas recorrentes devem virar checagens mecanicas ou ponteiros de navegacao.
- **Especificacao vinculada:** `./spec-06-diagnostic-loop-merge-danger-and-retro.md`

---

## 6. Indice das Especificacoes Geradas nesta Pasta (`docs/research/20261111-V3/`)

Todas as 6 especificacoes abaixo foram redigidas seguindo o padrao V2 do TRustKit (Product Trio, Diagrama Mermaid `flowchart LR`, Criterios de Aceitacao Gherkin e Matriz de Rastreabilidade fechada) e validadas pelo `trustkit spec lint`:

1. [`spec-01-deep-modules-and-seam-analysis.md`](./spec-01-deep-modules-and-seam-analysis.md)
2. [`spec-02-ubiquitous-glossary-and-adr-triage.md`](./spec-02-ubiquitous-glossary-and-adr-triage.md)
3. [`spec-03-expand-contract-wave-topology.md`](./spec-03-expand-contract-wave-topology.md)
4. [`spec-04-tautological-test-and-seam-guard.md`](./spec-04-tautological-test-and-seam-guard.md)
5. [`spec-05-dual-axis-review-and-fowler-smells.md`](./spec-05-dual-axis-review-and-fowler-smells.md)
6. [`spec-06-diagnostic-loop-merge-danger-and-retro.md`](./spec-06-diagnostic-loop-merge-danger-and-retro.md)
