# TASK-003: Implementar Comando CLI trustkit spec prompt

> **Camada:** CLI / Interface
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-002]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/spec_prompt_cmd.rs
- crates/trustkit-cli/src/cli_contract.rs
test_file: crates/trustkit-cli/tests/cli_spec_prompt_tests.rs

---

## Contexto
O desenvolvedor ou agente orquestrador necessita de um comando CLI deterministico que prepare o prompt estruturado e o esqueleto de especificacao para qualquer nova ideia de feature, com injecao automatica de contexto e respeito a invariantes.

## Escopo
Implementar em `crates/trustkit-cli/src/commands/spec_prompt_cmd.rs` e registrar no contrato Clap (`crates/trustkit-cli/src/cli_contract.rs`):
- Argumentos CLI: `trustkit spec prompt --idea <IDEIA> [--context-hops <HOPS>] [--budget <TOKENS>] [--json]`.
- Handler que executa o compositor de contexto e formata o payload.
- Renderizador dual-mode no `GatePresenter`:
  - Modo Humano (TTY): Apresenta o prompt pronto para copia com instrucoes constitutionais.
  - Modo Agente (`--json`): Retorna JSON estrito sem campos `null`.

### Fora do escopo
- Chamada direta a APIs de Modelos de Linguagem no CLI.

## Dependencias
- TASK-002

## Premissas
- Resposta emitida em menos de 10ms.
- Retorno de erro semantico com codigo de saida adequado caso a ideia seja vazia.

## Criterio de Aceite
- [ ] **Dado** uma ideia via `--idea "autenticacao biometrica"` em modo `--json`, **quando** executado, **entao** retorna JSON contendo `scaffold_markdown`, `context_summary` e `token_estimate` com status `SUCCESS`.
- [ ] **Dado** uma invocacao sem parametro `--idea`, **quando** validado pelo Clap, **entao** rejeita a chamada com mensagem de ajuda clara.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-cli/src/commands/spec_init_cmd.rs`.

## Rastreabilidade
- Origem: RF-045, RN-003
