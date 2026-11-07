# Backlog de evolução contínua do harness do TRustKit

As issues abaixo estão numeradas na ordem sugerida de implementação. As cinco primeiras operacionalizam o ciclo de aprendizagem e promoção de melhorias. As duas últimas tratam dos pontos de governança identificados no PRD. Os IDs são provisórios; podem ser substituídos pelos números gerados pela plataforma.

---

# [TK-HARNESS-01] Registrar resultados pós-certificação vinculados às execuções

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | `trustkit-core`, `trustkit-db`, `trustkit done`, relatório de certificação |
| **Tipo de Demanda** | Nova Feature |
| **Origem** | Monitoramento / evolução contínua do harness |
| **Interface / Localização** | CLI `trustkit`; banco SQLite WAL; `gauntlet-report.md` |
| **Regra de Negócio / Ref.** | RN-002, RN-003, RF-015; glossário `ExecutionEvidence` e `TraceabilityLink` |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo comportamento |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como responsável técnico pelo TRustKit, quero relacionar cada ciclo certificado aos resultados observados após a homologação, para saber se a execução não só passou nos gates, mas também entregou o resultado esperado sem retrabalho ou regressões.

## Cenário Atual / Comportamento Observado

O PRD descreve evidências técnicas append-only, hashes, exit codes, rastreabilidade de requisitos a tarefas e testes, além de metas de tempo, tokens e conformidade. Não especifica como registrar, junto ao ciclo, o resultado posterior da homologação, defeitos escapados, correções humanas ou rollback. Com isso, os dados de execução não fecham o vínculo entre conformidade técnica e resultado real.

## Comportamento Esperado

1. Cada ciclo deve possuir uma referência identificável à versão do harness, à especificação, aos gates e às evidências de execução.
2. O ciclo deve aceitar eventos de resultado pós-certificação, como homologação aprovada/reprovada, defeito pós-release, retrabalho, rollback ou resultado de negócio disponível.
3. Cada evento deve registrar fonte, timestamp, ciclo relacionado e evidência verificável, preservando o caráter append-only do histórico.
4. A ausência de resultado externo deve ser representada explicitamente como pendente ou não coletado, sem inferir sucesso.
5. `trustkit done` e o relatório devem expor os resultados conhecidos e distinguir fatos observados de dados ainda indisponíveis.

## Passos para Reproduzir / Jornada do Usuário

1. Executar um ciclo SDD até a certificação.
2. Registrar o resultado da homologação ou incidente associado ao ciclo.
3. Consultar o ciclo e seu relatório.
4. Confirmar que o resultado está ligado às evidências e à versão do harness usada.

## Impacto e Riscos

* **Operacional / Negócio:** Sem resultados posteriores, não é possível identificar se a certificação prevê qualidade em uso nem priorizar melhorias por impacto real.
* **Técnico / SLA / Risco:** Correlação inadequada ou alteração de registros comprometeria análises futuras. Eventos devem ser versionados, vinculados ao ciclo e protegidos contra sobrescrita silenciosa.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Modelo de dados e migrações suportam eventos pós-certificação vinculados a `cycle_id` e à versão do harness.
- [ ] Eventos são append-only e incluem tipo, fonte, timestamp e referência a evidência, com validações determinísticas.
- [ ] CLI ou interface equivalente permite registrar e consultar resultados sem bloquear o fluxo headless.
- [ ] Relatório diferencia resultado observado, pendente e não coletado; nenhum status é inferido por ausência de dados.
- [ ] Testes cobrem resultados repetidos, dados ausentes, eventos conflitantes e integridade histórica.
- [ ] Documentação do modelo e do fluxo de coleta é atualizada.

## Dependências e Referências

* **Issues Relacionadas:** Pré-requisito para TK-HARNESS-02 e TK-HARNESS-03.
* **Documentação de Referência:** PRD TRustKit, seções 5.1.1, 6, 9 e 10; template de issue fornecido.

## Evidências e Anexos

- PRD informa event sourcing e evidências técnicas, mas não define o registro estruturado de resultados pós-certificação.

---

