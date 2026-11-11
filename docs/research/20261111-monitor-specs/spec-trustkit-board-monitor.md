# Especificacao Tecnica: TRustKit Board Monitor Dinamico (`trustkit-board`)

**Identificador**: `20261111-monitor-specs`  
**Comando Principal**: `trustkit-board` (ou `trustkit board`)  
**Status**: Especificacao Tecnica e Requisitos de Aceite  
**Data**: 2026-11-11  
**Autor**: TRustKit Core Team  
**Prototipo Visual de Referencia**: `docs/prototypes/trustkit-kanban-dashboard.html`  

---

## 1. Visao Geral e Proposito

O **TRustKit Board Monitor** e a interface visual operacional, em tempo real e de alto desempenho projetada para monitorar a governanca deterministica de Spec-Driven Development (SDD) e execucao de agentes em Git Worktrees.

O monitor funciona no **Modo Dinamico**, disparado sob demanda atraves do comando `trustkit-board`. Ele inicializa um servidor local embutido de altissima eficiencia escrito em Rust nativo, servindo uma interface Single Page Application (SPA) reativa e alimentando o navegador via **Server-Sent Events (SSE)** diretamente a partir das mutacoes transacionadas no banco SQLite WAL (`.trustkit/state.db`).

---

## 2. Arquitetura do Modo Dinamico (`trustkit-board`)

### 2.1 Modelo de Execucao
- **Processo**: Processo local em Rust embutido na CLI (`trustkit-board` ou subcomando `trustkit board`).
- **Pilha de Rede**: Runtime assincrono Tokio com servidor HTTP leve (Axum / Hyper nativo).
- **Consumo de Recursos**:
  - Memoria RAM residente (RSS): < 20 MB.
  - Uso de CPU em repouso: < 0,1%.
  - Latencia de consulta local SQLite: < 2 ms.
- **Porta Padrao**: `http://127.0.0.1:8970` (configuravel via flag `--port <PORTA>`).
- **Abertura Automatica**: Flag opcional `--open` para acionar o navegador padrao do sistema operacional.

### 2.2 Topologia de Integracao
```text
+-----------------------------------------------------------------------------------+
|                            AGENTES & OPERADORES (CLI)                             |
|  trustkit init | trustkit spec lock | trustkit wave run | trustkit task test     |
+------------------------------------------+----------------------------------------+
                                           | Mutacoes ACID
                                           v
+-----------------------------------------------------------------------------------+
|                        PERSISTENCIA NATIVA (.trustkit/)                           |
|                      SQLite WAL Mode (.trustkit/state.db)                         |
+------------------------------------------+----------------------------------------+
                                           | Notificacao de Mudanca / inotify
                                           v
+-----------------------------------------------------------------------------------+
|                        SERVIDOR LOCAL (trustkit-board)                            |
|             Tokio + Axum (HTTP + Server-Sent Events /api/v1/events)               |
+------------------------------------------+----------------------------------------+
                                           | Push SSE unidirecional
                                           v
+-----------------------------------------------------------------------------------+
|                       INTERFACE WEB DO MONITOR (Navegador)                        |
|   Kanban 7 Gates | Cards com Next-Step | Drawer de Evidencias | Circuit Breakers  |
+-----------------------------------------------------------------------------------+
```

---

## 3. Estrutura do Kanban: Colunas e Status Analogos

O quadro e dividido obrigatoriamente em 7 colunas sequenciais, cada uma vinculada a um Gate da FSM com seu titulo tecnico e subtitulo funcional:

| Indice | Titulo da Coluna | Subtitulo (Status Analogo) | Cor Semantica | Regra de Transicao e Gatilho CLI |
| :--- | :--- | :--- | :--- | :--- |
| **Col 0** | `GATE 0: DISCOVERY` | Em Descoberta / Triagem | Slate (`#64748b`) | Disparado por `trustkit init --issue <ID>`. Valida identidade e workspace. |
| **Col 1** | `GATE 1: SPEC DRAFTING` | Em Refinamento / Especificacao | Sky Blue (`#38bdf8`) | Disparado por `trustkit spec prompt`. Passa por `trustkit spec lint` contínuo. |
| **Col 2** | `GATE 2: SPEC SEALED` | Escopo Congelado / Aprovado | Purple (`#c084fc`) | Disparado por `trustkit spec lock`. Gera selo BLAKE3 imutavel. |
| **Col 3** | `GATE 3: WAVE PLANNED` | Pronto para Execucao (Planejado) | Indigo (`#818cf8`) | Disparado por `trustkit wave plan`. Decompoe tarefas e monta DAG de ondas. |
| **Col 4** | `GATE 4: TASKS IMPLEMENTED` | Em Desenvolvimento (TDD) | Amber (`#fbbf24`) | Disparado por `trustkit wave run`. Executa TDD (RED obrigatorio antes de GREEN). |
| **Col 5** | `GATE 5: GAUNTLET CLEARED` | Em Auditoria / Validacao Estrita | Orange (`#f59e0b`) | Disparado por `trustkit gauntlet run`. Criticos de AppSec, AST e Spec. |
| **Col 6** | `GATE 6: CERTIFIED` | Concluido / Certificado (Done) | Emerald (`#34d399`) | Disparado por `trustkit cycle certify`. Habilita abertura de PR sob demanda. |

