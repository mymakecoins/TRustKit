# Analise Comparativa: TRustKit vs Skills de Code Review e Seguranca

Data da Analise: 2026-10-30  
Status: Aprovado para Pesquisa e Integracao  
Escopo: Avaliacao de equivalencia funcional, sobreposicao de gates e modelo de integracao entre o TRustKit e as skills externas:
- `/home/mmc/00_code/00_tools/_skills/security-audit`
- `/home/mmc/00_code/00_tools/_skills/claude-code-skills/code-review`
- `/home/mmc/00_code/00_tools/_skills/claude-code-skills/security-review`

---

## 1. Resumo Executivo

O **TRustKit nao cumpre o mesmo papel dessas tres skills**.

O TRustKit e um **motor deterministico em Rust para governanca de Spec-Driven Development (SDD)**: sua atuacao consiste em validar mecanicamente sintaxe AST, integridade de testes (politica anti-cheat), invariantes sintaticos/lexicos de repositorio, o ciclo formal TDD (fase RED comprovada antes da fase GREEN), isolamento fisico em Git Worktrees (com suporte a Copy-on-Write) e execucao do Gauntlet Loop V2 (linters estaticos, build de release e suites de teste Playwright/E2E).

Em contrapartida, as tres skills analisadas sao ferramentas especializadas de **inspecao semantica via Modelos de Linguagem (LLM), code review de diffs e auditoria defensiva de seguranca de aplicacao (AppSec / Pentest)**.

Ha uma intersecao pontual no **Gate 5 (Gauntlet)**, onde o TRustKit executa ferramentas de analise estatica de compilador/linter e checagem lexico-sintatica. Contudo, o TRustKit **nao analisa a semantica de negocio do diff, nao realiza analise de fluxo de dados (taint analysis) para vulnerabilidades OWASP, e nao executa pentest ou modelagem de ameacas**. Portanto, as skills nao sao concorrentes nem redundantes: sao **altamente complementares** ao ecossistema do TRustKit.

---

## 2. Diagrama de Posicionamento Arquitetural

```
+---------------------------------------------------------------------------------------------------+
| TRustKit: Governanca do Ciclo de Vida SDD (Gates 0 a 6, Worktrees, TDD, AST e Evidencias)        |
+---------------------------------------------------------------------------------------------------+
       |                                          |                                    |
       v (Gate 4 / Gate 5)                        v (Gate 5 / Gate 6 - Opcional)       v (Auditoria Global)
[code-review]                              [security-review]                    [security-audit]
Revisao semantica de diff,                 Deteccao de vulnerabilidades         Pentest exaustivo, modelo
corretude logica, simplificacao,           AppSec no diff (>80% certeza:        de ameacas, 10 classes de
altitude e invariantes de codigo.          SQLi, RCE, Auth bypass, XSS).        ataque e ledgers de cobertura.
```

---

## 3. Analise Detalhada por Skill

### 3.1. Skill `code-review`
Caminho: `/home/mmc/00_code/00_tools/_skills/claude-code-skills/code-review/SKILL.md`

#### Papel e Escopo da Skill
- Realiza revisao automatizada de diffs (working tree, branch, PR ou commit) focada em corretude logica e qualidade de codigo.
- Executa inspecao multi-angulo (8 a 10 angulos independentes):
  - **Varredura Linha a Linha (Corretude e Logica):** Busca condicoes invertidas, erros off-by-one, dereferencias nulas (`null`/`undefined`), `await` ausente, mascaramento de erros em blocos `catch` vazios e expressoes regulares mal formatadas.
  - **Auditoria de Comportamento Removido:** Rastreia cada linha deletada no diff e valida se o invariante, guard ou tratamento de erro foi reestabelecido no novo codigo.
  - **Rastreamento Cruzado (Call Sites e Contratos):** Mapeia callers e callees de cada funcao alterada para detectar quebra de compatibilidade em outros modulos.
  - **Reuso, Simplificacao, Eficiencia e Altitude:** Identifica duplicacao de funcoes ja existentes no projeto, complexidade desnecessaria, memory leaks por closures de longa duracao e verifica se a solucao trata a causa-raiz (altitude adequada) ou e apenas um paliativo fragil.
  - **Conformidade com Diretrizes:** Identifica violacoes diretas as regras documentadas no repositorio (`AGENTS.md`, diretrizes de arquitetura e guias de estilo).
- Emprega protocolo de verificacao e deduplicacao para minimizar falsos positivos, com flags para aplicacao de correcoes (`--fix`) e publicacao de comentarios inline em PRs (`--comment`).

