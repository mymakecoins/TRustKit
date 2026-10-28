# TASK-001: Implementar Entidades de Dominio da Camada 0 de AST e Slicing

> **Camada:** Domínio
> **Estimativa:** M
> **Status:** Concluída

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/entities.rs
test_file: crates/trustkit-core/src/entities_tests.rs

---

## Contexto
A Camada 0 de auditoria (Gate 5) e o mecanismo de fatiamento de contexto para os Builders (Gate 4) necessitam de representações puras no domínio sem dependência de bibliotecas externas, respeitando o Princípio I (Isolamento Hexagonal) e o Princípio II (Proibição de NULL).

## Escopo
Modelar os enums algébricos e structs:
- `AstAuditVerdict` com variantes `CleanPass { violations: [] }` e `Rejected { violations: Vec<AstViolation> }`.
- `TestIntegrityViolation` com arquivo, linha e contagem de asserções removidas.
- `ContextSliceRequest` e `ContextSliceResult` contendo símbolos e assinaturas extraídas.

### Fora do escopo
- Lógica de parsing léxico ou gramatical (coberta por TASK-004 e TASK-005).

## Dependências
- Nenhuma

## Premissas
- Todas as estruturas devem rejeitar `null` e utilizar o Padrão de Caso Especial para ausências.

## Critério de Aceite
- [x] **Dado** uma auditoria sem inconsistências, **quando** instanciada, **então** o resultado é `AstAuditVerdict::CleanPass` com lista vazia.
- [x] **Dado** uma violação de integridade, **quando** instanciada, **então** contém arquivo, linha e tipo da falha sem valores nulos.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-core/src/entities.rs`.
- Seguir estritamente o limite de 250 LOC por módulo.

## Rastreabilidade
- Origem: RF-007, RN-001, RN-004
