# TASK-003: Implementar Adaptador Cliente IPC PtySocketAdapter

> **Camada:** Orquestracao / Infraestrutura
> **Estimativa:** L
> **Status:** Pendente

depends_on: [TASK-001]
layer: ORCH
Arquivos Esperados:
- crates/trustkit-orch/src/pty_socket_adapter.rs
- crates/trustkit-orch/src/lib.rs
test_file: crates/trustkit-orch/tests/pty_socket_adapter_tests.rs

---

## Contexto
O daemon `multiplexador PTY` expoe um endpoint OpenRPC / JSON-RPC 2.0 sobre Unix Domain Socket para controle headless de pseudoterminais. O TRustKit deve implementar um adaptador assincrono em Tokio que cumpra a trait `TerminalMultiplexerPort`, comunicando-se com o socket sem introduzir fork ou dependencias proprietarias.

## Escopo
Implementar em `crates/trustkit-orch/src/pty_socket_adapter.rs` e registrar em `crates/trustkit-orch/src/lib.rs`:
- Estrutura `PtySocketAdapter`:
  - `socket_path: std::path::PathBuf`.
  - Conexao assincrona via `tokio::net::UnixStream`.
  - Serializador e despachante JSON-RPC 2.0 (`request_id: u64`, `method: String`, `params: serde_json::Value`).
- Implementacao dos metodos OpenRPC do `multiplexador PTY`:
  - `terminal.run`: repassa `argv`, `cwd`, variaveis de ambiente e timeout.
  - `terminal.get` / `terminal.wait`: traduz a resposta do daemon para `AgentSemanticState`.
  - `terminal.send`: envia caracteres ou comandos ao stdin do terminal gerenciado.
  - `terminal.read`: recupera o buffer de texto recente.
  - `terminal.kill`: encerra o processo do terminal.
- Descoberta automatica do socket ativo nos caminhos convencionais (`/tmp/pty-*.sock` ou sob `PTY_SOCKET`).
- Implementacao da trait `TerminalMultiplexerPort` para `PtySocketAdapter`.

### Fora do escopo
- Fallback para subprocessos tradicionais (gerenciado pelo seletor de runtime na TASK-005).
- Suporte a ConPTY no Windows nesta primeira iteracao (foco inicial em Unix Domain Sockets).

## Dependencias
- TASK-001

## Premissas
- Tratar todas as falhas de conexao e desserializacao convertendo em `DomainViolation::InfrastructureBoundaryFault`.
- Zero uso de `unwrap()` ou `panic!()`.
- Limite maximo de 250 linhas por arquivo modular.

## Criterio de Aceite
- [ ] **Dado** um servidor mock de socket Unix respondendo ao protocolo JSON-RPC 2.0, **quando** o adaptador despacha `terminal.run`, **entao** retorna `MultiplexedTerminalHandle` integro com o `terminal_id` atribuido.
- [ ] **Dado** uma queda de conexao ou resposta de erro com codigo JSON-RPC negativo, **quando** executado, **entao** retorna `DomainViolation::InfrastructureBoundaryFault` com os detalhes do erro sem panics.

## Notas Tecnicas
- Utilizar `tokio::io::AsyncWriteExt` e `AsyncBufReadExt` para framing JSON-RPC delimitado por newline.

## Rastreabilidade
- Origem: RF-055, RN-003
