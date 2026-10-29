# TASK-002: Definir Portas Abstratas para Worktree, Orquestracao, Evidencias Visuais e Certificacao

> **Camada:** Dominio  
> **Estimativa:** P  
> **Status:** Concluída  

---

## Contexto
Para manter o isolamento hexagonal estrito (`Constitutional Principle I`), os servicos de aplicacao de dominio precisam orquestrar worktrees, despachar comandos assincronos, validar integridade visual de screenshots e submeter Pull Requests sem depender diretamente de Git, tokio, Playwright ou da CLI do GitHub.

## Escopo
Definir em `crates/trustkit-core/src/ports.rs` as seguintes traits de porta de saida:
- `WorktreeIsolationPort`: metodos para provisionar worktree efemera (`provision_worktree`), checar status e aplicar merge deterministico com exclusao (`merge_and_prune_worktree`).
- `AsyncExecutionSupervisorPort`: despacho assincrono de processos/testes capturando duracao, `exit_code` e saida sanitizada.
- `VisualEvidencePort`: inspecao e restauracao automatica de screenshots alheias via Git (`restore_foreign_screenshots`).
- `CycleCertificationPort`: geracao deterministica do relatorio Markdown (`gauntlet-report.md`) e formatacao do comentario oficial na issue.
- Extensao de `ProjectBoardGatewayPort`: metodo `create_pull_request` para abertura sob demanda com revisores e assignee.

### Fora do escopo
- Implementacao concreta com Git ou tokio (feita nos adaptadores das tarefas de integracao).

## Dependencias
- TASK-001 — Requer as entidades e estruturas de estado de dominio.

## Premissas
- Todas as traits devem ser `Send + Sync`.
- Retornos utilizam `Result<T, DomainViolation>`.

## Criterio de Aceite
- [x] **Dado** o arquivo `ports.rs`, **quando** inspecionado, **entao** todas as 5 novas operacoes de infraestrutura estao representadas por traits puras em `std`.
- [x] **Dado** testes unitarios em `trustkit-core`, **quando** mockadas essas portas, **entao** o dominio consegue compilar e ser testado sem bibliotecas de terceiros.

## Notas Tecnicas
- Arquivo: `crates/trustkit-core/src/ports.rs`.
- Evitar acoplamento com tipos de `rusqlite` ou `tokio`.

## Rastreabilidade
- Origem: RF-005, RF-006, RF-008, RF-010, RN-002
