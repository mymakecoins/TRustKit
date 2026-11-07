Estou tendo dificuldades de entender o que é separado como trabalho a ser executado quando rodo um "trustkit init" contra um PRD do sistema por completo. Dito isso, acredito que precisamos de uma etapa onde o PRD e o ADR são estuados e nesta analise, sejam identificados os milestones do projeto. E a partir desses milestone é que partiríamos com a etapa que hoje é feita pelo init.

Estudar PRD e ADR para decidir os marcos antes de decompor o trabalho em tarefas — eu usaria um comando explícito: trustkit milestones, ou seja, um milestone planning a ser executado antes mesmo do "trustkit init". O comando "trustkit init" por sua vez se basearia no documento do milestone ou no link de uma issue.

Essa etapa costuma ser chamada de **planejamento de release** (*release planning*) ou **definição do roadmap de entrega**. Mas, no nosso contexto — estudar PRD e ADR para decidir os marcos **antes** de decompor o trabalho em tarefas é importante para que o init tenha um foco mais específico na hora de decompor as atividades

O fluxo costuma funcionar assim:

1. **Entender o resultado pretendido** pelo PRD: quem recebe valor, quais problemas serão resolvidos e quais capacidades precisam existir.
2. **Levantar restrições e decisões técnicas** no ADR: arquitetura, integrações, migrações, segurança, infraestrutura e dependências que influenciam a ordem das entregas.
3. **Agrupar capacidades em entregas coerentes**, procurando resultados que possam ser demonstrados ou validados — não simplesmente agrupando por camada técnica.
4. **Definir os milestones** como estados verificáveis do produto: o que estará utilizável, por quem e com quais limitações ao final de cada marco.
5. **Ordenar e validar a sequência**: conferir dependências, riscos, valor entregue em cada marco e se há um caminho utilizável cedo o suficiente.
6. **Só então atomizar** cada milestone em features, histórias e tarefas de implementação.

Por exemplo, para um projeto de criação de um CRM, os marcos poderiam ser:

- **M1 — Base do CRM:** usuários podem acessar o sistema e cadastrar empresas e contatos.
- **M2 — Gestão de oportunidades:** usuários podem criar, atualizar e acompanhar oportunidades em etapas configuráveis do funil de vendas.
- **M3 — Atividades e relacionamento:** usuários podem registrar interações, agendar tarefas e consultar o histórico de relacionamento com cada empresa ou contato.
- **M4 — Visibilidade comercial:** gestores podem consultar o desempenho do funil e relatórios de vendas com filtros relevantes.

Esses marcos ainda **não são tarefas**. São resultados intermediários do produto. A pergunta para cada um é: **“O que será possível fazer ou comprovar quando este milestone estiver concluído?”**

Uma distinção importante: **milestone não é necessariamente uma fase de calendário** — como “mês 1” ou “Sprint 3”. Pode ter uma data-alvo, mas sua definição principal é o resultado verificável. Datas e estimativas não fazem parte do nosso escopo.

Então, para o Trustkit, a etapa anterior à atomização, eu a descreveria como **“Planejamento de Milestones a partir do PRD e ADR”**. 
A saída deve ser uma lista ordenada de milestones, um arquivo pra cada milestone na pasta docs do projeto. Cada um com objetivo, capacidades incluídas, critérios de conclusão, dependências e rastreabilidade aos requisitos, decisões técnicas e referenciamento direto aos arquivos do PRD e ADR com citação das linhas ou conjunto de linhas de cada um dos arquivos de origem do estudo.

Preveja a atualização de toda a documentação do projeto, manuais (markdown e html), readme, grafos do archify, etc...