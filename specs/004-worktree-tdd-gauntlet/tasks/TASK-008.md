# TASK-008: Implementar Provedor de Isolamento de Git Worktrees (WorktreeIsolationProvider)

> **Camada:** Integracao  
> **Estimativa:** G  
> **Status:** Concluido  

---

## Contexto
Para viabilizar a execução paralela de tarefas de uma onda sem disputas na árvore de trabalho (`node_modules`, `.git/index`), o TRustKit precisa criar worktrees efêmeras em `.trustkit/worktrees/task-NNN`, realizar isolamento com reflink ou cópia rápida, e executar o merge determinístico com limpeza posterior.

## Escopo
Implementar em `crates/trustkit-orch/src/worktree_provider.rs`:
- Implementação da porta `WorktreeIsolationPort`.
- Criação física de worktree via comandos Git nativos (`git worktree add -b ...`).
- Otimização de tempo e disco: criação de links simbólicos ou reflinks Copy-on-Write para pastas pesadas de dependência (`node_modules`, `target`).
- Aplicação de merge determinístico com resolução e validação de commits na branch base da issue.
- Remoção e poda física das worktrees ao final da execução (`git worktree remove --force`).

### Fora do escopo
- Decisão de quais tarefas compõem a onda (definido pelo planejador DAG da Fase 2).

## Dependencias
- TASK-002 — Requer a trait de porta `WorktreeIsolationPort`.

## Premissas
- A máquina hospedeira possui Git `>= 2.38` instalado.
- Worktrees efêmeras residem estritamente dentro de `.trustkit/worktrees/`.

## Criterio de Aceite
- [x] **Dado** um repositório Git limpo, **quando** `provision_worktree` é chamado para `TASK-001`, **entao** o diretório `.trustkit/worktrees/TASK-001` é criado e aponta para uma branch isolada.
- [x] **Dado** alterações commitadas na worktree, **quando** `merge_and_prune_worktree` é invocado, **entao** o commit é mergeado na branch base e a pasta da worktree é expurgada.

## Notas Tecnicas
- Arquivo: `crates/trustkit-orch/src/worktree_provider.rs`.
- Cuidado com locking de arquivos no Windows/Linux ao remover worktrees.

## Rastreabilidade
- Origem: RF-006, RN-006