# [TK-HARNESS-02] Consolidar métricas de qualidade, tempo e custo por ciclo e versão

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | Observabilidade, `trustkit-db`, `trustkit report` |
| **Tipo de Demanda** | Melhoria |
| **Origem** | Monitoramento / descoberta de produto |
| **Interface / Localização** | CLI `trustkit`; relatório agregado por ciclo, gate e versão do harness |
| **Regra de Negócio / Ref.** | RF-015; seções 7 e 10 do PRD |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo comportamento |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como tech leader, quero comparar qualidade, tempo e custo dos ciclos por versão do harness, para detectar ganhos reais sem sacrificar confiabilidade nem transferir custo para retrabalho posterior.

## Cenário Atual / Comportamento Observado

O PRD define métricas de latência e metas de redução de tempo e tokens, além de metas de conformidade. Não especifica um relatório consolidado por ciclo e versão que combine aprovação na primeira tentativa, intervenções humanas, falhas pós-certificação, custo por gate e defeitos escapados.

## Comportamento Esperado

1. Consolidar métricas calculáveis a partir das evidências existentes e dos resultados pós-certificação registrados na TK-HARNESS-01.
2. Apresentar recortes por período, versão do harness, gate e tipo de execução.
3. Incluir pelo menos: duração total e por gate; tentativas do Gauntlet; aprovação na primeira tentativa; intervenções humanas; consumo de tokens/custo quando disponível; defeitos pós-certificação e rollback.
4. Exibir denominadores, janela temporal e cobertura dos dados para evitar conclusões com amostras incompletas.
5. Permitir saída JSON estável para automação e saída legível para revisão humana.

## Passos para Reproduzir / Jornada do Usuário

1. Executar ciclos com dados de execução e, quando disponíveis, resultados pós-certificação.
2. Solicitar relatório por intervalo ou versão.
3. Comparar duas versões do harness e verificar métricas, cobertura e amostras consideradas.

## Impacto e Riscos

* **Operacional / Negócio:** Sem métricas combinadas, otimizações podem privilegiar velocidade ou aprovação dos gates e ocultar retrabalho e falhas em produção.
* **Técnico / SLA / Risco:** Métricas sem denominadores, cobertura ou definição consistente podem induzir decisões incorretas. Dados inexistentes não devem ser tratados como zero.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Métricas têm definição, fórmula, unidade, origem e janela temporal documentadas.
- [ ] Relatório apresenta duração por gate e total, tentativas, aprovação inicial, intervenção humana e resultados pós-certificação disponíveis.
- [ ] Tokens e custo são apresentados apenas quando houver fonte confiável; ausência é explicitada, não convertida em zero.
- [ ] Comparações por versão incluem tamanho da amostra e cobertura dos dados.
- [ ] Saídas JSON são determinísticas e compatíveis com as convenções do PRD.
- [ ] Testes validam agregações, ciclos incompletos, valores ausentes e comparações entre períodos.

## Dependências e Referências

* **Issues Relacionadas:** Depende de TK-HARNESS-01 para incorporar resultados pós-certificação.
* **Documentação de Referência:** PRD TRustKit, seções 7, 9 e 10.

## Evidências e Anexos

- As métricas e metas atuais estão definidas na seção 10 do PRD; esta issue adiciona agregação e segmentação para suportar decisões de evolução.

---

# [TK-HARNESS-03] Gerar propostas de melhoria do harness a partir de padrões observados

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | `trustkit memory`, análise de ciclos, fluxo de melhoria do harness |
| **Tipo de Demanda** | Nova Feature |
| **Origem** | Monitoramento / evolução contínua do harness |
| **Interface / Localização** | CLI `trustkit`; relatório de análise e proposta versionada |
| **Regra de Negócio / Ref.** | RN-002, RN-018; seções 5.1.5 e 10 do PRD |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo comportamento |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como responsável pelo harness, quero receber propostas de melhoria baseadas em falhas recorrentes e resultados observados, para priorizar mudanças com evidências em vez de ajustar prompts ou regras por intuição.

## Cenário Atual / Comportamento Observado

O PRD inclui memória perene para lições, incidentes e especificações e prevê um Gauntlet que realimenta builders após falhas. Não descreve um processo explícito de análise longitudinal que agrupe falhas entre ciclos e gere propostas rastreáveis para mudar o próprio harness.

