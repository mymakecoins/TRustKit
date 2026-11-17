# TASK-001: Entidades de Profundidade de Modulos e Costuras no Dominio Core

> **Camada:** CORE
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/graph_entities.rs
- crates/trustkit-core/src/module_depth_entities.rs
test_file: crates/trustkit-core/src/graph_entities_tests.rs

---

## Contexto
Para incorporar a disciplina de Modulos Profundos (John Ousterhout) e Costuras (Michael Feathers) no motor do TRustKit, o dominio `trustkit-core` precisa modelar as metricas de profundidade de modulo, candidatos a refatoracao pelo Deletion Test e classificacoes de costuras reais vs hipoteticas sem valores nulos e usando apenas `std`.

## Escopo
- Definir `ModuleDepthClassification` (`Deep`, `Balanced`, `ShallowPassThrough`), `ModuleDepthMetric`, `ShallowModuleCandidate`, `SeamKind` (`RealSeam`, `HypotheticalSeam`) e `SeamClassification`.
- Integrar os campos `module_depth_metrics`, `shallow_modules` e `seam_classifications` em `GraphHealthReport` e em sua serializacao JSON livre de `null`.
- Escrever testes unitarios em `crates/trustkit-core/src/graph_entities_tests.rs`.
