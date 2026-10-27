# Guia de Bootstrapping & Self-Hosting do TRustKit

**Documento:** Estratégia de Construção em 2 Estágios (Spec Kit + `mmc-gauntlet-sdd-v2` $\rightarrow$ `trustkit` Nativo)  
**Localização:** `/home/mmc/00_code/00_tools/TRustKit/docs/BOOTSTRAPPING_E_SELF_HOSTING.md`  
**Data:** Outubro/2026  
**Status:** Oficial / Pronto para Execução  

---

## 1. Visão Geral: O Padrão de Bootstrapping em Engenharia de Sistemas

Na construção de compiladores (como o próprio `rustc`) e ferramentas críticas de infraestrutura, utiliza-se o padrão de **Bootstrapping seguido de Self-Hosting (Dogfooding)**:

1. **Estágio 0 (Semente / Bootstrap):** Utiliza-se o harness atual (**Spec Kit + `mmc-gauntlet-sdd-v2`**) para especificar, planejar, quebrar em tarefas e construir o núcleo funcional mínimo do **TRustKit** (**Fases 1 e 2** do ADR-001).
2. **Ponto de Virada de Chave (The Cutover Gate):** Ao final da Fase 2, o binário `./target/release/trustkit` já compila e é capaz de executar os Gates de inicialização, bloqueio de hooks (`< 3ms`), congelamento criptográfico de Spec (`BLAKE3`), planejamento de ondas via DAG e fiscalização do ciclo TDD (`RED -> GREEN`).
3. **Estágio 1 (Self-Hosting / Dogfooding):** A partir da **Fase 3**, o próprio binário `trustkit` assume o controle do repositório `/home/mmc/00_code/00_tools/TRustKit`, governando a implementação das **Fases 3 e 4** dele mesmo.

```text
+-----------------------------------------------------------------------------------+
| ESTAGIO 0: BOOTSTRAP (Semanas 1 e 2 - Fases 1 e 2)                                |
| Harness Ativo: Spec Kit (.specify/) + Skill `/mmc-gauntlet-sdd-v2`                |
|                                                                                   |
|  [PRD_TRustKit.md + ADR_001]                                                      |
|        |                                                                          |
|        +--> Fase 1: `trustkit-core` (PORS/Ports) + `trustkit-db` (SQLite WAL)     |
|        |            + `trustkit-cli` (`init`, `status`, `hook pre-tool` < 3ms)    |
|        |                                                                          |
|        +--> Fase 2: `trustkit-ast` (Spec Validator, `--from-prd`, `spec lock`)    |
|                     + `PetgraphTopologyAdapter` (`wave plan`)                     |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
                        =====================================
                        [ MARCO DE VIRADA (THE CUTOVER GATE) ]
                        Binario `./target/release/trustkit`
                        compilado, testado e instalado no PATH
                        =====================================
                                          |
                                          v
+-----------------------------------------+-----------------------------------------+
| ESTAGIO 1: SELF-HOSTING / DOGFOODING (Semanas 3 e 4 - Fases 3 e 4)                |
| Harness Ativo: Binario Nativo `trustkit` + Thin Skill                             |
|                                                                                   |
|  O proprio `trustkit` governa os Gates, o `.trustkit/state.db` e o TDD para:      |
|        |                                                                          |
|        +--> Fase 3: `TestIntegrityAuditor` (tree-sitter Anti-Cheat),              |
|        |            `InvariantLexicalScanner` e `AstContextSlicer`                |
|        |                                                                          |
|        +--> Fase 4: `GitWorktreeReflinkAdapter` (Worktrees CoW), `TddPhaseGuard`, |
|                     `SwarmDispatcher` (tokio), `VisualEvidenceGuard` e MCP Server |
+-----------------------------------------------------------------------------------+
```

---

## 2. Por Que Usar Spec Kit + `mmc-gauntlet-sdd-v2` no Estágio 0?

