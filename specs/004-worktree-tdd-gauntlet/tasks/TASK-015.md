# TASK-015: Integrar Adaptadores no Composition Root em bootstrap.rs

> **Camada:** Infra/Setup  
> **Estimativa:** M  
> **Status:** Concluído  

---

## Contexto
O Composition Root do TRustKit (`bootstrap.rs`) é responsável por instanciar os adaptadores de infraestrutura concretos (`trustkit-db`, `trustkit-gh`, `trustkit-ast`, `trustkit-orch`), injetá-los nos casos de uso de domínio e despachar a execução a partir do ponto de entrada da CLI (`main.rs`).

## Escopo
Implementar em `crates/trustkit-cli/src/bootstrap.rs` e `crates/trustkit-cli/src/main.rs`:
- Expansão de `FullConcurrentBootstrap` para instanciar:
  - `WorktreeIsolationProvider` (adaptador de `WorktreeIsolationPort`).
  - `AsyncExecutionSupervisor` (adaptador de `AsyncExecutionSupervisorPort`).
  - `VisualEvidenceGuard` (adaptador de `VisualEvidencePort`).
  - `PrGateway` e `BoardSynchronizer` expandido (adaptadores de `ProjectBoardGatewayPort`).
  - `SqliteCycleRepository` com novos métodos de persistência.
- Roteamento completo de todos os comandos dos Gates 4, 5 e 6 para seus respectivos handlers.

### Fora do escopo
- Lógica de domínio nos adaptadores ou comandos.

## Dependencias
- TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-014 — Requer todos os adaptadores e handlers.

## Premissas
- Preservar o caminho de inicialização rápida e síncrona (`FastSyncBootstrap`) para subcomandos leves como `hook pre-tool` e `status`.

## Criterio de Aceite
- [x] **Dado** o binário compilado, **quando** qualquer um dos subcomandos novos for executado, **entao** as dependências são resolvidas pelo composition root sem erro de linking ou compilação.
- [x] **Dado** o comando `trustkit hook pre-tool`, **quando** executado, **entao** continua respondendo no caminho síncrono rápido sem carregar o runtime do `tokio`.

## Notas Tecnicas
- Arquivos: `crates/trustkit-cli/src/bootstrap.rs`, `crates/trustkit-cli/src/main.rs`.

## Rastreabilidade
- Origem: RF-005 a RF-012, Secao 5.1 do PRD
