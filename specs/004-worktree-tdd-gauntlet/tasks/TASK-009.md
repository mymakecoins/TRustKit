# TASK-009: Implementar Despachante Assincrono Concorrente e Capturador de Evidencias

> **Camada:** Integracao  
> **Estimativa:** M  
> **Status:** Concluido  

---

## Contexto
O Gauntlet e a execução de ondas exigem o disparo concorrente de comandos de teste, linters e builders. Cada execução deve ser cronometrada com precisão, ter seu código de saída (`Exit Code`) capturado e a saída de texto sanitizada contra vazamento de segredos (`SecretRedactor`) antes de ser persistida no banco SQLite como `ExecutionEvidence`.

## Escopo
Implementar em `crates/trustkit-orch/src/async_supervisor.rs`:
- Implementação da porta `AsyncExecutionSupervisorPort` sobre o runtime `tokio`.
- Execução de comandos de subprocesso assíncronos (`tokio::process::Command`).
- Medição precisa de duração em milissegundos (`std::time::Instant`).
- Sanitização de credenciais e tokens em `stdout` e `stderr` (ocultando chaves `ghp_...`, senhas, tokens bearer).
- Empacotamento do resultado em `ExecutionEvidence`.

### Fora do escopo
- Lógica de aprovação ou reprovação do Gauntlet (pertence ao domínio).

## Dependencias
- TASK-002 — Requer a trait `AsyncExecutionSupervisorPort`.

## Premissas
- Nenhum token de autenticação ou chave privada pode vazar na saída gravada no banco ou nos relatórios.

## Criterio de Aceite
- [x] **Dado** um comando shell executado pelo supervisor, **quando** finaliza, **entao** retorna com exatidão o `exit_code`, tempo em milissegundos e resumo sanitizado da saída.
- [x] **Dado** uma saída contendo um token sensível (`ghp_xyz123`), **quando** processada, **entao** o token é substituído por `[REDACTED_SECRET]`.

## Notas Tecnicas
- Arquivo: `crates/trustkit-orch/src/async_supervisor.rs`.
- Suporte a cancelamento gracioso via timeout.

## Rastreabilidade
- Origem: RF-006, RF-008, RN-002
