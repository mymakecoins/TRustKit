# TASK-004: Implementar Scanner Lexico de Invariantes e Proibicoes

> **Camada:** Integração
> **Estimativa:** M
> **Status:** Concluída

depends_on:
- TASK-001
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/lexical_scanner.rs
test_file: crates/trustkit-ast/tests/lexical_scanner_tests.rs

---

## Contexto
O harness TRustKit exige bloqueio absoluto de caracteres dos blocos Unicode de Emojis/Emoticons/Dingbats (RN-007) e de trailers de autoria de IA (RN-013) em todo arquivo novo ou modificado.

## Escopo
- Implementar `InvariantLexicalScanner` que varre bytes/strings em busca de:
  - Faixas Unicode proibidas (Emoji blocks).
  - Padrões de trailers `Co-Authored-By: Claude`, `Claude-Session` ou links `claude.ai/code`.
  - Uso não autorizado de tipos `any` soltos em código tipado.

### Fora do escopo
- Contagem estrutural de asserções de testes (coberta por TASK-005 e TASK-006).

## Dependências
- TASK-001 — Requer tipos de violações do domínio.

## Premissas
- Varredura direta em memória com latência inferior a 1 milissegundo.

## Critério de Aceite
- [x] **Dado** um código com emoji Unicode, **quando** escaneado, **então** acusa violação RN-007 com a linha correspondente.
- [x] **Dado** um texto limpo, **quando** escaneado, **então** retorna aprovação sem apontamentos.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/lexical_scanner.rs`.

## Rastreabilidade
- Origem: RF-007, RN-004, RN-007, RN-013
