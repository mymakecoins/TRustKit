# Quickstart Validation Guide: Motor de Spec V2, Ingestao Upstream e Planejamento de Ondas DAG

Este guia documenta os cenarios executaveis de validacao de ponta a ponta para homologar a Fase 2 do TRustKit (`spec init --from-prd`, `spec lock` e `wave plan`).

---

## Pre-requisitos de Ambiente

1. Rust toolchain `>= 1.82` instalado:
   ```bash
   cargo --version
   ```
2. Binario compilado no workspace:
   ```bash
   cargo build --workspace
   ```

---

## Cenario 1: Ingestao Upstream a partir de PRD (`trustkit spec init --from-prd`)

### 1. Criar PRD de teste
```bash
mkdir -p /tmp/trustkit-quickstart/docs
cat << 'EOF' > /tmp/trustkit-quickstart/docs/PRD_Sample.md
# Central Auth Service

## 1. Regras de Negocio

| ID | Descricao |
| :--- | :--- |
| RN-001 | Todo usuario administrativo deve utilizar autenticacao em dois fatores |
| RN-002 | Senhas devem expirar apos 90 dias de inatividade |

## 2. Requisitos Funcionais

| ID | Descricao |
| :--- | :--- |
| RF-001 | Validar token TOTP de 6 digitos no login |
| RF-002 | Bloquear sessao e disparar e-mail de redefinicao de senha aos 90 dias |
EOF
```

### 2. Executar ingestao
```bash
cd /tmp/trustkit-quickstart
trustkit spec init --from-prd docs/PRD_Sample.md --output-dir specs/001-central-auth --json
```

### 3. Resultado Esperado
- Saida JSON com `status: "SPEC_INITIALIZED"`, `topic_slug: "central-auth"`, `rules_extracted_count: 2`.
- Arquivo `specs/001-central-auth/central-auth-spec.md` gerado contendo o esqueleto da Spec V2, bloco Mermaid e Matriz de Rastreabilidade pré-populada com `RN-001` e `RN-002`.

---

## Cenario 2: Validacao e Congelamento Criptografico (`trustkit spec lock`)

### 1. Testar Rejeicao de Spec Incompleta (Regra Orfa na Matriz)
Se a Spec contiver `RN-002` declarada mas a tabela de rastreabilidade nao mapear uma `TASK` para ela:
```bash
trustkit spec lock --spec specs/001-central-auth/central-auth-spec.md --json
```
**Resultado Esperado**:
- Exit code `3`.
- Saida no Stderr com `status: "SPEC_LOCK_REJECTED"`, `rejection_kind: "TraceabilityGap"`, `orphan_rule: "RN-002"`.

### 2. Testar Congelamento com Sucesso
Apos preencher a Matriz de Rastreabilidade fechando todas as linhas com TASK e Screenshot:
```bash
trustkit spec lock --spec specs/001-central-auth/central-auth-spec.md --json
```
**Resultado Esperado**:
- Exit code `0`.
- Saida no Stdout com `status: "SPEC_FROZEN"`, `spec_lock_state: "FROZEN"`, e hash `blake3_digest` de 64 caracteres.
- Registro inserido em `artifact_seals` e status atualizado na tabela `sdd_cycles`.

---

## Cenario 3: Planejamento Topologico de Ondas via DAG (`trustkit wave plan`)

### 1. Criar tarefas de teste com dependencia e arquivo compartilhado
```bash
mkdir -p specs/001-central-auth/tasks
cat << 'EOF' > specs/001-central-auth/tasks/TASK-001.md
# TASK-001: Modelo de Dados
depends_on: []
Arquivos Esperados:
- src/auth.rs
EOF

cat << 'EOF' > specs/001-central-auth/tasks/TASK-002.md
# TASK-002: Validador TOTP
depends_on:
- TASK-001
Arquivos Esperados:
- src/totp.rs
EOF

cat << 'EOF' > specs/001-central-auth/tasks/TASK-003.md
# TASK-003: Logger de Auditoria
depends_on: []
Arquivos Esperados:
- src/auth.rs
EOF
```

### 2. Executar planejamento de ondas
```bash
trustkit wave plan --tasks-dir specs/001-central-auth/tasks --json
```

### 3. Resultado Esperado
- Como `TASK-001` e `TASK-003` tocam o mesmo arquivo (`src/auth.rs`), o `WaveExecutionPlanner` automaticamente isola as duas tarefas em ondas sequenciais distintas, priorizando a de menor índice (`TASK-001`).
- `Wave 1`: contem `TASK-001`.
- `Wave 2`: contem `TASK-002` (que dependia de `TASK-001`) e `TASK-003` (que compartilha arquivo com `TASK-001`).
- Total de ondas: 2 (ou 3 caso encadeamento sequencial adicional). Zero conflitos em paralelo.

---

## Cenario 4: Deteccao de Dependencia Circular

Se `TASK-001` depender de `TASK-002` e `TASK-002` depender de `TASK-001`:
```bash
trustkit wave plan --tasks-dir specs/001-central-auth/tasks --json
```
**Resultado Esperado**:
- Exit code `4`.
- Saida no Stderr com `status: "CYCLE_DETECTED"`, `error_kind: "DagValidationError::CycleDetected"` e lista `cycle_path: ["TASK-001", "TASK-002", "TASK-001"]`.
