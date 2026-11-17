# Especificacao: notificar-explicitamente-criacao-de-branch-e-checkout-no-provision-worktree

## 1. Product Trio

- **PM**: Ao provisionar uma worktree para tarefas do SDD, desenvolvedores e agentes necessitam de visibilidade transparente e inequivoca sobre o estado do repositorio Git. O sistema deve emitir mensagens claras confirmando a criacao da branch especifica da tarefa, a realizacao do checkout restrito ao diretorio isolado da worktree (.trustkit/worktrees/<task_id>) e a confirmacao de que a branch do repositorio raiz permaneceu intocada e preservada.
- **Tech Lead**: Em `crates/trustkit-orch`, o `WorktreeIsolationProvider` deve reportar detalhes de alocacao contendo branch da tarefa, caminho absoluto ou relativo da worktree e branch base preservada. No crate `trustkit-db` e `trustkit-core`, as evidencias de execucao (`ExecutionEvidence`) devem armazenar esses dados estruturados de provisionamento. Em `crates/trustkit-cli`, atualizar `wave_run_cmd.rs`, `auto_cmd.rs` e `gate_presenter.rs` para renderizar as mensagens informativas padronizadas em ASCII puro no modo TTY e disponibilizar os campos no JSON (`--json`) em estrita conformidade com a politica null-free.
- **Design / QA**: Garantir que as mensagens no terminal sigam o formato sem emojis `[INFO] Branch 'worktree/<task_id>' criada com sucesso.`, `[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/<task_id>` e `[INFO] Branch ativa do repositorio raiz preservada: '<base_branch>'`. Validar via suites de teste unitario e integracao em todos os crates impactados com 100% de cobertura e respeito aos invariantes constitucionais.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[WorktreeIsolationProvider Provisionamento] --> Node2[ExecutionEvidence Gravacao de Evidencias]
    Node2 --> Node3[GatePresenter Formatacao TTY ASCII]
    Node3 --> Node4[JsonWaveRunResponse Payload Estruturado]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Detalhes estruturados no retorno de provisionamento de worktree (Atendimento de RF-001)

- **Dado** o WorktreeIsolationProvider executando o provisionamento para uma tarefa
- **Quando** o comando git worktree add for concluido com sucesso
- **Entao** a alocacao retornada contem a branch criada, o diretorio isolado da worktree e a branch base preservada.

### Cenario 2: Persistencia das informacoes de branch e checkout em ExecutionEvidence (Atendimento de RF-002)

- **Dado** uma tarefa com worktree recem-provisionada
- **Quando** a evidencia de execucao for gravada no repositorio SQLite
- **Entao** o registro armazena os metadados explicitando a branch da tarefa criada e o checkout na worktree.

### Cenario 3: Mensagens informativas em ASCII puro no modo TTY (Atendimento de RF-003)

- **Dado** a execucao de onda ou ciclo autonomo em terminal interativo
- **Quando** o provisionamento de worktree for apresentado
- **Entao** o terminal exibe as mensagens informativas em ASCII puro notificando a criacao da branch, o checkout no diretorio isolado e a preservacao da branch raiz.

### Cenario 4: Payload estruturado sem campos nulos no modo JSON (Atendimento de RF-004)

- **Dado** a execucao de onda com a flag --json
- **Quando** o payload de resposta for serializado
- **Entao** o JSON contem os dados estruturados de provisionamento sem nenhum valor nulo.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | crates/trustkit-orch/tests/worktree_provision_notification_tests.rs | EV-001-worktree-provision-notification.json |
| RN-002 | RF-002 | TASK-002 | Node2 | crates/trustkit-db/tests/sqlite_worktree_evidence_tests.rs | EV-002-sqlite-worktree-evidence.json |
| RN-003 | RF-003 | TASK-003 | Node3 | crates/trustkit-cli/tests/cli_worktree_notification_tty_tests.rs | EV-003-cli-worktree-notification-tty.json |
| RN-004 | RF-004 | TASK-004 | Node4 | crates/trustkit-cli/tests/cli_worktree_notification_json_tests.rs | EV-004-cli-worktree-notification-json.json |