---

## 4. Requisitos de Interface e Experiencia do Usuario

### 4.1 Card da Atividade
Todo card apresentado no quadro contera:
1. **Identificador Unico**: Formato `#<CYCLE_ID>-<SLUG>` (ex: `#005-memory-fts5`).
2. **Prioridade Semantica**: `PRIMARY` (vermelho tenue), `SECONDARY` (amarelo tenue), `OPTIONAL` (cinza neutro).
3. **Titulo e Resumo**: Descricao clara da feature ou tarefa ativa.
4. **Badges de Contexto**:
   - Hash BLAKE3 truncado (ex: `BLAKE3: a19c...33de`).
   - Fase de TDD: `TDD: RED PROVED` ou `TDD: GREEN`.
   - Worktree vinculada: `Worktree: wt-mem-1`.
   - Indicador de ondas: `Wave: 1/2`.
   - Contador de falhas/tentativas: `Circuit Breaker: 2/5`.
5. **Bloco NEXT STEP**: Exibicao destacada do comando exato gerado pelo `NextStepResolver` do TRustKit para avancar o ciclo, com botao de copia em um clique.

### 4.2 Estado de Excecao: Circuit Breaker Tripped (Blocked)
- Quando uma atividade acumular 5 reprovacoes consecutivas no Gauntlet, seu card recebe borda vermelha destacada (`#ef4444`) e badge `Circuit Breaker: 5/5 TRIPPED`.
- O card e marcado com status analogo **Impedido / Bloqueado (Blocked)**.
- O Next-Step informa a acao de intervencao humana (`trustkit breaker unlock --cycle <ID>`).

### 4.3 Barra Superior (KPIs)
Apresenta em tempo real:
- Total de Ciclos em Execucao.
- Taxa de Aprovacao Gauntlet V2 (%).
- Quantidade de Git Worktrees ativas (CoW reflinks).
- Invariantes Constitucionais ativas (Zero Emojis, Zero AI Trailers, Std Only, BLAKE3).

### 4.4 Drawer Lateral de Inspecao
Ao clicar no card, abre-se uma gaveta lateral com:
- **Aba Visao Geral**: Dados cadastrais, issue vinculada, prioridade, branch e worktree.
- **Aba Evidencias SDD**: Relatorio de validacao Mermaid/Gherkin e hash de integridade.
- **Aba Audit Gauntlet**: Veredito nominal de cada um dos criticos e logs de execucao.

---

## 5. Contratos de API Local (Endpoints HTTP/SSE)

O servidor embutido fornece uma API REST minima e eficiente:

### `GET /api/v1/board`
Retorna a fotografia atual completa do board com todas as colunas e cards.
- **Formato**: `application/json`
- **Exemplo de Resposta**:
```json
{
  "system": {
    "version": "4.2.0",
    "active_cycles": 6,
    "gauntlet_pass_rate": 0.948,
    "active_worktrees": 3
  },
  "columns": [
    {
      "gate": 0,
      "gate_label": "GATE_0_DISCOVERY",
      "status_label": "Em Descoberta / Triagem",
      "cards": [
        {
          "cycle_id": "008",
          "title": "Native Specification and Autonomy",
          "priority": "PRIMARY",
          "next_step": "trustkit spec prompt --idea \"...\""
        }
      ]
    }
  ]
}
```

### `GET /api/v1/events`
Canal de transmissao reativo unidirecional via **Server-Sent Events (SSE)**.
- **Formato**: `text/event-stream`
- **Eventos Emitidos**:
  - `cycle_created`: Novo card inserido na coluna 0.
  - `gate_transitioned`: Card movido entre colunas.
  - `tdd_phase_changed`: Atualizacao de RED para GREEN no card.
  - `circuit_breaker_incremented`: Contador de tentativas atualizado (ex: 2/5).
  - `circuit_breaker_tripped`: Card bloqueado por excesso de falhas.

---

## 6. Criterios de Aceite (Gherkin)

### Cenario 1: Inicializacao do Monitor Dinamico
**Dado** um repositorio governado pelo TRustKit com banco `.trustkit/state.db` presente,  
**Quando** o operador executa o comando `trustkit-board` (ou `trustkit board`),  
**Entao** o servidor HTTP deve ser iniciado na porta `8970` em menos de 100 milissegundos,  
**E** o consumo de memoria residente (RSS) nao deve exceder 20 MB,  
**E** o terminal deve exibir a URL local `http://127.0.0.1:8970` e o rodape de proximo passo.

