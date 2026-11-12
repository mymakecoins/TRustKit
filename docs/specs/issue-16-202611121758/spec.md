# Especificacao: redefinir-nome-e-sintaxe-do-comando-de-fechamento-de-ciclo

## 1. Product Trio

- **PM**: O comando `trustkit merge --target <branch>` causava ambiguidade cognitiva ao sugerir integracao Git local em branch alvo, quando sua funcao real e exclusivamente o encerramento formal da governanca do ciclo no SQLite e a transicao da issue no Project Board para ReadyForTest. A solucao define o novo comando `trustkit close` com sintaxe concisa sem a flag `--target`, eliminando termos ambiguos e orientando o desenvolvedor explicitamente a utilizar `trustkit pr --target <branch>` para a criacao de Pull Request e integracao remota de codigo.
- **Tech Lead**: No crate `trustkit-core`, evoluir o caso de uso `merge_cycle` para `close_cycle` (mantendo aliases ou compatibilidade de transicao), descontinuando a dependencia do parametro `target_branch` na transicao de governanca. Em `trustkit-cli`, registrar o comando `trustkit close` (e manter `trustkit merge` temporariamente como alias deprecado com aviso explicativo direcionando para `trustkit close`), eliminando a obrigatoriedade de `--target`. Atualizar `GatePresenter` e `NextStepResolver` para emitir mensagens semanticas claras e payloads JSON estritamente livres de valores nulos (null-free JSON).
- **Design / QA**: Garantir por meio de testes de unidade, integracao de CLI e testes E2E que o comando `trustkit close` exige que o ciclo esteja em estado Certified (Gate 6), fecha o ciclo no banco local, transiciona o card no GitHub Projects para ReadyForTest, e renderiza as recomendacoes deterministicas de proximo passo (`trustkit pr --target develop` ou `trustkit init`) sem nenhum emoji ou violacao constitucional.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[Entrada CLI trustkit close] --> Node2[Validacao de Ciclo e Gate 6 Certified]
    Node2 --> Node3[Transicao de Estado SQLite e Board para ReadyForTest]
    Node3 --> Node4[Renderizacao de Mensagem Semantica e Proximos Passos]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Fechamento de ciclo bem-sucedido com trustkit close (Atendimento de RF-001)

- **Dado** um ciclo SDD ativo no estado Certified aprovado no Gate 6
- **Quando** o desenvolvedor executa o comando `trustkit close`
- **Entao** o ciclo e formalmente encerrado no SQLite, o card e movido para ReadyForTest e a mensagem confirma o fechamento sem alegar merge Git local.

### Cenario 2: Rejeicao de fechamento para ciclo nao certificado (Atendimento de RF-002)

- **Dado** um ciclo SDD ativo que ainda nao passou com sucesso pelo Gate 6
- **Quando** o desenvolvedor tenta executar `trustkit close`
- **Entao** o comando falha deterministicamente exigindo a certificacao previa via `trustkit done`.

### Cenario 3: Compatibilidade e aviso de depreciacao no comando legacy trustkit merge (Atendimento de RF-003)

- **Dado** o comando legado `trustkit merge` executado pelo desenvolvedor
- **Quando** o comando for processado pelo parser da CLI
- **Entao** uma mensagem de aviso de depreciacao e emitida orientando a adocao imediata de `trustkit close`.

### Cenario 4: Output deterministico e resolucao de proximo passo no Presenter (Atendimento de RF-004)

- **Dado** a conclusao bem-sucedida do comando `trustkit close`
- **Quando** a saida for renderizada em modo TTY humano ou `--json`
- **Entao** o payload exibe a transicao para ReadyForTest e recomenda como proximo passo primario `trustkit pr --target develop` sem campos nulos e sem emojis.

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | crates/trustkit-core/src/use_cases/close_cycle_tests.rs | EV-001-close-cycle-nominal.json |
| RN-002 | RF-002 | TASK-002 | Node2 | crates/trustkit-core/src/use_cases/close_cycle_tests.rs | EV-002-close-cycle-uncertified.json |
| RN-003 | RF-003 | TASK-003 | Node3 | crates/trustkit-cli/tests/cli_close_command_tests.rs | EV-003-cli-deprecation-warning.json |
| RN-004 | RF-004 | TASK-004 | Node4 | crates/trustkit-cli/src/presenter_tests.rs | EV-004-close-presenter-null-free.json |
