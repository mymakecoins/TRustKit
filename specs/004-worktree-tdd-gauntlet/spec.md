# Especificacao: Fase 4 — Isolador de Worktrees, TDD Guard & Gauntlet V2 (Gates 4, 5 e 6)

**Versao:** 1.0  
**Data:** Outubro/2026  
**Status:** Pronto para Atomizacao e Implementacao  
**Origem:** `docs/PRD_TRustKit.md` (RF-005, RF-006, RF-008, RF-009, RF-010, RF-011, RF-012, RN-001 a RN-016)

---

## 1. Visao e Objetivo da Fase 4

A Fase 4 completa o ciclo de governanca do **TRustKit**, automatizando a execucao fisica das tarefas em ambientes isolados (Gate 4), a auditoria concorrente do Gauntlet (Gate 5) e a certificacao final com publicacao e merge (Gate 6):

1. **Gate 4 (Builders & TDD Estrito):** Isolamento de cada tarefa em Git Worktrees efemeras com Copy-on-Write (`WorktreeIsolationProvider`), fiscalizacao do ciclo RED (falha obrigatoria) -> GREEN (sucesso comprovado) via SQLite (`TddPhaseGuard`), e merge deterministico na branch de trabalho.
2. **Gate 5 (Gauntlet V2 & Swarm Concorrente):** Despacho assincrono (`tokio`) dos 4 Criticos (Adversarial de Dominio/Tecnico, Contrato, Estatico/Build e E2E Playwright), acompanhado do `VisualEvidenceGuard` (restaurador automatico de capturas alheias) e do `CircuitBreakerGuard` (acionamento do disjuntor na 5a falha).
3. **Gate 6 (Certificacao, Merge & PR Sob Demanda):** Geracao deterministica de `gauntlet-report.md`, formatacao do comentario oficial na issue com Matriz de Rastreabilidade total, merge na branch de destino (`trustkit merge`) e abertura de Pull Request exclusivamente sob demanda humana (`trustkit pr`).
4. **Thin Skill & Servidor MCP:** Servidor MCP sobre `stdio` e Skill enxuta em Markdown (`SKILL.md`) atuando em simbiose com o binario nativo.

---

## 2. Requisitos Funcionais e Criterios BDD

### RF-005: Fiscalizacao Deterministica do Ciclo TDD (Gate 4)
- **Cenario 5.1:** Dado que o Builder executou `trustkit task test TASK-001 --phase red` com saida de falha (`Exit Code != 0`), quando `trustkit task test TASK-001 --phase green` passar com sucesso (`Exit Code == 0`), entao `trustkit task complete TASK-001` transiciona a tarefa para `TddLifecycleState::GreenVerified`.
- **Cenario 5.2:** Dado que o Builder tenta executar `--phase green` ou `complete` sem registro previo da fase RED no banco, entao o comando aborta com `TddViolationError::MissingRedPhaseProof` (`Exit Code: 3`).

### RF-006: Isolamento e Merge de Worktrees por Onda (Gate 4)
- **Cenario 6.1:** Dado uma onda com $N$ tarefas independentes, quando `trustkit wave run --wave <W>` e disparado, entao o provedor cria $N$ worktrees em `.trustkit/worktrees/`, executa os Builders em paralelo via `tokio`, valida os commits no padrao `<type>: #<id> <descricao>` e realiza o merge deterministico na branch de trabalho, limpando as worktrees.

### RF-008: Orquestracao do Swarm de Criticos e Playwright E2E — Camada 1 (Gate 5)
- **Cenario 8.1:** Dado que a Camada 0 retornou `CleanPass`, quando `trustkit gauntlet run` inicia a Camada 1, entao dispara em paralelo os 4 Criticos (Dominio, Contrato, Build e E2E), confirma a gravacao das screenshots em `docs/specs/.../tests/evidence/` e restaura via `git checkout --` qualquer screenshot de outra issue regravada.

### RF-009: Disjuntor de Seguranca / Circuit Breaker (Gate 5)
- **Cenario 9.1:** Dado que o ciclo acumulou 4 reprovacoes no Gauntlet, quando a 5a tentativa consecutiva resulta em `FAIL`, entao o ciclo transiciona para `CycleStatus::CircuitBreakerTripped`, bloqueando novos runs e gerando o relatorio `trustkit doctor`.

### RF-010: Certificacao Imutavel e Comentario na Issue (Gate 6)
- **Cenario 10.1:** Dado que a Camada 0, os 4 Criticos e testes possuem eventos `PASS` no SQLite, quando `trustkit done` e invocado, entao renderiza `gauntlet-report.md`, commita as evidencias e gera o comentario oficial na issue com Matriz de Rastreabilidade e tags `<img>` HTML.

### RF-011: Gate de Merge e Transicao de Board
- **Cenario 11.1:** Dado um ciclo em `Certified` e confirmacao de branch alvo, quando `trustkit merge --target develop` executa, entao realiza o merge e move o card no GitHub Projects para `Await Staging`.

### RF-012: Abertura de Pull Request Exclusivamente Sob Demanda
- **Cenario 12.1:** Dado solicitacao explicita `trustkit pr`, entao abre o PR para `develop` com revisor, assignee, referencia `Ref #<id>` e move o card para `In review`.
