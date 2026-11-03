# TASK-005: Implementar Seletor de Runtime com Fallback Gracioso

> **Camada:** Orquestracao
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-002, TASK-004]
layer: ORCH
Arquivos Esperados:
- crates/trustkit-orch/src/supervisor_runtime_selector.rs
test_file: crates/trustkit-orch/tests/supervisor_runtime_selector_tests.rs

---

## Contexto
O ecossistema TRustKit precisa operar tanto em maquinas onde o daemon `herdr-core` esta ativo e configurado quanto em ambientes minimos de CI/CD ou estacoes sem multiplexador PTY instalado. O seletor de runtime deve inspecionar as condicoes locais e selecionar o backend mais adequado com fallback gracioso.

## Escopo
Implementar em `crates/trustkit-orch/src/supervisor_runtime_selector.rs`:
- Estrutura `SupervisorRuntimeSelector`:
  - `detect_available_backends() -> Vec<RuntimeBackendMode>`.
  - `resolve_backend(requested: RuntimeBackendMode) -> Result<ResolvedRuntimeBackend, DomainViolation>`.
- `ResolvedRuntimeBackend`:
  - Contem a referencia abstrata ou adapter configurado (`TerminalMultiplexerPort`).
  - Flag indicando se houve fallback de `Herdr` para `Native`.
  - Registro de telemetria emitindo sequencia `OSC 7501` informando o backend ativo.
- Fallback deterministico:
  - Se `requested == Auto`: tenta conectar ao socket Unix do `herdr-core`; caso indisponivel, seleciona `Native` sem gerar erro.
  - Se `requested == Herdr`: falha explicitamente se o daemon nao estiver alcancavel, emitindo orientacao clara de como iniciar o `herdr-core`.
  - Se `requested == Native`: utiliza diretamente o `AsyncExecutionSupervisor` baseado em pipes de processo.

### Fora do escopo
- Instalacao automatica de binarios ou alteracao de servicos de sistema.

## Dependencias
- TASK-002
- TASK-004

## Premissas
- Zero panics durante a checagem de sockets ou existencia de arquivos.
- Transparencia total para os casos de uso superiores.

## Criterio de Aceite
- [ ] **Dado** o modo `RuntimeBackendMode::Auto` com socket ausente, **quando** resolvido o backend, **entao** seleciona `Native` com flag de fallback ativa e veredito de sucesso.
- [ ] **Dado** o modo `RuntimeBackendMode::Herdr` com socket ausente, **quando** resolvido, **entao** retorna `DomainViolation::InfrastructureBoundaryFault` com mensagem de orientacao de remediacao.

## Notas Tecnicas
- Seguir a arquitetura de portas e adaptadores de `crates/trustkit-orch/`.

## Rastreabilidade
- Origem: RF-057, RN-005
