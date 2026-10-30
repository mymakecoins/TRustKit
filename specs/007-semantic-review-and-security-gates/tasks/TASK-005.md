# TASK-005: Implementar Prompt Interativo TTY (`[s/N]`) e Fallback Headless nos Handlers de Gate 5 e 6

> **Camada:** Interface CLI / Apresentacao
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-004, TASK-006, TASK-007]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/gauntlet_run_cmd.rs
- crates/trustkit-cli/src/commands/done_cmd.rs
test_file: crates/trustkit-cli/tests/interactive_security_prompt_tests.rs

---

## Contexto
Quando o Gauntlet mecanico aprova a entrega, a politica e `"prompt"` e o desenvolvedor nao passou `--security-review` nem `--skip-security-review`, o TRustKit deve perguntar interativamente no terminal humano (`[s/N]`, padrao `Nao` no `<Enter>`) se deseja rodar a `security-review` antes da certificacao final, e usar fallback automatico nao-bloqueante para `Skip` quando executado em modo `--json` ou sem TTY.

## Escopo
Implementar nos handlers `gauntlet_run_cmd.rs` e `done_cmd.rs`:
- Integracao com `resolve_security_review_decision` (TASK-002) e `InteractivePromptPort`.
- Quando a decisao for `SecurityReviewDecision::PromptUser`:
  - Exibir o banner ASCII (zero emojis):
    ```text
    [trustkit] Deseja executar a revisao de seguranca especializada (security-review)
    [trustkit] no diff desta entrega antes da certificacao final? [s/N]:
    ```
  - Ler a entrada do usuario: `s` ou `S` dispara `run_security_review::execute`; `n`, `N` ou linha vazia (`<Enter>`) registra `SecurityReviewVerdict::SkippedByDeveloper { reason: SecuritySkipReason::DeveloperPromptOptOut }`.
- Quando a `security-review` detectar vulnerabilidades (`>= 8`) em modo interativo:
  - Exibir o alerta detalhado de cada vulnerabilidade e perguntar:
    `Deseja abortar o Gate 6 para correcao imediata? [S/n]:` (padrao `Sim` no `<Enter>`, abortando com `Exit Code: 5`).

### Fora do escopo
- Logica interna de filtragem de confianca (implementada na TASK-006).

## Dependencias
- TASK-004
- TASK-006
- TASK-007

## Premissas
- Em modo `--json` ou quando `stdin` nao for um terminal TTY, jamais bloquear aguardando leitura de `stdin`.
- Saida 100% ASCII livre de emojis.

## Criterio de Aceite
- [x] **Dado** execucao interativa sem flags e entrada `\n` (`<Enter>`) no prompt `[s/N]`, **quando** `done_cmd` processa a resposta, **entao** avanca sem executar a revisao e grava `SkippedByDeveloper` com motivo `DeveloperPromptOptOut`.
- [x] **Dado** execucao interativa sem flags, resposta `s` no prompt e 1 vulnerabilidade SQL Injection detectada, **quando** o usuario pressiona `<Enter>` no prompt `Deseja abortar o Gate 6 para correcao imediata? [S/n]:`, **entao** aborta a certificacao com `Exit Code: 5`.

## Notas Tecnicas
- Injetar um adaptador de `InteractivePromptPort` testavel (permitindo mockar entradas `s`, `n`, `\n` nos testes de integracao sem travar o runner de testes).

## Rastreabilidade
- Origem: RF-037, RN-005
