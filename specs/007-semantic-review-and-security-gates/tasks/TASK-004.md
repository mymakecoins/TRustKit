# TASK-004: Adicionar Flags CLI `--security-review` e `--skip-security-review` em `gauntlet run` e `done`

> **Camada:** Interface CLI (Contrato Clap)
> **Estimativa:** P
> **Status:** Concluido

depends_on: [TASK-002]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/cli_gate_args.rs
test_file: crates/trustkit-cli/tests/cli_security_review_flags_tests.rs

---

## Contexto
Conforme especificado na Secao 6.2 e 6.3 do documento de pesquisa, tanto no Modo Humano Interativo quanto no Modo Agente (`--json`), o usuario deve poder passar sua escolha de execucao da `security-review` diretamente como parametro nos comandos `trustkit gauntlet run` e `trustkit done`, evitando o prompt interativo quando desejar decidir de antemao.

## Escopo
Implementar em `crates/trustkit-cli/src/cli_gate_args.rs`:
- Adicionar nos argumentos de `gauntlet run` e `done`:
  - `--security-review`: Forca a execucao da revisao de seguranca especializada no diff.
  - `--skip-security-review`: Dispensa explicitamente a revisao de seguranca sem exibir prompt.
- Configurar `conflicts_with` no Clap entre `--security-review` e `--skip-security-review` para garantir exclusao mutua deterministica.
- Adicionar helper `resolve_cli_security_flag(&self) -> CliSecurityReviewFlag` convertendo o par de booleans em `CliSecurityReviewFlag::ForceExecute`, `CliSecurityReviewFlag::ForceSkip` ou `CliSecurityReviewFlag::Unspecified`.
- Adicionar tambem a opcao `--code-review <EFFORT>` em `trustkit task complete` e o subcomando `trustkit task review <TASK_ID> [--effort <low|medium|high|max>]`.

### Fora do escopo
- Logica de execucao do prompt interativo ou invocacao da revisao (tratados nas TASK-005 e TASK-006).

## Dependencias
- TASK-002

## Premissas
- Passar `--security-review` e `--skip-security-review` simultaneamente deve ser rejeitado pelo parser CLI com erro claro de argumento conflitante.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** o comando `trustkit gauntlet run --security-review` ou `trustkit done --security-review`, **quando** parseado pelo Clap, **entao** produz `CliSecurityReviewFlag::ForceExecute`.
- [x] **Dado** o comando `trustkit done --skip-security-review`, **quando** parseado pelo Clap, **entao** produz `CliSecurityReviewFlag::ForceSkip`.
- [x] **Dado** o comando `trustkit gauntlet run --security-review --skip-security-review`, **quando** parseado pelo Clap, **entao** rejeita a combinacao por conflito de flags.

## Notas Tecnicas
- Manter compatibilidade retroativa integral quando nenhuma das duas flags for informada (`CliSecurityReviewFlag::Unspecified`).

## Rastreabilidade
- Origem: RF-036, RN-004
