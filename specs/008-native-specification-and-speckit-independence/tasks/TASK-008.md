# TASK-008: Executar Desacoplamento e Remocao Segura de speckit

> **Camada:** Integracao / Testes E2E
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-007]
layer: INFRA
Arquivos Esperados:
- .agents/skills/speckit-specify/SKILL.md
test_file: crates/trustkit-cli/tests/e2e_speckit_decoupling.rs

---

## Contexto
Para consolidar a emancipacao funcional do TRustKit, as dependencias e referencias legadas de `.agents/skills/speckit-*` devem ser desacopladas e o ciclo de vida completo deve ser testado de ponta a ponta sem qualquer interferencia externa.

## Escopo
Implementar em `crates/trustkit-cli/tests/e2e_speckit_decoupling.rs`:
- Teste E2E cobrindo a jornada completa:
  1. Geracao de prompt com `trustkit spec prompt --idea "..." --json`.
  2. Criacao e preenchimento de `spec.md`.
  3. Validacao com `trustkit spec lint --file ... --json`.
  4. Congelamento com `trustkit spec lock --spec ... --json`.
  5. Planejamento de ondas com `trustkit wave plan`.
- Arquivamento/desativacao segura das skills legadas em `.agents/skills/speckit-*`.
- Validacao de que nenhum script de build, CI ou teste depende de speckit.

### Fora do escopo
- Remocao de dados de ciclos de vida arquivados no SQLite.

## Dependencias
- TASK-007

## Premissas
- A suite de testes completa (`cargo test`) deve passar 100% apos a remocao.

## Criterio de Aceite
- [ ] **Dado** um ambiente onde `.agents/skills/speckit-*` nao esta presente, **quando** executado o teste E2E de ciclo SDD do Gate 0 ao Gate 3, **entao** todo o fluxo e executado com sucesso e zero dependencias externas.
- [ ] **Dado** a remocao de speckit, **quando** executado `cargo test --all`, **entao** nenhum teste existente e quebrado.

## Notas Tecnicas
- O teste E2E deve criar um diretorio temporario isolado para nao impactar o repositorio de desenvolvimento.

## Rastreabilidade
- Origem: RF-050, RN-008
