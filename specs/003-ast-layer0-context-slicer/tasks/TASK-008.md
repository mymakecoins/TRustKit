# TASK-008: Implementar Fatiador de Contexto AST (AstContextSlicer)

> **Camada:** Integração
> **Estimativa:** G
> **Status:** Pendente

depends_on:
- TASK-002
- TASK-003
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/context_slicer.rs
test_file: crates/trustkit-ast/tests/context_slicer_tests.rs

---

## Contexto
Para evitar sobrecarga de tokens e manter o foco cirúrgico dos Builders em cada tarefa de onda (Gate 4), o motor deve fatiar o código-fonte gerando uma visão compactada contendo apenas contratos públicos, assinaturas de métodos e tipos relevantes.

## Escopo
- Implementar `AstContextSlicer` compatível com `ContextSlicerPort`.
- Fatiar arquivos `.rs` preservando `pub trait`, `pub struct`, `pub fn` e interfaces sem os corpos internos das funções.
- Respeitar orçamento de linhas/tokens definido no request.

### Fora do escopo
- Invocação de subprocessos shell.

## Dependências
- TASK-002 — Porta abstrata no domínio.
- TASK-003 — Gramáticas sintáticas Tree-Sitter.

## Premissas
- A representação fatiada deve ser código sintaticamente compreensível pelo agente.

## Critério de Aceite
- [ ] **Dado** um arquivo Rust com 500 linhas de implementação, **quando** fatiado, **então** retorna apenas as assinaturas públicas e estruturas em menos de 80 linhas.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-ast/src/context_slicer.rs`.

## Rastreabilidade
- Origem: RF-006, RF-007, RN-008
