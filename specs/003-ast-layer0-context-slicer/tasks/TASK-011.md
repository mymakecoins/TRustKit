# TASK-011: Implementar Caso de Uso de Auditoria AST da Camada 0

> **Camada:** Domínio
> **Estimativa:** M
> **Status:** Pendente

depends_on:
- TASK-002
- TASK-010
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/audit_ast_layer0.rs
test_file: crates/trustkit-core/tests/audit_ast_use_case_tests.rs

---

## Contexto
O caso de uso de auditoria de Camada 0 é acionado antes de qualquer execução de LLM no Gauntlet (Gate 5). Ele consolida a verificação das 16 invariantes e a integridade matemática dos testes em `< 5ms`.

## Escopo
- Criar `AuditAstLayer0UseCase` recebendo o diff dos arquivos e referências dos selos.
- Orquestrar a execução do scanner de invariantes e do auditor de integridade.
- Persistir o veredito via `CycleEvidenceRepositoryPort`.

### Fora do escopo
- Chamadas de CLI ou formatação de strings de console (cobertas por TASK-014).

## Dependências
- TASK-002 — Portas abstratas de auditoria.
- TASK-010 — Repositório SQLite para gravação de evidências.

## Premissas
- Tempo de resposta determinístico inferior a 5 milissegundos.

## Critério de Aceite
- [ ] **Dado** arquivos íntegros, **quando** o caso de uso executa, **então** retorna `CleanPass` e persiste o evento com sucesso.
- [ ] **Dado** uma quebra de invariante, **quando** o caso de uso executa, **então** rejeita imediatamente e bloqueia o avanço.

## Notas Técnicas
- Arquivo alvo: `crates/trustkit-core/src/use_cases/audit_ast_layer0.rs`.

## Rastreabilidade
- Origem: RF-007, RN-003, RN-011
