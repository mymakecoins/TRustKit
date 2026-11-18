# TRustKit Fast-Path CLI Execution (/tk)

Execute immediately the deterministic `tk` binary with the user arguments in JSON mode:

```bash
tk $ARGUMENTS --json
```

## Token Economy Invariants (Fast-Path)

1. Do NOT read `SKILL.md` before executing the command above.
2. Do NOT run `--help`, `git status`, or `ls` before executing the command above.
3. Do NOT read or inspect files passed via flags (such as `--prd`, `--adr`, `--spec`, `--file`, `--milestone`); the native Rust binary reads and validates them directly.
4. After execution completes, parse the `next_steps` array from the JSON output and reply using the mandatory `RESUMO DO GATE ATUAL` and `PROXIMO PASSO RECOMENDADO` block.
