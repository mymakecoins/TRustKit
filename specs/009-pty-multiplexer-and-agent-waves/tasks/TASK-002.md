# TASK-002: Implementar Codificador e Emissor de Telemetria OSC 7501

> **Camada:** Dominio
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/osc7501_encoder.rs
- crates/trustkit-core/src/entities.rs
test_file: crates/trustkit-core/src/osc7501_encoder_tests.rs

---

## Contexto
O protocolo OSC 7501 padronizado pelo Ghostty e adotado pelo `multiplexador PTY` permite a programas de terminal reportar mecanicamente seu estado operacional (`working`, `blocked`, `done`, `error`), progresso percentual e mensagens estruturadas. Para que o TRustKit seja observavel por multiplexadores externos sem depender de bibliotecas de terceiros, o core deve implementar um codificador deterministico em `std`.

## Escopo
Implementar em `crates/trustkit-core/src/osc7501_encoder.rs` e registrar em `crates/trustkit-core/src/entities.rs`:
- Estrutura `Osc7501Payload`: `state: String`, `id: String`, `kind: String`, `progress: u8`, `app: String`, `title: String`, `msg: String`.
- Funcao deterministica `encode_osc7501(payload: &Osc7501Payload) -> String`:
  - Formata no padrao `\x1b]7501;state=...:id=...:kind=...:progress=...:app=...:title=...:msg=...\x1b\`.
  - Codifica campos textuais `title` e `msg` em Base64 deterministico (utilizando implementacao pura em `std`).
  - Omite campos opcionais vazios.
- Helper `emit_osc7501(payload: &Osc7501Payload)` que escreve na saida padrao caso a variavel de ambiente ou configuracao de telemetria esteja ativa.

### Fora do escopo
- Inclusao de icones Unicode ou emojis nas mensagens reportadas (proibicao estrita de RN-001).
- Parsing de sequencias recebidas no core (pertence ao sanitizador de buffers em `trustkit-orch`).

## Dependencias
- TASK-001

## Premissas
- Implementacao 100% em Rust standard library (`std` only).
- Zero alocacoes excessivas e zero panics.
- Conformidade estrita com o formato esperado pelo `ghostty-vt` e `multiplexador PTY`.

## Criterio de Aceite
- [ ] **Dado** um payload com estado `working`, progresso `45` e app `trustkit`, **quando** codificado via `encode_osc7501`, **entao** gera uma cadeia no padrao `\x1b]7501;...` contendo campos chave-valor delimitados por `:` e terminados com `\x1b\`.
- [ ] **Dado** um payload contendo emojis ou caracteres proibidos, **quando** validado antes da codificacao, **entao** retorna `DomainViolation::AstIntegrityOrInvariantViolation`.

## Notas Tecnicas
- Seguir o padrao de isolamento de `crates/trustkit-core/src/entities.rs`.

## Rastreabilidade
- Origem: RF-054, RN-002
