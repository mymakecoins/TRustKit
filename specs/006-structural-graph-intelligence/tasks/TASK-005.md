# TASK-005: Implementar Analisador de Acoplamento AST e God Nodes no WaveExecutionPlanner

> **Camada:** Analise Estatica AST & Planejamento DAG
> **Estimativa:** M
> **Status:** Concluido

depends_on: [TASK-002]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/wave_coupling_analyzer.rs
- crates/trustkit-ast/src/wave_planner.rs
test_file: crates/trustkit-ast/tests/wave_coupling_analyzer_tests.rs

---

## Contexto
Atualmente o `WaveExecutionPlanner` no Gate 3 serializa tarefas apenas quando elas compartilham exatamente o mesmo caminho em `expected_files` ou declaram `depends_on` manualmente. Duas tarefas que modificam arquivos diferentes, mas onde o arquivo de uma importa diretamente um tipo/funcao modificado pela outra (ou quando ambas alteram vizinhos diretos de um `GodNode`), podem gerar quebras de compilacao no merge das Git Worktrees.

## Escopo
Implementar `WaveCouplingAnalyzer` em `crates/trustkit-ast/src/wave_coupling_analyzer.rs` e integrar a `wave_planner.rs`:
- Dado um conjunto de `TaskNode` e a lista de `StructuralEdge` do repositorio, detectar se algum `expected_file` da tarefa `T_i` possui aresta direta de acoplamento (`Imports` ou `ReferencesSymbol` com confianca `Extracted`) para um `expected_file` da tarefa `T_j`.
- Identificar `GodNodes` (arquivos com grau de entrada acima do limiar configuravel) e impedir que duas tarefas que alterem simultaneamente contratos consumidos pelo mesmo `GodNode` rodem na mesma onda sem serializacao.
- Inserir aresta direcionada no DAG (`petgraph`) preservando a aciclicidade (menor `TASK-NNN` precede o maior quando nao houver caminho previo).

### Fora do escopo
- Alteracoes no provisionador de Git Worktrees ou no banco SQLite.

## Dependencias
- TASK-002

## Premissas
- Preservar todos os testes existentes de `wave_planner.rs`.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** `TASK-001` modificando `crates/trustkit-core/src/ports.rs` e `TASK-002` modificando `crates/trustkit-ast/src/ast_adapter.rs` (sem `depends_on` declarado), **quando** existe uma aresta `Extracted` de `ast_adapter.rs` importando `ports.rs`, **entao** o `WaveExecutionPlanner` aloca `TASK-001` na Onda 1 e `TASK-002` na Onda 2.
- [x] **Dado** duas tarefas com arquivos completamente independentes no grafo AST, **quando** `plan_waves` e executado, **entao** ambas permanecem alocadas paralelamente na Onda 1.

## Notas Tecnicas
- Manter a resolucao deterministica por ordem lexicografica de `task_id` para evitar introduzir ciclos no `DiGraph`.

## Rastreabilidade
- Origem: RF-029, RN-005
