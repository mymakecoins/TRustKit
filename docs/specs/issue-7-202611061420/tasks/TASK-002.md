# TASK-002: Refinar Tratamento de Erros de Parsing e Argumentos Invalidos

> **Camada:** CLI
> **Estimativa:** P
> **Status:** Pendente

depends_on: [TASK-001]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/error_handler.rs
- crates/trustkit-cli/src/main.rs
test_file: crates/trustkit-cli/tests/cli_help_exit_code_tests.rs

---

## Contexto
Argumentos desconhecidos ou invalidos devem continuar encerrando com codigo de erro diferente de zero (1 ou 2) e emitindo mensagens sanitizadas para o stderr ou JSON estruturado quando em modo agente.

## Escopo
- Assegurar que falhas de parsing reais continuem direcionando para stderr com exit code adequado.
- Tratar formatacao JSON consistente caso uma flag `--json` tenha sido passada antes ou junto a um argumento desconhecido.