## Comportamento Esperado

1. Analisar ciclos concluídos e evidências disponíveis, agrupando padrões por gate, regra, ferramenta, tipo de falha e resultado.
2. Gerar propostas contendo problema observado, evidências e ciclos relacionados, causa provável, mudança sugerida, impacto esperado, riscos e forma de validação.
3. Identificar explicitamente a quantidade e qualidade da amostra, incerteza e hipóteses não confirmadas.
4. Criar a proposta como artefato versionado e revisável; não alterar automaticamente regras, prompts, agentes ou invariantes.
5. Permitir aceitar, rejeitar, adiar ou marcar como duplicada a proposta, registrando justificativa e responsável.

## Passos para Reproduzir / Jornada do Usuário

1. Acumular ciclos com evidências suficientes.
2. Solicitar análise de melhoria para um período ou categoria.
3. Revisar a proposta, suas evidências e riscos.
4. Registrar decisão sem mutar o harness automaticamente.

## Impacto e Riscos

* **Operacional / Negócio:** Falhas repetidas permanecem sem tratamento sistemático, enquanto ajustes manuais podem tratar sintomas isolados.
* **Técnico / SLA / Risco:** Análise automatizada pode confundir correlação com causalidade ou recomendar enfraquecimento de controles. A proposta deve ser explicável e sujeita a revisão.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Análise usa evidências vinculadas aos ciclos e versões, com referências consultáveis.
- [ ] Cada proposta apresenta amostra, padrão observado, hipótese, mudança, impacto, risco e método de avaliação.
- [ ] Evidência insuficiente é sinalizada; o sistema não inventa causa-raiz nem afirma causalidade sem suporte.
- [ ] A geração da proposta não modifica automaticamente arquivos ativos do harness nem regras de governança.
- [ ] Decisões humanas sobre propostas são registradas com autoria, data e justificativa.
- [ ] Testes cobrem duplicidade, baixa amostragem, dados inconsistentes e recomendações conflitantes.

## Dependências e Referências

* **Issues Relacionadas:** Depende de TK-HARNESS-01 e se beneficia de TK-HARNESS-02.
* **Documentação de Referência:** PRD TRustKit, seções 5.1.5, 8, 9 e 10.

## Evidências e Anexos

- A memória FTS5 já suporta recuperação de conhecimento, mas a geração e governança de propostas de mudança do harness não estão descritas como fluxo.

---

# [TK-HARNESS-04] Avaliar versões candidatas do harness em replay e shadow mode

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | Ambiente de avaliação, `trustkit gauntlet`, regressão de harness |
| **Tipo de Demanda** | Nova Feature |
| **Origem** | Auditoria / evolução contínua do harness |
| **Interface / Localização** | CLI `trustkit`; execução de benchmark/replay isolado |
| **Regra de Negócio / Ref.** | RN-002 a RN-005; RN-019; seção 10 do PRD |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo comportamento |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como mantenedor do harness, quero testar uma versão candidata contra casos históricos e execuções controladas antes de colocá-la em uso, para medir melhorias e detectar regressões com segurança.

## Cenário Atual / Comportamento Observado

O PRD descreve CI contínua e validações de harness-score, mas não define um protocolo de comparação entre a versão vigente e uma candidata com corpus fixo, execução isolada, *shadow mode* ou critérios de aprovação.

## Comportamento Esperado

1. Cada avaliação identifica versão vigente, candidata, configuração, corpus, ambiente e hashes dos artefatos.
2. A candidata roda em replay isolado ou shadow mode, sem promover alterações nem executar ações externas não autorizadas.
3. Resultados são comparados com baseline em qualidade, invariantes, taxa de aprovação, tentativas, tempo, tokens/custo e falhas relevantes.
4. O corpus e os critérios são versionados; alterações de corpus são rastreáveis e não podem ser usadas para ocultar regressões.
5. O relatório apresenta diferenças, cobertura, limites da amostra e veredito segundo critérios configurados e aprovados.

## Passos para Reproduzir / Jornada do Usuário

1. Selecionar uma proposta aprovada para avaliação e preparar uma versão candidata.
2. Executar o replay ou shadow mode contra um corpus versionado.
3. Consultar o relatório comparativo e decidir se a candidata pode seguir para promoção controlada.

