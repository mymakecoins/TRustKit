# CLI Contract: `trustkit spec init`

## Command Signature
```bash
trustkit spec init --from-prd <PATH> [--name <SLUG>] [--output-dir <DIR>] [--json]
```

## Description
Inicializa um esqueleto de especificacao V2 (`<slug>-spec.md`) atraves da ingestao upstream de um arquivo PRD ou ADR em formato Markdown.

## Arguments & Flags
- `--from-prd <PATH>` *(Obrigatorio)*: Caminho para o arquivo Markdown upstream contendo as tabelas de Regras de Negocio e Requisitos Funcionais.
- `--name <SLUG>` *(Opcional)*: Sobrescreve o nome do topico/arquivo. Se omitido, deriva o slug a partir do primeiro titulo H1 do PRD.
- `--output-dir <DIR>` *(Opcional)*: Diretorio de destino. Se omitido, utiliza o `spec_dir` da issue ativa ou `specs/<slug>/`.
- `--json` *(Opcional)*: Emite a saida estruturada em JSON (sem campos null).

## Exit Codes
- `0`: Sucesso. Especificacao V2 gerada em disco.
- `2`: `SpecIngestionError::MissingBusinessRules` ou arquivo upstream inexistente/ilegivel. Nenhum arquivo e gerado em disco.

## JSON Payload Structure (Stdout)
```json
{
  "status": "SPEC_INITIALIZED",
  "topic_slug": "central-auth",
  "generated_spec_file": "/path/to/central-auth-spec.md",
  "rules_extracted_count": 8,
  "requirements_extracted_count": 12,
  "business_rules": [
    { "rule_id": "RN-001", "description": "Autenticacao MFA obrigatoria para contas admin" }
  ],
  "functional_requirements": [
    { "req_id": "RF-001", "description": "Sistema valida TOTP de 6 digitos" }
  ]
}
```
