# TASK-002: Dominio de Milestones e Geracao de Artefatos em Disco

> **Camada:** CORE
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/entities.rs
- crates/trustkit-core/src/ports.rs
- crates/trustkit-core/src/use_cases/plan_milestones.rs
- crates/trustkit-core/src/use_cases/mod.rs
test_file: crates/trustkit-core/tests/milestone_plan_tests.rs

---

## Contexto
O dominio do TRustKit precisa de modelos formais e casos de uso isolados (std only) para representar os marcos de entrega (milestones) derivados da combinacao de PRD e ADR, bem como o algoritmo de geracao dos documentos individuais em `docs/milestones/MNN-<slug>.md` e indice `docs/milestones/INDEX.md`.

## Escopo
- Adicionar ao `trustkit-core/src/entities.rs`: `Milestone`, `MilestonePlan`, `MilestoneCapability`, `SourceCitation`, `MilestoneStatus`.
- Definir os ports em `trustkit-core/src/ports.rs`: `MilestoneRepositoryPort` e `MilestoneExtractorPort`.
- Criar o caso de uso `plan_milestones.rs` em `crates/trustkit-core/src/use_cases/`.
- Estruturar a renderizacao em Markdown de cada marco com objetivo, capacidades, criterios de conclusao verificaveis e dependencias.
- Escrever testes em `crates/trustkit-core/tests/milestone_plan_tests.rs`.