1. **Blindagem Arquitetural via `constitution.md`:**
   Sem um harness no Estágio 0, um agente codificador trabalhando diretamente sobre o PRD/ADR tenderia a tomar atalhos (importar `rusqlite` ou `clap` dentro de `trustkit-core`, usar `unwrap()` em produção ou criar colunas `NULL` no SQLite). No Spec Kit, a `constitution.md` atua como lei suprema no Gate 2 (`/speckit-plan` e `/speckit-analyze`) e no Gate 5 (Gauntlet), bloqueando qualquer desvio do `ADR_001_TRustKit_Architecture.md`.
2. **O Próprio Repositório Serve como Massa de Teste Real:**
   O `TRustKit` precisa suportar nativamente o **Modo Spec Kit** (detectar `.specify/`, ler `constitution.md`, `spec.md`, `plan.md` e `tasks.md`). Ao construí-lo com o Spec Kit desde o dia 1, o repositório `/home/mmc/00_code/00_tools/TRustKit` terá artefatos reais de `.specify/` e `specs/001-.../` que servirão de *fixtures* autênticas para a suíte de testes de integração do próprio `trustkit`.
3. **Decomposição Controlada por Crate:**
   Em vez de tentar gerar as 6 crates simultaneamente, cada Fase do cronograma do PRD vira uma feature numerada no Spec Kit (`001-core-db-cli`, `002-spec-seal-dag`, `003-ast-anticheat-slicer`, `004-worktree-swarm-mcp`), mantendo o contexto enxuto e determinístico.

---

## 3. Adaptação Obrigatória para Projetos CLI em Rust (Equivalência de Invariantes)

A skill `mmc-gauntlet-sdd-v2` foi originalmente calibrada para aplicações fullstack TypeScript/Web (cobrando `npm run lint && npm run build` e Playwright E2E com screenshots de navegador). Como o `TRustKit` é um **binário estático CLI em Rust sem interface gráfica**, a `constitution.md` do projeto estabelece a tabela oficial de equivalência determinística para o Gauntlet:

| Portão / Exigência Original (`mmc-gauntlet-sdd-v2`) | Equivalente Estrito no Repositório `TRustKit` (Rust CLI) |
| :--- | :--- |
| **Lint & Typecheck (`npm run lint`)** | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (`Exit Code: 0`, zero warnings permitidos). |
| **Build de Produção (`npm run build`)** | `cargo build --workspace --release` (`Exit Code: 0`) + auditoria `cargo tree -p trustkit-core` provando zero dependências externas no domínio. |
| **Testes Unitários & Integração (`npm test`)** | `cargo test --workspace --lib --bins` (suíte unitária F.I.R.S.T. de `trustkit-core` rodando em `< 50ms`). |
| **Suite E2E (`npm run test:e2e` / Playwright)** | `cargo test --test 'e2e_*'` utilizando a crate `assert_cmd` + `predicates` + `tempfile` para invocar o binário compilado `trustkit` de ponta a ponta em repositórios Git temporários reais. |
| **Evidência do Gate 5/6 (`tests/evidence/`)** | Gravação dos logs literais de execução E2E e do benchmark `hyperfine --warmup 10 './target/release/trustkit status --json'` comprovando *cold-start* `< 3ms` em `docs/specs/<issue-id>-AAAAMMDDHHMM/tests/evidence/`. |

---

## 4. Constituição Oficial do Repositório (`.specify/memory/constitution.md`)

Ao inicializar o Spec Kit em `/home/mmc/00_code/00_tools/TRustKit`, grave o conteúdo abaixo em `.specify/memory/constitution.md` para que tanto o `/speckit-analyze` quanto os Críticos do `mmc-gauntlet-sdd-v2` fiscalizem as regras de *Clean Code* do `ADR-001`:

