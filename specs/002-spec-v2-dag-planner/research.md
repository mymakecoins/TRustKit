# Technical Research: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

## 1. Upstream PRD/ADR Ingestion Engine (`PrdSpecificationIngestor`)

### Decision
Implementar um parser deterministico e robusto para documentos Markdown em `trustkit-ast` (ou adaptador especializado) que extrai o primeiro titulo H1 para geracao de slug, varre tabelas de Regras de Negocio (`RN`) e Requisitos Funcionais (`RF`) atraves de correspondencia semantica de colunas Markdown, e constroi o modelo de documento `SpecificationV2`.

### Rationale
- Documentos de produto (PRD) e arquitetura (ADR) produzidos por times de produto ou analistas seguem convencoes Markdown padronizadas de cabecalhos (`# [Titulo]`) e tabelas com colunas `ID`, `Descricao`, `Criterio`, etc.
- O parser deve ser estrito na validacao: caso o arquivo nao exista ou nao possua tabelas com colunas identificadoras de `RN` e `RF`, aborta com erro tipado (`SpecIngestionError::MissingBusinessRules`) sem gerar arquivos parciais corrompidos em disco.
- Tempo de processamento para documentos de ate 1000 linhas e inferior a 5ms em Rust nativo, cumprindo a meta de SC-001 (< 100ms).

### Alternatives Considered
- *Expressao Regular simples*: Rejeitado por ser fragil contra variacoes legitimas de espacamento e quebras de linha em tabelas Markdown.
- *LLM / Chamada de IA*: Rejeitado categoricamente; o harness e deterministico e nativo, sem latencia de rede ou custo de tokens no Gate 0a.

---

## 2. Mermaid & Gherkin Validation & Cryptographic Sealing (`Gate 1 & Gate 2`)

### Decision
1. **Validacao Mermaid**: Inspecionar os blocos com cerca de linguagem `mermaid` verificando o tipo suportado (`flowchart LR` / `flowchart TD`), conectividade dos nos e presenca de rotulos sem tags HTML invalidas.
2. **Validacao Gherkin**: Inspecionar blocos de cenarios assegurando a presenca de todas as clausulas `Dado` / `Quando` / `Entao` (ou `Given` / `When` / `Then`), recusando cenarios incompletos ou sem clausula condicional.
3. **Fechamento da Matriz de Rastreabilidade**: Verificar que cada `RN` possui mapeamento bidirecional completo para `RF`, `TASK`, `E2E` e `Screenshot`. Qualquer omissao produz `TraceabilityGap { orphan_rule }`.
4. **Selo Criptografico**: Computar digest `BLAKE3` do conteudo exato do arquivo de especificacao, salvar o registro na tabela `artifact_seals` com categoria `'SPEC'` e atualizar o campo `spec_lock_state` na tabela `sdd_cycles` para `'FROZEN'`.

### Rationale
- `BLAKE3` e a funcao de hash criptografico recomendada pela Constituicao do TRustKit devido ao seu paralelismo com SIMD e arvore de Merkle, garantindo calculo em menos de 1ms.
- A persistencia no SQLite em modo WAL garante atomicidade: o estado do ciclo so muda para `FROZEN` se a insercao do selo for bem-sucedida.

### Alternatives Considered
- *SHA-256*: Rejeitado em favor do `BLAKE3`, que e mais rapido e ja e a dependencia padronizada no workspace `trustkit-db`.
- *Permitir gaps na Matriz de Rastreabilidade*: Rejeitado; o objetivo do SDD e garantir que 100% das regras de negocio cheguem com teste e evidencia ao final do ciclo.

---

## 3. DAG Construction & Wave Scheduling (`petgraph`)

### Decision
Adicionar a dependencia `petgraph = "0.7"` no crate `trustkit-ast` e implementar o `WaveExecutionPlanner`:
1. Mapear cada tarefa atomica (`TASK-NNN.md`) para um vertice de `petgraph::graph::DiGraph<TaskNode, DependencyKind>`.
2. Criar arestas direcionadas para cada item declarado no frontmatter/secao `depends_on`.
3. Executar deteccao de ciclos com `petgraph::algo::toposort` ou `petgraph::algo::kosaraju_scc`. Em caso de ciclo, extrair a rota ciclica exata e emitir `DagValidationError::CycleDetected`.
4. Analisar a secao `Arquivos Esperados` de cada tarefa. Caso duas tarefas concorrentes (sem aresta direta de precedencia entre si) declarem o mesmo arquivo, criar uma aresta de isolamento direcionada da tarefa de menor indice numerico (`TASK-001`) para a de maior indice (`TASK-002`), forçando-as a pertencerem a ondas consecutivas distintas.
5. Calcular o indice da onda de cada tarefa como a distancia maxima no DAG a partir de nos raiz (tarefas sem dependencias), agrupando tarefas por `wave_index` em uma estrutura ordenada `WaveSchedule`.

### Rationale
- `petgraph` e a biblioteca de referencia no ecossistema Rust para algoritmos de grafos, comprovadamente confiavel, rapida e sem dependencias pesadas.
- O cálculo de ondas por distância em DAG acíclico executa em complexidade $O(V + E)$, consumindo menos de 2ms para centenas de tarefas atomicas.
- O isolamento deterministico por menor indice numerico cumpre rigorosamente a decisao validada no `/sdd-clarify`.

### Alternatives Considered
- *Implementar DAG ad-hoc sem petgraph*: Rejeitado; reinventar algoritmos de ordenacao topologica e deteccao de ciclos em grafos direcionados aumentaria a superficie de bugs. A Constituicao do TRustKit explicitamente lista `petgraph` como biblioteca autorizada para `trustkit-ast`.
- *Permitir execucao paralela em arquivos compartilhados com git merge posterior*: Rejeitado categoricamente; conflitos de merge durante ondas paralelas quebram o fluxo de automacao dos agentes de IA.

---

## 4. Port Contracts & Hexagonal Purity (`trustkit-core`)

### Decision
Manter `trustkit-core` com zero dependencias externas. As novas capacidades serao expostas ao dominio atraves de novas portas abstratas em `trustkit_core::ports`:
- `SpecValidatorPort`: operacoes de validacao de sintaxe e fechamento de matriz.
- `PrdIngestorPort`: operacao de parse e extracao de regras upstream.
- `WavePlannerPort`: operacao de construcao do DAG e agendamento de ondas.
- `CycleEvidenceRepositoryPort`: expandir para suportar consulta/atualizacao de travas de spec e registro de tarefas planejadas.

Os adaptadores concretos serao implementados em:
- `trustkit-ast`: implementa `SpecValidatorPort`, `PrdIngestorPort` e `WavePlannerPort`.
- `trustkit-db`: implementa a persistencia dos selos e atualizacao do typestate.
- `trustkit-cli`: registra os comandos `trustkit spec init --from-prd`, `trustkit spec lock` e `trustkit wave plan`, conectando as portas no Composition Root (`bootstrap.rs`).