#### O que o TRustKit Cumpre
- **Gate 5 (Camada 0 - AST Audit):**
  - O `TestIntegrityAuditor` monitora o numero de assercoes de testes antes e depois do ciclo via AST Tree-Sitter (`rust_assertion_counter.rs` e `ts_assertion_counter.rs`), barrando reducoes maliciosas ou diluicoes de testes.
  - O `InvariantLexicalScanner` bloqueia o uso de `@ts-ignore` e tipos genericos `any` em TypeScript, alem de proibir emojis e trailers de IA em commits e codigos.
- **Topologia Estrutural (`trustkit graph`):**
  - Extracao de AST e deteccao de God Nodes e acoplamento topologico entre modulos via Tree-Sitter (`crates/trustkit-ast/src/ast_graph_extractor.rs`).
- **Invariantes Mecanicos:**
  - O `trustkit hook pre-tool` bloqueia execucoes destrutivas de Git (`git stash`, `git reset --hard`, `git add .`, commits em branches protegidas).

#### O que o TRustKit NAO Cumpre (Gaps)
- Nao realiza analise semantica de diff por inferencia LLM. Nao identifica condicoes invertidas, off-by-one em regras de negocio ou erros engolidos em `catch`.
- Nao avalia altitude arquitetural (se uma correcao e paliativa ou estrutural).
- Nao gera sugestoes de refatoracao de codigo em linguagem natural nem comentarios inline no GitHub/GitLab.

---

### 3.2. Skill `security-review`
Caminho: `/home/mmc/00_code/00_tools/_skills/claude-code-skills/security-review/SKILL.md`

#### Papel e Escopo da Skill
- Focada exclusivamente em identificar **vulnerabilidades de seguranca de alto impacto introduzidas pelo diff** de uma branch ou PR.
- Cobertura de categorias AppSec criticas:
  - **Validacao de Entrada e Injecoes:** SQL Injection, Command Injection, XXE, Template Injection (SSTI), NoSQL Injection e Path Traversal.
  - **Autenticacao e Autorizacao:** Bypasses de autenticacao, escalacao de privilegios, vulnerabilidades em tokens JWT e falhas de controle de sessao (BOLA/IDOR).
  - **Criptografia e Segredos:** Chaves de API ou senhas hardcoded, geradores de numeros pseudoaleatorios fracos e bypass de validacao de certificados TLS.
  - **Execucao Remota de Codigo (RCE):** Desserializacao insegura (Python pickle, YAML inseguro), injecao em `eval` e falhas de Cross-Site Scripting (XSS refletido, armazenado e DOM-based).
  - **Exposicao de Dados:** Vazamento de PII e credenciais em logs ou endpoints de API.
- Filosofia de baixissimo falso positivo: so relata achados com mais de 80% de certeza (nota >= 8/10) de explorabilidade real.

#### O que o TRustKit Cumpre
- **Gate 5 (Camada 1 - Swarm de Criticos):**
  - O critico `StaticSecurityCritic` (`run_gauntlet_layer1.rs`) executa linters estaticos nativos da linguagem:
    - Rust: `cargo clippy -- -D warnings`.
    - TypeScript/Node: `npm run lint && npm run build`.
- **Invariantes Lexicos:**
  - O `InvariantLexicalScanner` bloqueia URLs externas suspeitas (como `raw.githubusercontent.com`) e previne tipos fracos.
- **Protecao do Ambiente Local:**
  - Hooks nativos impedem acoes indesejadas de agentes locais.

#### O que o TRustKit NAO Cumpre (Gaps)
- Nao realiza analise de vulnerabilidades de aplicacao (AppSec / OWASP Top 10) no codigo desenvolvido.
- Ferramentas como Clippy ou ESLint detectam boas praticas e consistencia de tipos, mas nao realizam analise de fluxo de dados (taint analysis) para detectar se inputs de usuario desinfetados chegam a comandos de sistema, consultas de banco ou tags HTML.
- Nao faz analise de bypass logico de controle de acesso (BOLA/IDOR) ou quebra de modelos de autenticacao.

---

### 3.3. Skill `security-audit`
Caminho: `/home/mmc/00_code/00_tools/_skills/security-audit/SKILL.md`

#### Papel e Escopo da Skill
- Framework exaustivo de **auditoria defensiva de seguranca e pentest abrangente de repositorios inteiros**.
- Metodologia formal em 6 fases com ledger deterministico de cobertura (`coverage-ledger.json`) e catalogo de achados (`findings.json`, `report-schema.json`).
- Cobre 10 classes estruturadas de ataque:
  1. AI e LLM (prompt injection, vazamento de contexto, abuso de tools)
  2. Attack Classes gerais
  3. Client-Side e Protocolos Web (XSS, CSRF, CORS, OAuth, Session Management)
  4. Cloud e Deploy (politicas IAM, conteineres, CI/CD, segredos)
  5. Isolamento de Dados e Ciclo de Vida
  6. Desktop/Mobile e IPC local
  7. Memory Safety e Binarios
  8. Protocolos, RPC e Mensageria
  9. Supply Chain e Release
  10. Esgotamento de Recursos e Disponibilidade
