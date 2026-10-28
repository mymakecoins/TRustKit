# TASK-012: Implementar Caso de Uso de Fatiamento de Contexto AST

> **Camada:** Domínio
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-002
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/slice_ast_context.rs
test_file: crates/trustkit-core/tests/slice_ast_use_case_tests.rs

---

## Contexto
O caso de uso de fatiamento de contexto é invocado pelo orquestrador no Gate 4 para alimentar cada Builder com o contexto estritamente necessário para sua tarefa, sem poluir a janela de contexto.

## Escopo
- Criar `SliceAstContextUseCase` recebendo caminhos de arquivos e símbolos-chave.
- Delegar a extração sintática ao `ContextSlicerPort`.
- Retornar o pacote estruturado de assinaturas fatiadas.

### Fora do escopo
- Dispatch concorrente em worktrees (coberto na Fase 4).

## Dependências
- TASK-002 — Porta abstrata `ContextSlicerPort`.

## Premissas
- Nenhum valor nulo nas estruturas de retorno.

## Critério de Aceite
- [ ] **Dado** uma lista de arquivos de dependência de uma tarefa, **quando** o caso de uso executa, **então** entrega o resumo fatiado das assinaturas.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-core/src/use_cases/slice_ast_context.rs`.

## Rastreabilidade
- Origem: RF-006, RF-007, RN-012
