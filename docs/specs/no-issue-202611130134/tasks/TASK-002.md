# TASK-002: Scanner Lexico Bloqueando Probes DEBUG Residuais

layer: AST
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-ast/tests/lexical_scanner_tests.rs
expected_files:
  - crates/trustkit-ast/src/lexical_scanner.rs
  - crates/trustkit-ast/tests/lexical_scanner_tests.rs

## Contexto
Scanner Lexico Bloqueando Probes DEBUG Residuais atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** um arquivo de codigo-fonte contendo logs de instrumentacao temporaria com o prefixo `[DEBUG-` (ex: `println!("[DEBUG-a4f2] state={:?}", s)`)
- **Quando** o `InvariantLexicalScanner` executa durante o `trustkit gauntlet ast-audit` ou `trustkit done`
- **Entao** o scanner emite uma violacao de invariante lexical impedindo a homologacao ate que todos os marcadores `[DEBUG-` sejam removidos.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-debug-tag-lexical-guard.json
