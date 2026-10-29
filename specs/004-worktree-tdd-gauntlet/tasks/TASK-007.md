# TASK-007: Implementar Persistencia SQLite WAL para Fases TDD, Worktrees e Certificacao

> **Camada:** Banco  
> **Estimativa:** M  
> **Status:** Concluída  

---

## Contexto
O lema "O log de eventos prova" exige que toda transição de fase TDD (`RedVerified`, `GreenVerified`), alocação de worktree, contagem de falhas do Gauntlet e veredito de certificação fiquem gravados imutavelmente no banco SQLite em modo WAL (`.trustkit/state.db`), com constraints `NOT NULL` estritas e suporte à concorrência.

## Escopo
Implementar em `crates/trustkit-db/src/sqlite_adapter.rs`:
- Migrações ou ajustes de schema para suportar persistência de:
  - Fases TDD da tarefa: atualizar `tdd_lifecycle_state`, referenciando o id da evidência de falha e de sucesso.
  - Alocação e status de worktree (`worktree_path`, `branch_name`, `merged_sha`).
  - Atualização do contador de tentativas do Gauntlet (`gauntlet_attempt`) e status do ciclo (`CircuitBreakerTripped`, `Certified`, `Merged`).
  - Métodos para consultar histórico completo de evidências para compilação do relatório final.

### Fora do escopo
- Lógica de negócio de transição de Gates (pertence ao domínio em `trustkit-core`).

## Dependencias
- TASK-001 — Requer definições de entidades de domínio.

## Premissas
- Todas as colunas devem conter restrições `NOT NULL`.
- SQLite operando em modo WAL com timeout concorrente configurado para 5000ms.

## Criterio de Aceite
- [x] **Dado** uma tarefa salva, **quando** atualizada para `RedVerified` com id de evidência, **entao** a persistência é gravada e recuperada com fidelidade.
- [x] **Dado** 5 gravações concorrentes de evidências durante a bateria do Gauntlet, **quando** executadas, **entao** nenhuma falha por lock do SQLite ocorre (`busy_timeout`).
- [x] **Dado** a consulta de certificação, **quando** chamada, **entao** lista todas as evidências ordenadas cronologicamente.

## Notas Tecnicas
- Arquivo: `crates/trustkit-db/src/sqlite_adapter.rs`.
- Escrever testes em `crates/trustkit-db/tests/sqlite_tdd_and_cert_tests.rs`.

## Rastreabilidade
- Origem: RF-005, RF-008, RF-010, RN-002