## Impacto e Riscos

* **Operacional / Negócio:** Sem testes comparativos, mudanças podem degradar comportamento em cenários conhecidos ou ser adotadas sem prova de benefício.
* **Técnico / SLA / Risco:** Corpus enviesado, dados sensíveis ou side effects durante replay podem comprometer a validade da avaliação e a segurança. Isolamento e sanitização são necessários.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] É possível identificar e reproduzir uma avaliação por hashes de versão, corpus, configuração e ambiente.
- [ ] Execução candidata ocorre isolada e não altera o harness ativo nem sistemas externos.
- [ ] Relatório compara métricas relevantes com baseline e mostra amostra, cobertura e limitações.
- [ ] Invariantes críticas e segurança são critérios de bloqueio, não compensáveis por ganho de velocidade ou custo.
- [ ] Casos de replay cobrem falhas históricas, casos positivos e casos de regressão definidos.
- [ ] Testes verificam isolamento, reprodutibilidade, tratamento de dados ausentes e integridade do corpus.

## Dependências e Referências

* **Issues Relacionadas:** Depende de TK-HARNESS-01, TK-HARNESS-02 e TK-HARNESS-03.
* **Documentação de Referência:** PRD TRustKit, seções 5.1.11, 7, 8 e 10.

## Evidências e Anexos

- O PRD declara CI e harness-score, mas não especifica comparação controlada entre versões candidata e vigente.

---

# [TK-HARNESS-05] Promover versões aprovadas do harness com rollback auditável

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | Governança de versão do harness, CI, `trustkit` |
| **Tipo de Demanda** | Nova Feature |
| **Origem** | Auditoria / governança de release |
| **Interface / Localização** | CLI e pipeline de CI; artefatos de configuração `.claude/`, `AGENTS.md` e regras associadas |
| **Regra de Negócio / Ref.** | RN-002, RN-003, RN-005, RN-011; seção 5.1.11 do PRD |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo comportamento |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como mantenedor e operador, quero promover uma versão de harness somente após avaliação aprovada, preservando versões anteriores e uma trilha de auditoria, para obter os benefícios da melhoria com possibilidade de reversão segura.

## Cenário Atual / Comportamento Observado

O PRD descreve artefatos de harness e validações contínuas, mas não define versionamento operacional, critérios de promoção, aprovação, implantação gradual nem rollback específico para versões do harness.

## Comportamento Esperado

1. Uma candidata aprovada pela TK-HARNESS-04 pode ser marcada para promoção, vinculada ao relatório de avaliação e à decisão de aprovação.
2. A promoção cria uma versão imutável com identificador, hash, autor, data, justificativa e conteúdo alterado.
3. A política de promoção define os ambientes e o alcance gradual; mudanças de risco elevado exigem aprovação humana explícita.
4. O sistema mantém a versão anterior e oferece rollback auditável, com motivo e evidências.
5. Promoções e rollbacks são registrados como eventos e não apagam o histórico.

## Passos para Reproduzir / Jornada do Usuário

1. Concluir uma avaliação candidata com resultado aprovado.
2. Solicitar promoção e atender aos requisitos de aprovação definidos.
3. Verificar versão ativa e registro de auditoria.
4. Simular regressão e executar rollback para a versão anterior.

## Impacto e Riscos

* **Operacional / Negócio:** Sem promoção controlada, melhorias testadas não chegam ao uso real de forma consistente ou são aplicadas sem governança.
* **Técnico / SLA / Risco:** Falha de versionamento ou rollback pode deixar ambientes em estado divergente ou dificultar investigação de incidentes.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Cada versão ativa do harness possui identificador imutável e hash do conteúdo.
- [ ] Promoção exige referência a avaliação aprovada e registra aprovador, data e justificativa.
- [ ] A política impede promover candidatas reprovadas ou com violações de invariantes críticas.
- [ ] Rollback restaura uma versão previamente conhecida e gera evento auditável sem apagar registros.
- [ ] É possível identificar qual versão foi usada em cada ciclo.
- [ ] Testes cobrem promoção, concorrência, rollback, versão inexistente e falha parcial.

