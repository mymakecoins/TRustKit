# TASK-005: Implementar Comando CLI trustkit spec lint

> **Camada:** CLI / Interface
> **Estimativa:** P
> **Status:** Pendente

depends_on: [TASK-004]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/spec_lint_cmd.rs
test_file: crates/trustkit-cli/tests/cli_spec_lint_tests.rs

---

## Contexto
O comando CLI `trustkit spec lint` expoe as capacidades do `SpecLinter` tanto para o desenvolvedor no terminal quanto para agentes autonomos integrados via JSON.

## Escopo
Implementar em `crates/trustkit-cli/src/commands/spec_lint_cmd.rs` e registrar no parser Clap:
- Argumentos CLI: `trustkit spec lint --file <CAMINHO_SPEC> [--json]`.
- Leitura do arquivo informado (ou resolucao da spec do ciclo ativo se omitido).
- Execucao do `SpecLinter` em `trustkit-ast`.
- Saida dual-mode:
  - TTY: Mensagens claras com linha e descricao das violacoes ou confirmacao de conformidade limpa.
  - JSON: Objeto tipado `JsonSpecLintResponse` contendo `verdict`, `total_violations` e array de achados.
- Codigos de saida semanticos: `0` para CleanPass e `2` (SpecOrTraceabilityIncomplete) caso haja violacoes.

### Fora do escopo
- Congelamento criptografico (delegado a `trustkit spec lock`).

## Dependencias
- TASK-004

## Premissas
- Resposta emitida em menos de 3ms.
- Retorno de erro apropriado caso o arquivo nao exista.

## Criterio de Aceite
- [ ] **Dado** uma especificacao valida informada via `--file`, **quando** `trustkit spec lint --json` e executado, **entao** retorna status `PASS`, `verdict: "CLEAN_PASS"`, `total_violations: 0` e codigo de saida `0`.
- [ ] **Dado** uma especificacao com diagrama Mermaid invalido, **quando** executado, **entao** retorna status `FAIL`, lista as violacoes e sai com codigo `2`.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-cli/src/commands/spec_lock_cmd.rs`.

## Rastreabilidade
- Origem: RF-047, RN-005