```markdown
# Constituicao Arquitetural do Projeto TRustKit (trustkit)

## 1. Isolamento Hexagonal Estrito (Clean Code Cap. 8)
- A crate `trustkit-core` NAO PODE conter nenhuma dependencia externa em seu `Cargo.toml` (apenas `std`).
- Bibliotecas como `rusqlite`, `tree-sitter`, `petgraph`, `blake3`, `gix`, `git2`, `tokio`, `clap` e `serde_json` sao estritamente restritas as crates de borda/infraestrutura (`trustkit-db`, `trustkit-ast`, `trustkit-orch`, `trustkit-gh`, `trustkit-cli`), acessadas pelo dominio exclusivamente via Traits definidas em `trustkit-core::ports`.

## 2. Proibicao Absoluta de NULL e Uso de Special Cases (Clean Code Cap. 7)
- Nenhuma coluna no schema SQLite (`trustkit-db`) pode ser nullable; todas as colunas exigem `NOT NULL` e valores explicitos de Caso Especial (`'UNLINKED_LOCAL_CYCLE'`, `'DRAFT_UNVERIFIED'`, `'NOT_PROVISIONED'`, `'[]'`).
- Nenhum payload JSON emitido pelo modo `--json` pode conter `: null`. Ausencias de dados devem ser modeladas via Enums tipados (*Special Case Pattern*) ou listas vazias `[]`.

## 3. Linguagem Ubiqua e Tamanho de Modulos (Clean Code Cap. 2)
- Sao proibidos nomes de structs, traits ou modulos com sufixos genericos: `Manager`, `Processor`, `Data`, `Info`, `Helper`, `Util`.
- Nenhum arquivo `.rs` pode ultrapassar 250 linhas de codigo. Funcoes devem ter responsabilidade unica.

## 4. Tratamento de Erros sem Panics em Producao (Clean Code Cap. 7)
- E proibido o uso de `.unwrap()`, `.expect()`, `panic!()` ou `todo!()` no codigo de producao das crates (permitido apenas em blocos `#[cfg(test)]`).
- Todo erro de biblioteca externa deve ser capturado no Adapter e traduzido na borda para `trustkit_core::errors::DomainViolation`.
- Apenas `trustkit-cli::error_handler::CliBoundaryErrorHandler` pode mapear `DomainViolation` para `ExitCode` POSIX (`0` a `6`) e escrever em `stderr`/`stdout`.

## 5. Separacao Construcao vs Execucao (Clean Code Cap. 11)
- Proibido o uso de Singletons globais mutaveis ou *Service Locators* no dominio.
- Toda a montagem e injecao de dependencias ocorre exclusivamente em `trustkit-cli::bootstrap`.
- O subcomando `trustkit hook` nao pode inicializar o runtime `tokio`, devendo manter cold-start `< 3ms`.