## Dependências e Referências

* **Issues Relacionadas:** Depende de TK-HARNESS-04; complementa TK-HARNESS-06 e TK-HARNESS-07.
* **Documentação de Referência:** PRD TRustKit, seções 5.1.11, 8, 9 e 10.

## Evidências e Anexos

- Recomendação derivada do ciclo de avaliação e decisão do vídeo: mudanças devem ser comparadas antes da adoção e permanecer reversíveis.

---

# [TK-HARNESS-06] Unificar a política de merge do comando `trustkit auto`

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | `trustkit auto`, `trustkit merge`, governança Git |
| **Tipo de Demanda** | Correção de especificação / Débito Técnico |
| **Origem** | Auditoria do PRD |
| **Interface / Localização** | CLI `trustkit auto` e `trustkit merge`; jornada dos Gates 6 e 6+ |
| **Regra de Negócio / Ref.** | RF-016, RF-021, RN-011; seções 4.1, 4.2 e 6 do PRD |
| **Criticidade no Negócio** | Alta (P1) |
| **Severidade (se bug)** | Maior |
| **Expectativa** | Correção de fluxo e especificação |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como operador do TrustKit, quero uma política única e inequívoca para merges realizados após `trustkit auto`, para saber quais ações são automáticas e quais exigem minha autorização explícita.

## Cenário Atual / Comportamento Observado

O RF-021 afirma que `trustkit auto` executa auto-merge final após certificação. Em contraste, a RN-011 proíbe merge silencioso em branches principais sem validação e autorização expressa. A jornada manual apresenta `trustkit merge --target develop` como ação autorizada do operador, enquanto a jornada autônoma e a visão do produto sugerem automação ponta a ponta. Essa inconsistência pode gerar comportamento imprevisível ou perigoso.

## Comportamento Esperado

1. Definir explicitamente quais destinos e tipos de branch podem ser mesclados por `trustkit auto`.
2. Definir se autorização prévia por flag/política é suficiente ou se o merge deve sempre exigir comando explícito do operador.
3. Alinhar PRD, regras de negócio, critérios BDD, ajuda CLI, documentação e implementação à mesma decisão.
4. Em ausência de política ou autorização inequívoca, o sistema deve parar antes do merge e reportar o estado de certificação sem executar a mutação.
5. Registrar destino, política aplicada, autorização e resultado no event log.

## Passos para Reproduzir / Jornada do Usuário

1. Consultar o comportamento documentado de `trustkit auto` e da RN-011.
2. Executar `trustkit auto` em ciclo certificado sem instrução específica de merge.
3. Confirmar que o comportamento implementado corresponde à política unificada e não realiza merge ambíguo.

## Impacto e Riscos

* **Operacional / Negócio:** A ambiguidade pode surpreender operadores e interromper o fluxo de revisão ou homologação.
* **Técnico / SLA / Risco:** Merge não autorizado pode alterar branches protegidas, integrar código prematuramente ou violar controles do repositório.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Política de merge pós-`auto` está decidida e documentada de forma consistente em todas as seções relevantes.
- [ ] RF-016, RF-021, RN-011 e jornadas apresentam comportamento sem contradições.
- [ ] Critérios BDD cobrem autorização ausente, destino protegido, ciclo não certificado e autorização válida.
- [ ] Comportamento padrão na ausência de autorização é seguro e não realiza merge.
- [ ] Toda tentativa e decisão de merge fica registrada com destino e fonte de autorização.
- [ ] Testes de integração confirmam a política e a compatibilidade com branches protegidas.

## Dependências e Referências

* **Issues Relacionadas:** Independente das issues de medição; deve ser concluída antes de ampliar promoção autônoma do harness.
* **Documentação de Referência:** PRD TRustKit, seções 1.5, 4.1, 4.2, 6 e 8.

## Evidências e Anexos

- Contradição textual entre RF-021 e RN-011, além da diferença entre jornada manual e autônoma descritas no PRD.

---

# [TK-HARNESS-07] Exigir revisão explícita para mudanças no próprio harness e nas invariantes

## Informações Gerais

