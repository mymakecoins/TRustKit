Aqui está a seção redigida no padrão formal de Engenharia de Software / ADR (Architectural Decision Record) para inclusão direta no seu PRD:

---

### Apêndice: Racional de Nomenclatura e Identidade Arquitetural (TRustKit)

A escolha do nome **TRustKit** consolida a fusão conceitual de **"True Rust"** com o ferramental (*Kit*) de governança de ciclo de vida para desenvolvimento assistido por agentes de IA. Longe de ser apenas um artifício fonético, o prefixo **T** define formalmente os pilares estruturais e operacionais que diferenciam este harness das implementações tradicionais de mercado:

#### 1. "True Rust" (Nativo vs. Runtimes Interpretados)

A maioria dos harnesses de agentes no mercado opera sobre Python ou Node.js, arcando com o custo de inicialização de interpretadores (300ms a 2s por sub-processo), alto consumo de memória e latência severa em hooks síncronos de ciclo de vida (`PreToolUse` / `PostToolUse`).

O **TRustKit** implementa um binário estático nativo em Rust (*True Rust*), garantindo:

* **Cold-start desprezível (~1 a 3ms):** Hooks e validadores executam instantaneamente no terminal sem quebrar a fluidez da sessão interativa do LLM.
* **Orquestração de baixo nível concorrente:** Uso direto de bibliotecas nativas de sistema (`git2`/`gix` para isolamento de worktrees com reflink copy-on-write e `tokio` para paralelização real dos builders), eliminando o *overhead* de chamadas shell externas em cadeia.

#### 2. Determinismo e Eliminação de Alucinações (*Trust*)

A sobreposição fonética intencional com o termo em inglês **"Trust"** (confiança) estabelece a fronteira entre a criatividade probabilística do LLM e a integridade matemática do código:

* **Invariantes determinísticos:** Validação de conformidade estrutural em microssegundos (fechamento formal da matriz $RN \rightarrow RF \rightarrow TASK \rightarrow E2E$, integridade de blocos Mermaid e critérios Gherkin).
* **Análise estática via AST (tree-sitter):** O binário audita alterações estruturais diretamente na árvore sintática, barrando quebra de contratos, adulteração de testes ou vazamento de escopo antes de acionar turnos caros de LLM para auditoria reflexiva.

#### 3. Especificação do Nome e Mapeamento CLI

* **Nome Formal do Projeto:** `TRustKit`
* **Binário CLI:** `trustkit` (todo em minúsculas para conformidade ergonômica com ambientes POSIX)
* **Slogan / Tagline do Produto:** *"True Rust performance for trusted, spec-driven agentic execution."*

---
