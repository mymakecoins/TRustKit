# TASK-001: Implementar Entidades e Porta TerminalMultiplexerPort

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Pendente

depends_on: []
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/terminal_multiplexer_entities.rs
- crates/trustkit-core/src/ports.rs
test_file: crates/trustkit-core/src/terminal_multiplexer_entities_tests.rs

---

## Contexto
Para capacitar o TRustKit com execucao interativa de agentes em pseudoterminais isolados sem acoplamento a um multiplexador especifico e mantendo o Principio Constitucional I (Isolamento Hexagonal com std only) e II (Proibicao absoluta de NULL), e necessario introduzir no core os modelos de dados e a porta que governam a criacao, monitoramento e manipulacao de terminais multiplexados.

## Escopo
Implementar em `crates/trustkit-core/src/terminal_multiplexer_entities.rs` e registrar em `crates/trustkit-core/src/entities.rs` e `crates/trustkit-core/src/ports.rs`:
- `MultiplexedTerminalRequest`: `task_id: String`, `working_dir: String`, `command_argv: Vec<String>`, `env_vars: Vec<(String, String)>`, `timeout_ms: u64`.
- `MultiplexedTerminalHandle`: `terminal_id: String`, `task_id: String`, `pid: u32`, `created_at_iso: String`.
- `AgentSemanticState`: enum estrito `Working`, `Blocked { kind: BlockReasonKind, detail: String }`, `Done { exit_code: i32 }`, `Failed { reason: String }`.
- `BlockReasonKind`: enum estrito `Permission`, `Question`, `Authentication`, `Unknown`.
- `RuntimeBackendMode`: enum estrito `Auto`, `Pty`, `Native`.
- Trait de porta pura `TerminalMultiplexerPort: Send + Sync` em `crates/trustkit-core/src/ports.rs` com metodos:
  - `spawn_terminal(&self, request: &MultiplexedTerminalRequest) -> Result<MultiplexedTerminalHandle, DomainViolation>`
  - `query_terminal_state(&self, handle: &MultiplexedTerminalHandle) -> Result<AgentSemanticState, DomainViolation>`
  - `send_terminal_input(&self, handle: &MultiplexedTerminalHandle, input: &str) -> Result<(), DomainViolation>`
  - `read_terminal_buffer(&self, handle: &MultiplexedTerminalHandle, max_lines: usize) -> Result<String, DomainViolation>`
  - `terminate_terminal(&self, handle: &MultiplexedTerminalHandle) -> Result<(), DomainViolation>`

### Fora do escopo
- Conexao direta com sockets Unix ou manipulacao de processos libc (tudo em std puro).
- Implementacao de clientes JSON-RPC (pertence a camada de infraestrutura `trustkit-orch`).

## Dependencias
- Nenhuma

## Premissas
- Zero dependencias externas em `trustkit-core` (`std` apenas).
- Proibicao absoluta de valores `null` ou campos `Option` desnecessarios.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [ ] **Dado** uma requisicao valida `MultiplexedTerminalRequest`, **quando** validada, **entao** constroi o handle e estados semanticos sem panics e livre de `null`.
- [ ] **Dado** uma requisicao com `working_dir` vazio ou `command_argv` vazio, **quando** validada, **entao** retorna `DomainViolation::AstIntegrityOrInvariantViolation`.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-core/src/ports.rs` e `crates/trustkit-core/src/entities.rs`.

## Rastreabilidade
- Origem: RF-053, RN-001