| Campo | Valor |
|---|---|
| **Módulo / Componente** | Governança L4, regras e agentes `.claude/`, `AGENTS.md`, CI, `trustkit contract` |
| **Tipo de Demanda** | Melhoria |
| **Origem** | Auditoria / governança de harness |
| **Interface / Localização** | Repositório, hooks e pipeline de CI; arquivos de configuração do harness |
| **Regra de Negócio / Ref.** | RN-001 a RN-005, RN-011, RN-018; seção 5.1.11 do PRD |
| **Criticidade no Negócio** | Crítica (P0) |
| **Severidade (se bug)** | Não se aplica |
| **Expectativa** | Novo controle de governança |
| **Status / Etapa** | Pendente (Backlog) |
| **Solicitante / Relator** | Solicitante do backlog de evolução do harness |
| **Responsável (Assignee)** | A definir na triagem |

## Contexto e Visão do Negócio / Usuário

Como responsável por governança, quero que toda mudança no harness — especialmente em invariantes, critérios do Gauntlet, proteções de teste e políticas de segurança — passe por revisão e evidência próprias, para impedir que um mecanismo de autoaperfeiçoamento enfraqueça os controles que o tornam confiável.

## Cenário Atual / Comportamento Observado

O PRD descreve governança L4, regras rígidas, CI e um mecanismo de realimentação de builders. Com a introdução de propostas e avaliação de mudanças no próprio harness, falta formalizar a separação entre sugerir uma melhoria e aplicá-la. Sem uma barreira explícita, uma recomendação automatizada poderia modificar suas próprias regras, testes ou limiares de aprovação.

## Comportamento Esperado

1. Toda mudança de harness deve ser identificada, versionada e vinculada a uma proposta, justificativa e evidências de avaliação.
2. Alterações em invariantes, testes de proteção, critérios de aprovação, controles de segurança e permissões nunca podem ser promovidas apenas por decisão automática do agente que propôs a mudança.
3. Essas alterações exigem aprovador humano autorizado, revisão independente e execução do conjunto de regressão aplicável.
4. O pipeline deve detectar alterações protegidas e exigir o fluxo de aprovação correspondente; a ausência de aprovação bloqueia a promoção.
5. A trilha de auditoria deve registrar conteúdo anterior e novo, hashes, aprovadores, resultados de teste e decisão.

## Passos para Reproduzir / Jornada do Usuário

1. Criar uma proposta que modifique uma regra ou artefato protegido do harness.
2. Tentar avaliá-la e promovê-la sem aprovação humana autorizada.
3. Confirmar o bloqueio e a emissão de diagnóstico claro.
4. Repetir com aprovação válida e regressões aprovadas; verificar trilha auditável.

## Impacto e Riscos

* **Operacional / Negócio:** Sem revisão especial, a evolução contínua pode normalizar falhas recorrentes removendo controles em vez de corrigir a causa.
* **Técnico / SLA / Risco:** Autoalteração de invariantes ou testes pode criar um caminho de escalada de privilégios e invalidar a confiança nos certificados do TrustKit.

## Critérios de Aceite (Definition of Done - DoD)

- [ ] Existe inventário versionado de arquivos e controles protegidos do harness.
- [ ] Mudanças nesses artefatos são detectadas por CI/hook e não podem ser promovidas sem aprovação humana autorizada.
- [ ] Aprovação registra identidade, justificativa, versão candidata e evidência de avaliação.
- [ ] O agente que gerou a proposta não pode aprová-la sozinho quando a mudança afeta controles críticos.
- [ ] Testes cobrem tentativa sem aprovação, aprovação insuficiente, alteração indireta e caminho aprovado.
- [ ] Nenhuma alteração automática modifica invariantes, testes de proteção, threshold de segurança ou política de merge.
- [ ] Documentação explica os papéis, o fluxo de aprovação e a trilha de auditoria.

## Dependências e Referências

* **Issues Relacionadas:** Deve governar TK-HARNESS-03, TK-HARNESS-04 e TK-HARNESS-05; alinhada com TK-HARNESS-06.
* **Documentação de Referência:** PRD TRustKit, seções 5.1.11, 8 e 11.

## Evidências e Anexos

- O PRD define invariantes rígidas, proteção de testes e governança L4; esta issue estende esses controles ao processo de mudança do próprio harness.
