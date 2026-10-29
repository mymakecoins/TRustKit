# TASK-010: Implementar Guardiao de Evidencias Visuais e Restaurador Alheio (VisualEvidenceGuard)

> **Camada:** Integracao  
> **Estimativa:** M  
> **Status:** Concluido  

---

## Contexto
Durante a execução de suítes de teste Playwright E2E em repositórios corporativos compartilhados, é frequente que testes regravem acidentalmente screenshots pertencentes a issues anteriores. A Invariante #8 do TRustKit exige que toda screenshot de outra issue regravada seja detectada e revertida automaticamente via Git antes da certificação e do commit. Além disso, as capturas da issue atual devem ser validadas quanto ao local de armazenamento.

## Escopo
Implementar em `crates/trustkit-orch/src/visual_evidence_guard.rs`:
- Implementação da porta `VisualEvidencePort`.
- Validação de diretório: garantir que as capturas de tela da issue atual residem estritamente em `docs/specs/<issue-id>-AAAAMMDDHHMM/tests/evidence/`.
- Detecção de screenshots alheias alteradas no diff do Git.
- Restauração automática via `git checkout -- <caminho>` de qualquer arquivo de evidência visual de outras issues.
- Auditoria de conformidade com tags HTML `<img src="https://github.com/.../raw=true" ... />`.

### Fora do escopo
- Renderização visual das telas ou emulação do navegador (feita pelo Playwright).

## Dependencias
- TASK-002 — Requer a trait `VisualEvidencePort`.

## Premissas
- O diff de imagens de outras issues é identificado pelo prefixo do diretório de especificações diferente do `cycle_id` ativo.

## Criterio de Aceite
- [x] **Dado** que uma suíte Playwright alterou um arquivo em `docs/specs/issue-12-.../tests/evidence/01.png` durante o ciclo da `issue-45`, **quando** o `VisualEvidenceGuard` inspeciona, **entao** executa a restauração imediata do arquivo da `issue-12` via Git.
- [x] **Dado** as capturas da `issue-45`, **quando** validadas, **entao** permanecem no stage prontas para commit.

## Notas Tecnicas
- Arquivo: `crates/trustkit-orch/src/visual_evidence_guard.rs`.
- Testes com repositório Git temporário simulando modificações em screenshots alheias.

## Rastreabilidade
- Origem: RF-008, RN-008