### Cenario 2: Atualizacao Reativa sem Bloqueio de Concorrencia
**Dado** que o `trustkit-board` esta ativo e aberto no navegador web,  
**Quando** um agente em segundo plano executa `trustkit spec lock --spec specs/009/spec.md --json`,  
**Entao** a mutacao de selamento deve ser gravada no SQLite WAL sem sofrer lock ou lentidao,  
**E** o monitor web deve receber o evento SSE e mover automaticamente o card da coluna `GATE 1` para `GATE 2`,  
**E** o card deve exibir imediatamente o novo hash BLAKE3 gerado sem necessidade de recarregar a pagina.

### Cenario 3: Resiliencia e Operacao Desacoplada
**Dado** um ciclo em execucao avancando entre os Gates 3 e 4,  
**Quando** o processo `trustkit-board` for encerrado ou nao estiver rodando,  
**Entao** a execucao da CLI e dos agentes deve continuar operando normalmente sem erros ou dependencias externas,  
**E** ao reiniciar o comando `trustkit-board` posteriormente, o estado exibido deve refletir fielmente o banco local.

---

## Anexo: Registro de Alinhamento Estrategico (Q&A de Engenharia)

Este anexo consolida formalmente as questoes levantadas pelo desenvolvedor e as respostas tecnicas que fundamentam os criterios de aceite e as restricoes arquiteturais desta especificacao.

### Questao 1: "Como seria o funcionamento deste board?"
* **Resposta Registrada**:
  O board opera sobre a persistencia ja existente do TRustKit (SQLite WAL em `.trustkit/state.db`). Todas as mutacoes disparadas pela CLI ou por agentes multi-worktree atualizam o banco. O board funciona primariamente no **Modo Dinamico**, onde um servidor local Tokio + Axum/Hyper embutido no binario fornece uma Single Page Application e transmite eventos de transicao de estado em tempo real via Server-Sent Events (SSE) para o navegador.
* **Impacto nos Criterios de Aceite**: Define a arquitetura reativa via SSE (`/api/v1/events`) e a fonte unica da verdade como o banco SQLite.

### Questao 2: "Seria preciso iniciar um trustkit-monitor (Trustkit Monitor) como um server ou servico que monitora o andamento das atividades?"
* **Resposta Registrada**:
  Nao como um daemon/servico de sistema operacional permanente ou obrigatorio. Ele e acionado sob demanda pelo comando `trustkit-board` quando o operador humano desejar acompanhar visualmente o progresso. A ausencia do monitor em execucao nao interfere em nenhum aspecto na operacao da CLI ou na automacao dos agentes.
* **Impacto nos Criterios de Aceite**: Garante o desacoplamento total; a parada do processo `trustkit-board` nao pode em hipotese alguma interromper ou falhar tarefas em execucao.

### Questao 3: "Qual o impacto em processamento?"
* **Resposta Registrada**:
  O impacto em CPU e desprezivel (< 0,1% em repouso). Por utilizar eventos baseados em mudanca de arquivo (`inotify`) e Server-Sent Events (push) em vez de polling periódico em loop infinito, nao ha consumo de ciclos de processamento desnecessarios. A renderizacao visual fica confinada a GPU/CPU do navegador do cliente.
* **Impacto nos Criterios de Aceite**: Estabelece a restricao nao-funcional estrita de uso de CPU inferior a 0,1% em modo ocioso.

### Questao 4: "Qual o impacto em performance?"
* **Resposta Registrada**:
  O binario nativo em Rust consome entre 8 MB e 18 MB de RAM residente (RSS), sem runtime de Node.js, Electron ou JVM. Pelo uso do modo SQLite WAL (Write-Ahead Logging), leituras de tela e escritas dos agentes ocorrem concorrentemente sem contenção ou travamento de banco.
* **Impacto nos Criterios de Aceite**: Estabelece o teto maximo de consumo de RAM em 20 MB e proibe bloqueio de escrita na FSM do TRustKit decorrente de consultas da interface grafica.

### Questao 5: "Quais vantagens podemos tirar deste tipo de visibilidade?"
* **Resposta Registrada**:
  1. Consciencia situacional instantanea para ondas paralelas em Git Worktrees.
  2. Deteccao imediata de Circuit Breakers (ex: tarefa acumulando falhas 2/5 ou 3/5 antes de estourar a cota).
  3. Visualizacao clara da cadeia de custodia (selos BLAKE3 e passagem estrita de TDD RED para GREEN).
  4. Unificacao de linguagem entre tecnologia e produto atraves do subtitulo com o status analogo.
  5. Acionabilidade direta fornecendo o comando CLI contextual em cada card para intervencao ou continuidade.
* **Impacto nos Criterios de Aceite**: Justifica os componentes do card (badges de TDD, selo criptografico, indicador de Circuit Breaker, caixa de comando NEXT STEP e drawer com relatorios de evidencias).
