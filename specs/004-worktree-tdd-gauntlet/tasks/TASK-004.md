# TASK-004: Implementar Caso de Uso de Execucao de Onda e Isolamento de Worktree

> **Camada:** Dominio  
> **Estimativa:** M  
> **Status:** Concluida  

---

## Contexto
O Gate 4 do TRustKit executa as tarefas atômicas organizadas em ondas paralelas (`WaveSchedule`). Para evitar colisão no diretório de trabalho e corrupção do índice Git, cada tarefa de uma onda deve ser despachada para uma worktree física isolada, executada e posteriormente incorporada via merge determinístico na branch de trabalho da issue.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/run_execution_wave.rs`:
- Validar se a Spec está congelada (`SpecLockState::Frozen`). Se não estiver, rejeitar.
- Obter a lista de tarefas da onda informada (`wave_number`).
- Para cada tarefa da onda:
  - Invocar `WorktreeIsolationPort::provision_worktree`.
  - Despachar o runner de tarefas atômicas.
  - Validar padrão de mensagem de commit (`<type>: #<id> <descricao em pt-BR>`).
  - Executar merge na branch de trabalho da issue via `WorktreeIsolationPort::merge_and_prune_worktree`.
- Registrar as evidências de cada tarefa no repositório.

### Fora do escopo
- Chamadas diretas de comando Git (delegadas à porta).
- Criação de branches protegidas.

## Dependencias
- TASK-002 — Requer portas de worktree e persistência.

## Premissas
- Apenas tarefas sem colisão de arquivos residem na mesma onda (garantido pelo planejador DAG da Fase 2).
- Falha em qualquer tarefa da onda interrompe o avanço e sinaliza o erro determinístico.

## Criterio de Aceite
- [x] **Dado** uma Spec em `DraftUnverified`, **quando** `run_execution_wave` é executado, **entao** recusa a execução exigindo congelamento prévio da especificação.
- [x] **Dado** uma onda válida com 2 tarefas, **quando** executada com sucesso, **entao** provisiona 2 worktrees isoladas e realiza o merge sequencial de cada uma na branch base da issue.

## Notas Tecnicas
- Arquivo: `crates/trustkit-core/src/use_cases/run_execution_wave.rs`.
- Seguir o limite de 250 LOC por módulo com decomposição coesa.

## Rastreabilidade
- Origem: RF-006, RN-006, RN-007, RN-013
