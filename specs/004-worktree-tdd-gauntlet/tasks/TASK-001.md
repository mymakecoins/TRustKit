# TASK-001: Implementar Entidades e Estados de Dominio para Worktrees, TDD e Gauntlet

> **Camada:** Dominio  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O fluxo deterministico do Gauntlet V2 e a execucao fisica das tarefas em worktrees efemeras necessitam de estruturas de dados e enums algebricos formais para representar o ciclo TDD (fases RED e GREEN), alocacao e ciclo de vida de worktrees, vereditos dos 4 criticos do Gauntlet e o disjuntor de seguranca (Circuit Breaker de 5 tentativas). Essa modelagem deve respeitar o Principio de Casos Especiais com proibicao estrita de valores `null`.

## Escopo
Modelar em `crates/trustkit-core/src/entities.rs` e `crates/trustkit-core/src/special_cases.rs`:
- Estruturas de alocacao de worktree: `WorktreeAllocation` (`NotProvisioned`, `Provisioned { worktree_path, branch_name }`, `MergedAndPruned { commit_sha }`).
- Estados do ciclo TDD: `TddPhase` (`Red`, `Green`), `TddExecutionProof { phase, exit_code, duration_ms, evidence_id }`.
- Estruturas dos Criticos do Gauntlet: `CriticKind` (`DomainAdversarial`, `ContractCompliance`, `StaticSecurity`, `PlaywrightVisual`), `CriticVerdict` (`Pass`, `Fail { reason, file, line }`).
- Estruturas de Certificacao e Circuit Breaker: `CertificationSummary`, `CircuitBreakerState` (`Closed`, `Tripped { consecutive_failures, diagnostic }`).

### Fora do escopo
- Acesso a banco de dados SQLite ou chamadas ao sistema de arquivos (cobertos por TASK-007 e TASK-008).
- Implementacao de logica de execucao de subprocessos (coberta por TASK-009).

## Dependencias
- Nenhuma

## Premissas
- Seguir estritamente o padrao Rust `std-only` na crate `trustkit-core`.
- Nenhum campo opcional contera `null` / `None` silencioso onde um Caso Especial semantico for aplicavel.

## Criterio de Aceite
- [X] **Dado** uma entidade `AtomicTask`, **quando** o builder avanca pelo ciclo TDD, **entao** o estado e representado com seguranca tipada sem possibilidade de representar `GreenVerified` sem uma prova de falha previa (`RedVerified`).
- [X] **Dado** uma alocacao de worktree, **quando** instanciada em memoria, **entao** transiciona deterministamente entre `NotProvisioned` -> `Provisioned` -> `MergedAndPruned`.
- [X] **Dado** o contador de tentativas do Gauntlet, **quando** atinge 5 falhas consecutivas, **entao** o estado do ciclo reflete `CycleStatus::CircuitBreakerTripped`.

## Notas Tecnicas
- Arquivos: `crates/trustkit-core/src/entities.rs`, `crates/trustkit-core/src/special_cases.rs`.
- Validar conformidade com as 16 Invariantes do TRustKit (especialmente RN-002 e RN-005).

## Rastreabilidade
- Origem: RF-005, RF-006, RF-008, RN-002, RN-005
