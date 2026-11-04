# TASK-008: Implementar Subcomando Desacoplado `trustkit audit security` para Homologacao Pre-Release

> **Camada:** Interface CLI & Orquestracao Pre-Release
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-006, TASK-007]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/audit_security_cmd.rs
test_file: crates/trustkit-cli/tests/e2e_security_audit_prerelease.rs

---

## Contexto
Enquanto `code-review` e `security-review` atuam dentro do ciclo de vida de uma issue/task nos Gates 4, 5 e 6, a skill `security-audit` realiza uma auditoria defensiva e pentest exaustivo de todo o repositorio em 6 fases com ledger deterministico (`coverage-ledger.json` e `findings.json`), devendo ser acionada de forma desacoplada em marcos de versao ou antes de merges em branches estaveis (`staging` ou `main`).

## Escopo
Implementar em `crates/trustkit-cli/src/commands/audit_security_cmd.rs`:
- Subcomando CLI `trustkit audit security [--profile <quick|standard|deep>] [--scope <PATH>] [--output-dir <DIR>] [--json]`.
- Validacao do perfil (`quick`, `standard` como padrao, ou `deep`) e verificacao de que o diretorio de saida para os artefatos da auditoria (`run-metadata.json`, `coverage-ledger.json`, `findings.json`, `REPORT.md`) reside fora do repositorio alvo ou esta devidamente ignorado pelo controle de versao.
- Invocacao supervisionada dos validadores `validate-coverage-ledger.cjs` e `validate-findings.cjs` (quando disponiveis) e emissao do sumario consolidado de homologacao pre-release em modo humano ou JSON sem campos nulos.

### Fora do escopo
- Mutacao de worktrees de tarefas ativas do Gate 4.

## Dependencias
- TASK-006
- TASK-007

## Premissas
- Execucao desacoplada: pode rodar mesmo quando nao houver ciclo atomico de issue ativo (`UnlinkedLocalCycle`).
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** a invocacao de `trustkit audit security --profile standard --json`, **quando** executada sobre o repositorio local, **entao** valida a configuracao de isolamento do diretorio de saida e retorna o payload JSON deterministico livre de `null` com `Exit Code: 0`.
- [x] **Dado** um diretorio de saida dentro do repositorio que nao esta no `.gitignore`, **quando** `trustkit audit security` e chamado, **entao** recusa a gravacao para proteger a arvore de trabalho.

## Notas Tecnicas
- Registrar o subcomando `audit` em `crates/trustkit-cli/src/cli_contract.rs` e `crates/trustkit-cli/src/commands/mod.rs`.

## Rastreabilidade
- Origem: RF-040, RN-008