## 6. Governanca Git, Zero Emojis e Zero Atribuicao a IA
- Proibicao total de emojis ou icones Unicode em codigo, comentarios, commits, specs, issues e PRs.
- Proibicao total de trailers `Co-Authored-By: Claude...` ou mencoes a assistentes de IA.
```

---

## 5. Roteiro Passo a Passo — Estágio 0 (Construindo as Fases 1 e 2)

No **Estágio 0**, você utilizará o pipeline **Spec Kit + `/mmc-gauntlet-sdd-v2`** (no Modo Spec Kit ou Modo Padrão com `--from-prd`) para construir as duas primeiras fases:

### Passo 0.1: Preparação do Repositório Git e `.specify/`
1. Inicializar o repositório Git em `/home/mmc/00_code/00_tools/TRustKit` com branch base `develop`.
2. Configurar `.specify/memory/constitution.md` com as regras da Seção 4 acima.
3. Referenciar `docs/PRD_TRustKit.md` e `docs/ADR_001_TRustKit_Architecture.md` como documentos-fonte.

### Passo 0.2: Execução da Fase 1 (`001-core-db-cli`)
**Escopo da Fase 1:** Workspace Cargo + `trustkit-core` (Entidades PORS, Special Cases, Ports, `DomainViolation`, `gates_fsm`) + `trustkit-db` (`SqliteWalRepositoryAdapter` e `Blake3ParallelSealerAdapter`) + `trustkit-cli` (`bootstrap`, `error_handler`, `trustkit init`, `trustkit status`, `trustkit hook pre-tool`) + `trustkit-gh` (`GitHubIdentityGuard` e `ProjectBoardSynchronizer`).

1. **Se usar Spec Kit + Gauntlet:**
   * `/speckit-specify` $\rightarrow$ Gera `specs/001-core-db-cli/spec.md` extraindo os requisitos `RF-001`, `RF-013` e a modelagem SQLite do `PRD_TRustKit.md` e `ADR_001_TRustKit_Architecture.md`.
   * `/speckit-plan` $\rightarrow$ Gera `specs/001-core-db-cli/plan.md` validado contra a `constitution.md`.
   * `/speckit-tasks` $\rightarrow$ Gera `specs/001-core-db-cli/tasks.md` organizado por ondas TDD.
   * `/mmc-gauntlet-sdd-v2 spec` $\rightarrow$ Gera a rubrica adversarial `gauntlet-rubric.md`.
   * `/mmc-gauntlet-sdd-v2 start` $\rightarrow$ Os Builders implementam em ciclo TDD (`RED -> GREEN -> REFACTOR` via `cargo test`).
   * `/mmc-gauntlet-sdd-v2 test-gauntlet` $\rightarrow$ Os 4 Críticos validam pureza do `trustkit-core`, ausência de `NULL` no SQLite, zero `unwrap()` e *cold-start* `< 3ms` do `trustkit hook`.
2. **Se usar diretamente `/mmc-gauntlet-sdd-v2` com `--from-prd` (Sem Spec Kit):**
   * `/mmc-gauntlet-sdd-v2 spec "Fase 1: Nucleo Hexagonal, SQLite WAL, Init e Fast Hooks" --from-prd docs/PRD_TRustKit.md`
   * `/mmc-gauntlet-sdd-v2 atomizer`
   * `/mmc-gauntlet-sdd-v2 start`
   * `/mmc-gauntlet-sdd-v2 test-gauntlet`

### Passo 0.3: Execução da Fase 2 (`002-spec-seal-dag`)
**Escopo da Fase 2:** `PrdSpecificationIngestor` (`--from-prd`), validador de Mermaid/Gherkin/Matriz de Rastreabilidade (`spec_validator.rs`), `SealSpecificationUseCase` (`trustkit spec lock` com hash `BLAKE3`) e `PetgraphTopologyAdapter` (`trustkit wave plan`).

* Repete-se o mesmo fluxo do Passo 0.2. Ao término da Fase 2, o comando `cargo build --release` produzirá o binário `./target/release/trustkit` funcional para os Gates 0, 1, 2, 3 e fiscalização individual de testes do Gate 4!

---

## 6. O Ponto de Virada de Chave (The Cutover Gate: Fase 2 $\rightarrow$ Fase 3)

Antes de iniciar a **Fase 3**, você realiza o ritual de **Cutover (Virada de Chave)**. A transição do harness antigo para o `trustkit` nativo só é autorizada quando os **5 critérios determinísticos abaixo** passarem com `Exit Code: 0`:

```bash
# 1. Compilacao estatica limpa em modo release
cargo clippy --workspace --all-targets -- -D warnings && cargo build --release

# 2. Prova de pureza arquitetural do dominio (apenas std)
cargo tree -p trustkit-core

# 3. Benchmark de cold-start (< 3ms p99)
hyperfine --warmup 10 './target/release/trustkit status --json'

# 4. Instalacao do binario no PATH local do desenvolvedor
ln -sf /home/mmc/00_code/00_tools/TRustKit/target/release/trustkit ~/.local/bin/trustkit

