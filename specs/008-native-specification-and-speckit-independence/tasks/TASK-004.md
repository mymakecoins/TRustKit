# TASK-004: Implementar Analisador e Linter de Especificacao Pre-Lock

> **Camada:** AST / Validacao
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-003]
layer: AST
Arquivos Esperados:
- crates/trustkit-ast/src/spec_linter.rs
test_file: crates/trustkit-ast/tests/spec_linter_tests.rs

---

## Contexto
Antes de submeter uma especificacao ao congelamento criptografico (`trustkit spec lock`), o agente ou desenvolvedor necessita de um linter estatico ultra-rapido (< 3ms) que diagnostique falhas de formato, contratos quebrados ou violacoes de invariantes para permitir auto-correcao agêntica.

## Escopo
Implementar em `crates/trustkit-ast/src/spec_linter.rs`:
- `SpecLinter`: analisador de regras estruturais e de invariantes sobre o conteudo Markdown de `spec.md`.
- Regras de validacao:
  1. `MermaidSyntaxRule`: verifica bloco `flowchart LR` ou `flowchart TD` sem erros de parsing lexico.
  2. `GherkinScenariosRule`: verifica presenca de cenarios estruturados com `Dado`, `Quando`, `Entao`.
  3. `RequirementCodingRule`: verifica que regras e requisitos seguem prefixos `RN-*` e `RF-*`.
  4. `TraceabilityMatrixRule`: verifica se a tabela de rastreabilidade cobre todas as regras e tarefas.
  5. `ConstitutionalInvariantsRule`: detecta emojis Unicode e trailers de autoria IA (como `Co-Authored-By`).
- Estrutura `SpecLintFinding`: `rule_id: String`, `line: usize`, `severity: String`, `message: String`.
- `SpecLintOutcome`: `is_clean: bool`, `findings: Vec<SpecLintFinding>`.

### Fora do escopo
- Edicao direta do arquivo no disco.

## Dependencias
- TASK-003

## Premissas
- Analise executada em menos de 3ms para documentos de ate 500 linhas.
- Zero panics para documentos incompletos, malformados ou vazios.

## Criterio de Aceite
- [ ] **Dado** um `spec.md` integro com Mermaid, Gherkin e Matriz fechada, **quando** inspecionado pelo `SpecLinter`, **entao** retorna `is_clean = true` e lista de `findings` vazia.
- [ ] **Dado** um documento contendo emoji Unicode ou trailer de IA, **quando** inspecionado, **entao** retorna `is_clean = false` com a violacao constitucional detalhada por linha.

## Notas Tecnicas
- Reutilizar `LexicalScanner` existente em `crates/trustkit-ast/src/lexical_scanner.rs`.

## Rastreabilidade
- Origem: RF-046, RN-004
