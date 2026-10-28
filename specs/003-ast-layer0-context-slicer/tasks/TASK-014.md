# TASK-014: Implementar Handlers CLI para Execucao de AST e Slicing

> **Camada:** API
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-013
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/gauntlet_ast_audit_cmd.rs
test_file: crates/trustkit-cli/tests/commands_ast_tests.rs

---

## Contexto
Os handlers da CLI conectam a entrada dos argumentos aos use cases correspondentes e formatam a saída em modo legível e JSON sem campos nulos.

## Escopo
- Criar `gauntlet_ast_audit_cmd.rs` com renderização de vereditos `PASS` / `FAIL` e serialização JSON.
- Mapear código de saída POSIX: `0` para `CleanPass` e `4` para rejeição anti-cheat / invariantes.

### Fora do escopo
- Injeção de dependências no Composition Root (coberta por TASK-015).

## Dependências
- TASK-013 — Contrato de comandos do `clap`.

## Premissas
- Toda saída em modo `--json` deve obedecer ao Princípio II (zero null).

## Critério de Aceite
- [ ] **Dado** uma auditoria com sucesso em modo `--json`, **quando** o handler executa, **então** emite JSON válido com status `PASS` e `Exit Code: 0`.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-cli/src/commands/gauntlet_ast_audit_cmd.rs`.

## Rastreabilidade
- Origem: RF-007, RN-007, RN-014
