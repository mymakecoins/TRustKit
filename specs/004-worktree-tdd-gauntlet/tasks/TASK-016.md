# TASK-016: Redigir a Thin Skill Declarativa SKILL.md v4 Integrada ao Binario trustkit

> **Camada:** Infra/Setup  
> **Estimativa:** P  
> **Status:** Concluído  

---

## Contexto
O modelo arquitetural "Hybrid Rust Core + Thin Skill" substitui as antigas skills monolíticas de 400+ linhas por um manifesto enxuto que preserva o KV-Cache do agente LLM. A Thin Skill deve orientar o modelo a delegar todas as operações mecânicas (Gatekeeping, Git, Board, TDD, Gauntlet e PR) ao binário `trustkit`.

## Escopo
Criar o documento da Thin Skill em `skills/trustkit/SKILL.md` (e/ou distribuição correspondente):
- YAML frontmatter com nome, descrição e ativação para fluxos de SDD e Gauntlet.
- Instruções sucintas de jornada:
  - Gate 0: Invocação de `trustkit init --issue <id> [--from-prd <path>] --json`.
  - Gate 1 & 2: Redação da Spec V2 e congelamento via `trustkit spec lock --json`.
  - Gate 3: Redação de tarefas atômicas e planejamento via `trustkit wave plan --json`.
  - Gate 4: Despacho de ondas via `trustkit wave run --wave <W> --json` e fiscalização via `trustkit task test`.
  - Gate 5: Execução do Gauntlet via `trustkit gauntlet run --json`.
  - Gate 6: Conclusão via `trustkit done --json`, merge via `trustkit merge` e PR via `trustkit pr --json`.
- Tabela de códigos de saída semânticos (`0` a `6`).
- Proibição estrita de emojis e trailers de IA no próprio documento da skill.

### Fora do escopo
- Lógica determinística em scripts bash (todo o determinismo reside no binário Rust).

## Dependencias
- TASK-015 — Requer o Composition Root e binário CLI finalizados.

## Premissas
- A skill deve ser ultra-enxuta (< 150 linhas) para máxima economia de tokens na janela de contexto.

## Criterio de Aceite
- [x] **Dado** o arquivo `SKILL.md`, **quando** inspecionado, **entao** não contém nenhum emoji ou trailer proibido.
- [x] **Dado** o passo a passo da jornada, **quando** seguido por um agente LLM, **entao** cada etapa mapeia para exatamente um subcomando determinístico do binário `trustkit`.

## Notas Tecnicas
- Arquivo: `skills/trustkit/SKILL.md`.

## Rastreabilidade
- Origem: Secao 3.3, Secao 4.2 do PRD
