# TASK-016: Implementar Suite de Testes E2E para Camada 0 de AST

> **Camada:** Infra/Setup
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-015
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/tests/e2e_gauntlet_ast_audit.rs
test_file: crates/trustkit-cli/tests/e2e_gauntlet_ast_audit.rs

---

## Contexto
O Gauntlet e a integridade de entrega exigem testes end-to-end reais executando o binário estático `trustkit` compilado contra repositórios temporários com asserções, emojis e adulterações de testes.

## Escopo
- Criar suíte E2E em `crates/trustkit-cli/tests/e2e_gauntlet_ast_audit.rs` usando `assert_cmd` e `tempfile`.
- Validar caso positivo (`CleanPass`, `Exit Code: 0`).
- Validar caso negativo anti-cheat (`TestIntegrityViolation`, `Exit Code: 4`).
- Validar caso de emojis proibidos (`RN-007`, `Exit Code: 4`).

### Fora do escopo
- Testes E2E de execução Playwright (Camada 1).

## Dependências
- TASK-015 — Composition Root integrado e binário compilável.

## Premissas
- Testes E2E devem rodar isoladamente sem interferir no repositório de trabalho.

## Critério de Aceite
- [ ] **Dado** teste E2E executando o binário `trustkit`, **quando** submetido a arquivo com teste removido, **então** o processo encerra com código 4 e mensagem informativa.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-cli/tests/e2e_gauntlet_ast_audit.rs`.

## Rastreabilidade
- Origem: RF-007, RN-004, RN-007, RN-013, RN-016
