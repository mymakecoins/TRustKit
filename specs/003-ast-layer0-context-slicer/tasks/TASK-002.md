# TASK-002: Definir Portas Abstratas de Auditoria AST e Context Slicer

> **Camada:** Domínio
> **Estimativa:** P
> **Status:** Concluída

depends_on:
- TASK-001
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/tests/ports_tests.rs

---

## Contexto
O domínio precisa acionar analisadores sintáticos e fatiadores de contexto sem acoplamento direto com Tree-Sitter ou com o sistema de arquivos, viabilizando testes unitários FIRST em memória.

## Escopo
Declarar as traits de porta:
- `AstAuditorPort`: métodos para auditoria léxica, anti-cheat de testes e conformidade de invariantes.
- `ContextSlicerPort`: métodos para fatiar arquivos-fonte e produzir resumos de assinaturas estruturais.

### Fora do escopo
- Implementação concreta dos adaptadores (coberta por TASK-007 e TASK-008).

## Dependências
- TASK-001 — Requer tipos e entidades de domínio modelados.

## Premissas
- Utilizar apenas a biblioteca padrão (`std`).

## Critério de Aceite
- [x] **Dado** uma implementação mock da trait, **quando** executada, **então** compila e valida o fluxo de entrada e saída.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-core/src/ports.rs`.

## Rastreabilidade
- Origem: RF-007, RN-002
