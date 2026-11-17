# TASK-002: Sintese Deterministica de Arquivos tasks/TASK-NNN.md sem Sobrescrita

layer: ORCH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-orch/tests/task_synthesis_fs_tests.rs
expected_files:
  - crates/trustkit-orch/src/task_synthesizer.rs
  - crates/trustkit-orch/tests/task_synthesis_fs_tests.rs

## Contexto
Materializar fisicamente no disco os arquivos `tasks/TASK-NNN.md` a partir dos rascunhos extraidos pelo atomizador, garantindo formato compativel com `task_parser.rs` e preservando arquivos pre-existentes customizados pelo desenvolvedor sem sobrescrita destrutiva.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma lista de rascunhos de tarefas atomicas e um diretorio de ciclo sem pasta `tasks/` pre-existente ou com tarefas parciais
- **Quando** o `TaskSynthesizer::materialize_tasks_dir` for executado
- **Entao** deve gravar os novos arquivos `TASK-NNN.md` no disco e preservar integralmente os que ja existirem.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-task-file-synthesis.json
