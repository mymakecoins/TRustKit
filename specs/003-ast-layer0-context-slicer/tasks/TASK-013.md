# TASK-013: Definir Comandos CLI para Auditoria AST e Context Slicing

> **Camada:** API
> **Estimativa:** P
> **Status:** Pendente

depends_on:
- TASK-011
- TASK-012
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/cli_contract.rs
test_file: crates/trustkit-cli/tests/cli_ast_contract_tests.rs

---

## Contexto
O binário `trustkit` precisa expor os subcomandos fortemente tipados com suporte a flags de arquivos e `--json` via `clap`.

## Escopo
- Adicionar comando `trustkit gauntlet ast-audit [--files <LISTA>] [--json]`.
- Adicionar comando `trustkit context slice [--file <PATH>] [--budget <TOKENS>] [--json]`.

### Fora do escopo
- Execução dos handlers de negócio (coberta por TASK-014).

## Dependências
- TASK-011 — Caso de uso de auditoria AST.
- TASK-012 — Caso de uso de context slicing.

## Premissas
- Parsing estrito sem panics em flags incorretas.

## Critério de Aceite
- [ ] **Dado** invocação via linha de comando, **quando** os argumentos forem passados, **então** o `clap` faz o parse para os tipos correspondentes.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-cli/src/cli_contract.rs`.

## Rastreabilidade
- Origem: RF-007, RN-013
