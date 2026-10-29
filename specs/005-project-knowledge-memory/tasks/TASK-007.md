# TASK-007: Implementar Caso de Uso de Ingestao de Incidente no Gate 5 (Gauntlet e Doctor)

> **Camada:** Dominio / Casos de Uso
> **Estimativa:** P
> **Status:** Concluida

depends_on: [TASK-004]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/ingest_gauntlet_incident_knowledge.rs
test_file: crates/trustkit-core/tests/sdd_gate5_incident_hook_tests.rs

---

## Contexto
Quando o Gauntlet V2 reprova uma iteracao ou o Circuit Breaker desarma apos 5 falhas consecutivas no Gate 5, os diagnosticos e violacoes de invariantes precisam ser convertidos em notas de conhecimento estruturadas em `.trustkit/wiki/incidents/` para evitar que os agentes repitam o mesmo erro em tentativas ou ciclos futuros.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/ingest_gauntlet_incident_knowledge.rs`:
- Caso de uso que recebe os detalhes da falha do Gauntlet ou do acionamento do Circuit Breaker.
- Registra e indexa `.trustkit/wiki/incidents/incident-<cycle_id>-attempt-<n>.md` com tags `["incident", "gauntlet", "post-mortem"]`, saidas sanitizadas, invariantes violadas e wikilink para a especificacao ativa.

### Fora do escopo
- Logica de contagem de falhas do Circuit Breaker (ja existente em `trip_circuit_breaker.rs`).

## Dependencias
- TASK-004

## Premissas
- Nenhuma credencial ou segredo pode constar no resumo de incidente (aplicar sanitizacao).

## Criterio de Aceite
- [x] **Dado** uma reprovacao no Gauntlet ou acionamento do disjuntor no Gate 5, **quando** o caso de uso de ingestao de incidente e disparado, **entao** grava e indexa a nota de pos-mortem em `.trustkit/wiki/incidents/` para consulta preventiva.

## Notas Tecnicas
- Arquivo alvo: `crates/trustkit-core/src/use_cases/ingest_gauntlet_incident_knowledge.rs`.

## Rastreabilidade
- Origem: RF-023, RN-007