- Execucao sob sandbox rigoroso: isolamento absoluto de rede externa, orcamento de agentes (budget gate), descritores de arquivo protegidos contra race condition (`fstat`, `no-follow`) para promocao de artefatos de `scratch/` para `artifacts/`.

#### O que o TRustKit Cumpre
- **Rigor de Isolamento e Rastreabilidade Criptografica:**
  - O TRustKit compartilha o mesmo nivel de obsessao por determinismo: isola tarefas em Git Worktrees efemeras com reflink, calcula hashes BLAKE3 de specs e arquivos, e registra cada evidencia de execucao de forma imutavel no banco SQLite WAL (`.trustkit/state.db`).
- **Relatorio de Certificacao:**
  - Emissao automatizada do `gauntlet-report.md` no Gate 6 (`certify_cycle_and_report.rs`).

#### O que o TRustKit NAO Cumpre (Gaps)
- Nao e uma ferramenta de auditoria de seguranca ofensiva/defensiva nem de pentest de aplicacoes.
- Nao modela vetores de ataque nem coordena agentes especializados em descobrir e demonstrar exploracao de falhas de seguranca.
- Seu escopo e a garantia de conformidade de engenharia de software (SDD), e nao a certificacao de ameacas de seguranca da aplicacao.

---

## 4. Matriz Comparativa de Capacidades

| Capacidade / Requisito | TRustKit | `code-review` | `security-review` | `security-audit` |
|---|:---:|:---:|:---:|:---:|
| **Governanca do Ciclo SDD (Gates 0 a 6)** | Nativo (Rust) | Nao possui | Nao possui | Nao possui |
| **Isolamento de Git Worktrees e TDD Estrito** | Nativo (Rust) | Nao possui | Nao possui | Nao possui |
| **Auditoria Anti-Cheat de Testes e Emojis** | Nativo (AST Camada 0) | Parcial (Removed-behavior) | Nao possui | Nao possui |
| **Analise Estatica via Linter/Compilador** | Nativo (Camada 1) | Nao possui | Nao possui | Nao possui |
| **Inteligencia de Grafo Estrutural AST** | Nativo (`trustkit graph`) | Parcial (Tracer) | Nao possui | Nao possui |
| **Revisao Semantica de Bugs de Logica no Diff** | Nao possui | Especialista (8-10 angulos) | Nao possui | Nao possui |
| **Inspecao de Simplificacao, Reuso e Altitude** | Nao possui | Especialista | Nao possui | Nao possui |
| **Deteccao de Falhas AppSec no Diff (>80% certeza)** | Nao possui | Nao possui | Especialista (OWASP) | Nao possui |
| **Pentest e Auditoria Global de Repositorio** | Nao possui | Nao possui | Nao possui | Especialista (6 fases) |
| **Matriz de Ameacas em 10 Classes de Ataque** | Nao possui | Nao possui | Nao possui | Especialista |

---

## 5. Recomendacao de Integracao no Pipeline TRustKit

Para alcancar o maximo de robustez e excelencia em ambientes agenticos de alta densidade, as tres skills devem ser acopladas aos pontos estrategicos do fluxo do TRustKit:

1. **Gate 4 (Fase Refactor do TDD):**
   - Apos a conclusao da fase GREEN de uma tarefa, disparar a skill `code-review` antes de `trustkit task complete <TASK_ID>`. Isso assegura que o codigo produzido pelo agente construtor seja legivel, livre de bugs sutis e aderente as convencoes locais.

2. **Gate 5 / Gate 6 (Inspecao Opcional de AppSec no Diff):**
   - Integrar a skill `security-review` como uma verificacao **opcional** acionada na transicao entre a aprovacao mecanica do Gauntlet (`trustkit gauntlet run`) e a certificacao final (`trustkit done`), conforme detalhado na Secao 6.

3. **Auditorias Periodicas e Pre-Release:**
   - Executar a skill `security-audit` de forma independente dos ciclos atomicos de issues, como rotina de homologacao antes de merges em `staging` ou `main`.

---

## 6. Especificacao da Interacao para Execucao Opcional da `security-review`

Como a `security-review` envolve inferencia semantica especializada de AppSec sobre o diff consolidado, sua execucao e definida como **opcional**, ficando sob controle direto do desenvolvedor tanto via parametros de linha de comando quanto via prompt interativo.

### 6.1. Ponto de Acionamento nos Gates

