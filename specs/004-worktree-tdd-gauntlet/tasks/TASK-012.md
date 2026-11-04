# TASK-012: Implementar Gateway de Pull Request e Movimentacao de Card Await Staging / In review

> **Camada:** Integracao  
> **Estimativa:** M  
> **Status:** Concluido  

---

## Contexto
Ao final do ciclo, após aprovação e certificação, o TRustKit precisa transicionar o card da issue no GitHub Projects v2 para as colunas finais (`Await Staging` no caso de merge direto, ou `In review` no caso de Pull Request) e abrir o Pull Request via GitHub CLI (`gh pr create`) respeitando rigorosamente as Invariantes #12, #14 e #15.

## Escopo
Implementar em `crates/trustkit-gh/src/board_synchronizer.rs` e `crates/trustkit-gh/src/pr_gateway.rs`:
- Suporte às novas colunas do GitHub Projects: `Await Staging`, `In review`, `In Staging`, `Done`.
- Implementação de criação de PR via `gh pr create`:
  - Validar se a conta ativa é válida no gh (`RN-014`).
  - Base branch: `develop`.
  - Revisores configurados.
  - Assignees configurados.
  - Referência de fechamento: Proibido `Closes #<id>`, `Fixes #<id>` ou `Resolves #<id>`; deve usar exclusivamente `Ref #<id>` (`RN-012`).
  - Corpo do PR: inclusão da Matriz de Rastreabilidade ponta a ponta e links HTML com `raw=true` para as evidências do Playwright.
  - Movimentação automática do card para `In review` após a abertura com sucesso.

### Fora do escopo
- Aprovação autônoma do PR no GitHub (responsabilidade humana do Tech Leader).

## Dependencias
- TASK-002 — Requer as extensões de `ProjectBoardGatewayPort`.

## Premissas
- A ferramenta `gh` está instalada e autenticada.
- Em caso de falha de rede temporária, adotar o Caso Especial `DeferredOffline`.

## Criterio de Aceite
- [x] **Dado** uma solicitação de criação de PR contendo "Closes #45", **quando** validada, **entao** o gateway rejeita a execução por violação da Invariante #12.
- [x] **Dado** parâmetros válidos com `Ref #45` e revisores/assignees configurados, **quando** executado, **entao** o PR é aberto e o card transiciona para `In review`.

## Notas Tecnicas
- Arquivos: `crates/trustkit-gh/src/pr_gateway.rs`, `crates/trustkit-gh/src/board_synchronizer.rs`.
- Testes com mocks e parsing da saída de comandos `gh`.

## Rastreabilidade
- Origem: RF-011, RF-012, RN-009, RN-012, RN-014, RN-015
