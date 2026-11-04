# TASK-006: Implementar Casos de Uso de Certificacao, Merge e Pull Request

> **Camada:** Dominio  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O Gate 6 coroa o ciclo de desenvolvimento gerando a certificação determinística a partir dos eventos imutáveis registrados no SQLite. Em seguida, orienta a decisão explícita de merge (Invariante #11) e disponibiliza a abertura de Pull Request exclusivamente sob demanda do usuário (Invariante #12).

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/`:
- `certify_cycle_and_report.rs`:
  - Verificar se existem registros `PASS` para Camada 0, 4 Críticos, Lint, Build e E2E.
  - Compilar `gauntlet-report.md` via `CycleCertificationPort` a partir do histórico do banco.
  - Formatar o comentário oficial da issue com Matriz de Rastreabilidade total, nota de negócios, sumário técnico e imagens HTML autenticadas.
  - Atualizar o estado do ciclo para `CycleStatus::Certified`.
- `merge_cycle.rs`:
  - Exigir estado `Certified`.
  - Executar o merge na branch de destino indicada pelo usuário (`develop`, `staging`).
  - Acionar a transição de card no GitHub Projects para `Await Staging`.
- `open_pull_request.rs`:
  - Validar autorização humana explícita.
  - Montar o corpo do PR contendo `Ref #<id>` (proibindo palavras-chave de fechamento automático), revisores e matriz de rastreabilidade.
  - Disparar a criação via `ProjectBoardGatewayPort` e transicionar o card para `In review`.

### Fora do escopo
- Chamadas diretas à API do GitHub (delegadas ao adaptador `trustkit-gh`).

## Dependencias
- TASK-002 — Requer portas de certificação e gateway de projetos.

## Premissas
- Nenhum PR pode ser aberto automaticamente pelo sistema sem invocação explícita de `trustkit pr`.
- É estritamente proibido o uso de `Closes`, `Fixes` ou `Resolves` no Pull Request (Invariante #12).

## Criterio de Aceite
- [x] **Dado** um ciclo com Gauntlet aprovado, **quando** `certify_cycle_and_report` executa, **entao** o status avança para `Certified` e os artefatos de relatório são gerados sem `null`.
- [x] **Dado** uma solicitação de merge sem o ciclo estar certificado, **quando** executada, **entao** o comando rejeita com erro de portão não cumprido.
- [x] **Dado** uma solicitação de abertura de PR, **quando** executada, **entao** o payload do PR contém revisores e assignees configurados e referência `Ref #<id>`.

## Notas Tecnicas
- Arquivos: `crates/trustkit-core/src/use_cases/certify_cycle_and_report.rs`, `crates/trustkit-core/src/use_cases/merge_cycle.rs`, `crates/trustkit-core/src/use_cases/open_pull_request.rs`.

## Rastreabilidade
- Origem: RF-010, RF-011, RF-012, RN-010, RN-011, RN-012