O acoplamento ocorre na transicao entre o **Gate 5 (Gauntlet)** e o **Gate 6 (Certificacao)**, suportado diretamente nos comandos `trustkit gauntlet run` e `trustkit done`:
- Primeiro, o TRustKit executa 100% das validacoes mecanicas deterministicas (Camada 0 AST + Camada 1 Linters/Clippy/Playwright).
- Somente apos a aprovacao unanime das validacoes mecanicas, o motor avalia a decisao de executar ou pular a `security-review` antes de consolidar o `gauntlet-report.md`.

### 6.2. Modelo de Interacao no Terminal (Modo Humano Interativo)

No modo interativo, o desenvolvedor pode escolher entre **antecipar a decisao via parametro de linha de comando** ou **responder ao prompt interativo** ao final da bateria mecanica:

#### A. Decisao Antecipada via Parametro CLI (Sem Prompt Bloqueante)
O usuario humano pode passar explicitamente sua escolha ao invocar `trustkit gauntlet run` ou `trustkit done`:

```bash
# Forca a execucao da revisao de seguranca sem perguntar no terminal
trustkit gauntlet run --security-review
trustkit done --security-review

# Dispensa explicitamente a revisao de seguranca sem exibir o prompt
trustkit gauntlet run --skip-security-review
trustkit done --skip-security-review
```

#### B. Decisao via Prompt Interativo (Quando Nenhuma Flag e Informada)
Se o desenvolvedor executar `trustkit gauntlet run` ou `trustkit done` em um terminal interativo (TTY) sem passar `--security-review` nem `--skip-security-review`, o TRustKit exibe o prompt de decisao:

```text
[trustkit] Gate 5: Camada 0 (AST Integrity) -> CLEAN PASS (1 ms)
[trustkit] Gate 5: Camada 1 (Critic Swarm)  -> UNANIMOUS PASS (4/4 criticos aprovados)
[trustkit] -------------------------------------------------------------
[trustkit] Deseja executar a revisao de seguranca especializada (security-review)
[trustkit] no diff desta entrega antes da certificacao final? [s/N]: 
```

1. **Resposta `n`, `N` ou `<Enter>` (Padrao: Nao):**
   - O TRustKit avanca imediatamente para a certificacao.
   - No `gauntlet-report.md`, registra a rastreabilidade da decisao:
     ```markdown
     - **Specialized Security Review:** SKIPPED (Opt-out pelo desenvolvedor)
     ```

2. **Resposta `s` ou `S` (Sim):**
   - O TRustKit aciona a analise `security-review` sobre o diff da branch atual contra a branch base (`develop`).
   - **Cenario Limpo (Zero vulnerabilidades com confianca >= 8):**
     ```text
     [trustkit] security-review concluida: Nenhuma vulnerabilidade detectada (Confianca >= 8).
     [trustkit] Certificacao aprovada com sucesso!
     ```
   - **Cenario com Alerta (Vulnerabilidade de Alta Confianca detectada):**
     ```text
     [trustkit] ALERTA DE SEGURANCA: 1 vulnerabilidade de alta confianca detectada!
     [trustkit] -> SQL Injection em crates/trustkit-db/src/custom_query.rs:42 (Confianca: 9/10)
     [trustkit] Cenario: Parametro de busca concatenado sem prepared statement.
     
     Deseja abortar o Gate 6 para correcao imediata? [S/n]:
     ```

### 6.3. Suporte a Automacao e Agentes (Modo Headless / `--json`)

Quando o TRustKit e invocado por agentes autonomos ou pipelines de CI/CD (presenca da flag `--json` ou ausencia de TTY interativo), o motor jamais bloqueia aguardando entrada de teclado, obedecendo aos mesmos parametros:

- `trustkit gauntlet run --security-review --json` ou `trustkit done --security-review --json`: Executa obrigatoriamente a revisao de seguranca e inclui os achados estruturados no payload JSON e no `gauntlet-report.md`.
- `trustkit gauntlet run --skip-security-review --json` ou `trustkit done --skip-security-review --json`: Dispensa a revisao de seguranca.
- **Comportamento padrao sem flag em modo `--json` / nao-interativo:** Segue a politica definida na configuracao do projeto (padrao: `skip`, preservando a execucao deterministica de baixa latencia).

### 6.4. Configuracao de Politica por Projeto (`trustkit.toml` / `.trustkit/config.json`)

O time pode parametrizar o comportamento padrao do repositorio quando nenhuma flag de linha de comando (`--security-review` ou `--skip-security-review`) for informada:

```toml
[governance.gauntlet]
# Opcoes:
# - "prompt": pergunta interativamente no terminal humano; pula no modo --json (padrao)
# - "always": executa sempre, a menos que --skip-security-review seja passado
# - "disabled": nunca executa, a menos que --security-review seja passado
security_review_mode = "prompt"
min_confidence_threshold = 8
```
