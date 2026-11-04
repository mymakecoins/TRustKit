# Feature Specification: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

**Feature Branch**: `002-spec-v2-dag-planner`

**Created**: 2026-10-27

**Status**: Draft

**Input**: User description: "Implementar a Fase 2 do PRD_TRustKit.md: Motor de Spec V2, Ingestão Upstream (--from-prd), Congelamento Criptográfico da Spec (Gate 1 e Gate 2) e Planejamento de Ondas via DAG com petgraph (Gate 3)"

## Clarifications

### Session 2026-10-27
- Q: Qual critério determinístico de ordenação deve ser utilizado pelo planejador de ondas para desempatar e encadear duas tarefas que compartilham o mesmo arquivo quando não há dependência declarada entre elas? → A: Ordenação pela numeração sequencial da tarefa (`TASK-NNN` com menor índice precede e roda na onda anterior).
- Q: Como o nome do arquivo de especificação gerado deve ser derivado na ingestão upstream (`--from-prd`) caso o usuário não informe explicitamente um nome de saída? → A: Derivado a partir do primeiro título H1 do PRD convertido em slug kebab-case (`<titulo-slug>-spec.md`), permitindo sobrescrita via flag opcional `--name <slug>`.
- Q: Como os comandos `trustkit spec lock` e `trustkit wave plan` devem localizar os arquivos de especificação e de tarefas caso nenhum caminho seja informado explicitamente na linha de comando? → A: Consultar a funcionalidade/issue ativa no banco SQLite local (`.trustkit/state.db`) para inferir os caminhos padrão, suportando sobrescrita via flags opcionais (`--spec` e `--tasks-dir`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ingestao Upstream de Especificacao a Partir de PRD/ADR (Priority: P1)

Como um engenheiro de software ou agente de desenvolvimento, desejo inicializar automaticamente uma especificacao de funcionalidade (Spec V2) a partir de um documento existente de requisitos (PRD ou ADR), para que as regras de negocio, requisitos funcionais e entidades sejam extraidos sem digitacao manual redundante ou erros de transcricao.

**Why this priority**: A ingestao upstream e a porta de entrada para o ciclo de desenvolvimento orientado a especificacoes (SDD). Sem ela, os desenvolvedores perdem tempo transcrevendo tabelas e correm risco de omitir requisitos essenciais definidos pela equipe de produto ou arquitetura.

**Independent Test**: Pode ser testado de forma isolada fornecendo um documento PRD/ADR em Markdown contendo tabelas de Regras de Negocio e Requisitos Funcionais e executando a ingestao; o sistema deve produzir um arquivo de especificacao completo com as seções estruturadas e as regras devidamente catalogadas.

**Acceptance Scenarios**:

1. **Given** um documento de requisitos em Markdown contendo secoes com tabelas de Regras de Negocio (`RN`) e Requisitos Funcionais (`RF`), **When** o comando de inicializacao com parametro upstream (`--from-prd <caminho>`) e executado, **Then** o sistema gera um arquivo de especificacao V2 contendo o Product Trio, cenarios Gherkin e o esqueleto da Matriz de Rastreabilidade pre-populados com todas as regras e requisitos extraidos.
2. **Given** que o caminho indicado para o documento de requisitos nao existe no sistema de arquivos, **When** o comando com `--from-prd` e invocado, **Then** a execucao e imediatamente interrompida com mensagem clara de erro e nenhum arquivo parcial ou corrompido e gerado em disco.
3. **Given** um documento PRD valido existente, mas que nao contem nenhuma definicao de tabela de Regras de Negocio (`RN`) ou Requisitos Funcionais (`RF`), **When** o comando com `--from-prd` e invocado, **Then** a operacao e rejeitada indicando a ausencia de regras formais para extracao e orientando a correcao do documento fonte.

---

### User Story 2 - Validacao Estrutural e Congelamento Criptografico da Spec (Priority: P2)

Como um auditor de qualidade, Tech Lead ou agente fiscalizador, desejo submeter a especificacao a uma validacao formal de completude (fluxograma Mermaid, sintaxe Gherkin e fechamento 100% da Matriz de Rastreabilidade) e registrar seu selo criptografico de imutabilidade, para assegurar que nenhum desenvolvimento inicie sobre uma especificacao incompleta, incoerente ou sujeita a alteracoes posteriores nao auditadas.

**Why this priority**: O congelamento criptografico (Gate 1 e Gate 2) e a espinha dorsal de confianca do harness. Ele garante que qualquer desvio, perda de rastreabilidade ou adulteracao de regras de negocio durante a construcao seja instantaneamente detectado e bloqueado antes de afetar a producao.

**Independent Test**: Pode ser testado fornecendo especificacoes com diferentes estados de completude: uma especificacao valida e selada com sucesso gerando seu hash e bloqueando mutacoes; enquanto uma especificacao com lacunas de rastreabilidade ou sintaxe invalida e rejeitada com diagnostico detalhado do erro.

**Acceptance Scenarios**:

1. **Given** um arquivo de especificacao contendo fluxograma Mermaid valido, cenarios em formato Gherkin consistentes e uma Matriz de Rastreabilidade fechada (onde toda RN mapeia para um RF, tarefas e testes), **When** o comando de travamento da especificacao (`spec lock`) e acionado, **Then** o sistema calcula a assinatura criptografica imutavel do arquivo, armazena o selo no banco de estado e transiciona o status da especificacao para congelado (`FROZEN`).
2. **Given** uma especificacao em que uma Regra de Negocio declarada nao possui tarefa ou teste correspondente mapeado na Matriz de Rastreabilidade (regra orfa), **When** o comando de travamento e acionado, **Then** a operacao de congelamento e rejeitada, o status permanece nao verificado (`DRAFT_UNVERIFIED`) e o sistema emite relatorio apontando explicitamente o identificador da regra sem cobertura.
3. **Given** uma especificacao cujo bloco Mermaid possui erro de sintaxe ou cujos cenarios de aceitacao nao atendem a estrutura formal Gherkin (Dado/Quando/Entao), **When** o travamento e solicitado, **Then** o sistema recusa o congelamento e indica o bloco e a linha onde ocorreu a inconsistencia sintatica.
4. **Given** uma especificacao previamente congelada que sofreu alteracao em disco sem autorizacao, **When** qualquer etapa subsequente de governanca verifica a integridade do ciclo, **Then** a divergencia da assinatura criptografica e imediatamente acusada e o fluxo construtivo e bloqueado.

---

### User Story 3 - Planejamento Topologico de Ondas de Execucao via Grafo (Priority: P3)

Como um desenvolvedor ou agente orquestrador de tarefas, desejo analisar a lista de tarefas atomicas da funcionalidade para construir um Grafo Aciclico Direcionado (DAG) de dependencias e gerar um cronograma de ondas de execucao, para viabilizar a execucao paralela segura das tarefas sem riscos de condicoes de corrida ou conflitos em arquivos compartilhados.

**Why this priority**: A execucao em paralelo por multiplos agentes so e confiavel se houver garantia matematica de ausencia de dependencias circulares e isolamento de arquivos. O planejamento de ondas (Gate 3) organiza o trabalho em lotes deterministas e independentes.

**Independent Test**: Pode ser testado submetendo conjuntos de tarefas com dependencias lineares, ramificadas, independentes e conflitantes; o sistema deve gerar as ondas topologicas corretas e separar tarefas concorrentes que alteram os mesmos arquivos em ondas sequenciais.

**Acceptance Scenarios**:

1. **Given** um conjunto de tarefas atomicas com declaracoes de precedencia validas e sem referencias circulares, **When** o comando de planejamento de ondas (`wave plan`) e executado, **Then** o planejador gera o cronograma topologico onde tarefas de uma mesma onda possuem independencia total entre si.
2. **Given** duas ou mais tarefas atomicas que nao possuem dependencia direta declarada entre si, mas ambas declaram intencao de modificar o mesmo arquivo de codigo, **When** o planejamento de ondas e executado, **Then** o sistema detecta o conflito de recursos e automaticamente posiciona essas tarefas em ondas distintas e consecutivas.
3. **Given** um conjunto de tarefas contendo uma cadeia de dependencia circular (por exemplo, Tarefa A depende de Tarefa B e Tarefa B depende de Tarefa A), **When** o comando de planejamento e invocado, **Then** o planejador aborta a operacao, rejeita o cronograma e exibe o caminho exato do ciclo detectado no grafo.
4. **Given** que a especificacao da funcionalidade ainda se encontra no estado de rascunho nao verificado (`DRAFT_UNVERIFIED`), **When** o planejamento de ondas e solicitado, **Then** o comando recusa a execucao e orienta o usuario a concluir e congelar a especificacao antes de planejar as ondas.

---

### Edge Cases

- O que acontece se o documento de entrada `--from-prd` for um arquivo binario ou nao codificado em UTF-8? O sistema detecta a falha de decodificacao de caracteres antes do processamento, aborta imediatamente com codigo de erro apropriado e nao gera arquivos de saida.
- O que acontece se a Matriz de Rastreabilidade contiver referencias a tarefas que nao foram declaradas no documento de tarefas ou vice-versa? O validador acusa a discrepancia referencial e aponta as entidades ausentes.
- O que acontece se um arquivo de tarefa nao possuir secao de dependencias ou lista de arquivos afetados? O sistema assume valores especiais padronizados seguros (sem dependencias e lista de arquivos vazia) ou emite aviso especifico se a ausencia de arquivos violar o contrato de atomicidade.
- Como o sistema se comporta caso ocorra falta de espaco em disco ou falha de escrita no momento de salvar o selo no banco de estado? A transacao do banco e revertida integralmente e nenhum selo orfao e persistido.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema DEVE fornecer comando de inicializacao de especificacao capaz de ler arquivos de requisitos externos atraves do parametro `--from-prd <caminho>`.
- **FR-002**: O sistema DEVE extrair deterministicamente identificadores e descricoes de Regras de Negocio (`RN`) e Requisitos Funcionais (`RF`) a partir de tabelas estruturadas em Markdown presentes no documento upstream.
- **FR-003**: O sistema DEVE gerar o arquivo de especificacao V2 (nomeado por padrao como `<titulo-slug>-spec.md` derivado do primeiro cabecalho H1 do PRD ou customizado via `--name <slug>`), contendo o modelo de Product Trio, diagrama conceitual, criterios Gherkin e a Matriz de Rastreabilidade previamente preenchida com as regras extraidas.
- **FR-004**: O sistema DEVE interromper a execucao com erro e sem modificacoes em disco se o arquivo PRD upstream nao existir ou nao contiver tabelas de regras validas.
- **FR-005**: O sistema DEVE fornecer comando de travamento da especificacao (`spec lock`), localizando a especificacao via registro de issue ativa no banco SQLite (`.trustkit/state.db`) ou atraves da flag `--spec <caminho>`, que valide simultaneamente a sintaxe do diagrama visual Mermaid, a formatacao dos criterios Gherkin e o fechamento da Matriz de Rastreabilidade.
- **FR-006**: O sistema DEVE verificar se 100% das Regras de Negocio (`RN`) declaradas na especificacao estao associadas a pelo menos um Requisito Funcional (`RF`), uma Tarefa (`TASK`) e uma Evidencia de Teste (`E2E`/Screenshot), bloqueando o avanco caso existam regras orfas.
- **FR-007**: O sistema DEVE calcular a assinatura criptografica imutavel (digest seguro de alta performance) do conteudo integral do arquivo de especificacao aprovado no momento do travamento.
- **FR-008**: O sistema DEVE persistir o selo criptografico e a data/hora do congelamento no repositorio de estado local, atualizando o estado do ciclo para `FROZEN`.
- **FR-009**: O sistema DEVE validar se o arquivo de especificacao foi adulterado em relacao ao selo criptografico armazenado antes de autorizar comandos de planejamento e execucao subsequentes.
- **FR-010**: O sistema DEVE fornecer comando de planejamento de ondas (`wave plan`), localizando as definicoes de tarefas atomicas via issue ativa no banco SQLite ou atraves da flag `--tasks-dir <caminho>`, processando os arquivos de tarefas e o indice correspondente.
- **FR-011**: O sistema DEVE construir uma representacao em Grafo Aciclico Direcionado (DAG) das relacoes de dependencia entre todas as tarefas identificadas.
- **FR-012**: O sistema DEVE detectar qualquer dependencia circular no grafo de tarefas e emitir o roteiro completo do ciclo com interrupcao imediata do planejamento.
- **FR-013**: O sistema DEVE identificar tarefas concorrentes que declaram manipulacao dos mesmos caminhos de arquivo e forcar a separacao dessas tarefas em ondas cronologicas sequenciais, adotando a menor numeracao de tarefa (`TASK-NNN`) como criterio deterministico de precedencia.
- **FR-014**: O sistema DEVE emitir o cronograma ordenado de ondas de execucao em formato estruturado legivel para agentes de IA e humanos (suportando saida padrao e modo JSON).
- **FR-015**: O sistema DEVE recusar o planejamento de ondas caso o ciclo de governanca atual nao esteja no estado de especificacao congelada (`FROZEN`).

### Key Entities *(include if feature involves data)*

- **UpstreamDocument**: Representa o documento original de requisitos (PRD/ADR). Possui metadados de origem, caminho do arquivo e colecoes de regras de negocio e requisitos funcionais extraidos.
- **SpecificationV2**: Documento central de governanca de uma funcionalidade. Contem declaracoes do Product Trio, diagrama em Mermaid, cenarios Gherkin, Matriz de Rastreabilidade multidimensional e status de congelamento.
- **TraceabilityMatrix**: Estrutura relacional que vincula cada Regra de Negocio (`RN`) ao seu Requisito Funcional (`RF`), Tarefas de Implementacao (`TASK`), Cenarios Automatizados (`E2E`) e Evidencias Visuais esperadas.
- **CryptographicArtifactSeal**: Selo de integridade imutavel associado a um artefato de especificacao. Contem o identificador da funcionalidade, o hash criptografico calculado sobre o conteudo e o instante temporal do congelamento.
- **TaskNode**: Vertice do grafo de planejamento que representa uma tarefa atomica de desenvolvimento. Contem o identificador da tarefa, lista de tarefas precedentes obrigatorias e lista de caminhos de arquivos a serem alterados.
- **WaveSchedule**: Plano de execucao gerado a partir do grafo topologico. Agrupa conjuntos de tarefas em sequencias de ondas, garantindo que tarefas dentro da mesma onda sejam independentes de ordem e de recursos compartilhados.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O processo de ingestao upstream a partir de um documento PRD/ADR de 500 linhas conclui em menos de 100 milissegundos gerando a especificacao inicial sem perdas de regras.
- **SC-002**: A validacao completa da especificacao e calculo do selo criptografico (`spec lock`) executam em menos de 15 milissegundos para especificacoes tipicas de funcionalidades.
- **SC-003**: 100% das tentativas de congelamento de especificacoes com regras de negocio orfas ou sintaxe Gherkin/Mermaid corrompida sao interceptadas com mensagem de diagnostico exata.
- **SC-004**: O algoritmo de ordenacao topologica e planejamento de ondas via DAG processa grafos de ate 100 tarefas atomicas em menos de 10 milissegundos.
- **SC-005**: 100% dos ciclos de dependencia circular e conflitos de arquivos compartilhados entre tarefas sao detectados e resolvidos deterministicamente antes de qualquer execucao de codigo.
- **SC-006**: Qualquer modificacao nao autorizada de 1 byte no arquivo de especificacao congelada e detectada imediatamente na validacao de integridade.

## Assumptions

- O documento upstream de requisitos e fornecido em formato Markdown compativel com padroes comuns de tabelas e cabecalhos de secao.
- As tarefas atomicas a serem planejadas seguem a convencao de arquivos e identificadores numericos sequenciais no repositorio da funcionalidade.
- O repositorio local de estado (banco SQLite do TRustKit) esta inicializado e disponivel para leitura e gravacao do selo criptografico e dos estados dos gates.
- O ambiente de execucao possui permissoes padrao de leitura e escrita nos diretorios do projeto.
- O modelo de governanca e operado em modo local de linha de comando ou via protocolo de comunicacao estruturado com ferramentas de desenvolvimento.