# 5. Auto-inicializacao (Self-Init) no proprio repositorio do TRustKit
trustkit init --json
```

---

## 7. Roteiro Passo a Passo — Estágio 1 (Self-Hosting nas Fases 3 e 4)

Uma vez aprovado no **Cutover Gate**, você desativa a orquestração manual da skill antiga e passa a usar o **próprio `trustkit`** para construir as Fases 3 e 4:

### Passo 1.1: Ativação dos Hooks Nativos do `trustkit` no Próprio Repositório
Configure o hook `PreToolUse` do agente codificador no repositório `/home/mmc/00_code/00_tools/TRustKit` para apontar para:
```bash
trustkit hook pre-tool
```
* **Efeito imediato:** Qualquer tentativa do LLM de rodar `git stash`, `git add .`, commitar fora do padrão ou usar emojis durante a construção das Fases 3 e 4 será bloqueada em **2 milissegundos** pelo próprio binário que está sendo construído.

### Passo 1.2: Construção da Fase 3 (`003-ast-anticheat-slicer`) Governada pelo `trustkit`
1. **Congelamento da Spec da Fase 3 pelo próprio `trustkit`:**
   ```bash
   trustkit spec init --from-prd docs/PRD_TRustKit.md
   trustkit spec lock --json
   ```
   O `trustkit` grava o `ArtifactSeal` (`BLAKE3`) de todos os testes unitários e de integração já criados nas Fases 1 e 2 dentro de `.trustkit/state.db`.
2. **Planejamento das Ondas da Fase 3:**
   ```bash
   trustkit wave plan --json
   ```
   O `PetgraphTopologyAdapter` recém-construído na Fase 2 organiza as tarefas de implementação do `TreeSitterAstAdapter`, `TestIntegrityAuditor`, `InvariantLexicalScanner` e `AstContextSlicer`.
3. **Execução TDD Supervisionada pelo `trustkit`:**
   Para cada tarefa da Fase 3, o Builder é obrigado a registrar a fase RED e a fase GREEN no `.trustkit/state.db`:
   ```bash
   trustkit task test TASK-001 --phase red    # Deve falhar (Exit != 0) e gravar RedVerified
   # Builder implementa o codigo Rust...
   trustkit task test TASK-001 --phase green  # Deve passar (Exit == 0) e gravar GreenVerified
   ```
4. **Auto-Auditoria na Camada 0:**
   Assim que a `TASK` do `TestIntegrityAuditor` (`tree-sitter` para Rust) ficar pronta na Fase 3, você já roda:
   ```bash
   trustkit gauntlet ast-audit --json
   ```
   O `trustkit` analisará a própria árvore sintática (`.rs`) das suas 6 crates em `< 5ms`, verificando que nenhum teste das Fases 1 e 2 foi adulterado!

### Passo 1.3: Construção da Fase 4 (`004-worktree-swarm-mcp`) e Certificação Final
1. Com a Camada 0 (`trustkit-ast`) ativa, inicia-se o ciclo da Fase 4 para implementar o `GitWorktreeReflinkAdapter`, o `SwarmDispatcher` (`tokio`), o `VisualEvidenceGuard` e o servidor `mcp_stdio_server.rs`.
2. Ao concluir a Fase 4, o comando completo é executado sobre o próprio repositório:
   ```bash
   trustkit gauntlet run --json
   trustkit done --json
   ```
3. **Resultado do Dogfooding:** O primeiro relatório `gauntlet-report.md` gerado de ponta a ponta pelo `TRustKit` será o relatório de certificação **dele mesmo**, provando matematicamente (via `.trustkit/state.db`) que todas as 16 *Hard Invariants*, os 7 Gates e os requisitos de performance (`< 3ms` de boot e `< 10ms` de auditoria AST) estão operacionais antes de o binário ser implantado nos projetos de produto da organização.
