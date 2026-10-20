# AGENTS.md — Política e Arquitetura do Time de Agentes (Engine Lang)

**Workspace**: `/data/projects/workspaces/engine-lang`
**Padrão Normativo**: ENG-SPEC-03.1 & ENG-SLS-01.1
**Diretório Central de Agentes**: `.agents/`

---

## 1. Princípios de Governança do Workspace

1. **Diretório Único**: Toda a inteligência do projeto (regras, skills, agentes e handoffs) reside fisicamente em `.agents/`. Nenhuma ferramenta proprietária deve criar pastas concorrentes.
2. **Escrita Restrita ao Workspace**: Qualquer alteração de código ou documentação deve se limitar ao workspace local `engine-lang`.
3. **Padrão de Marca Oficial**: O projeto é exclusivamente **Engine Lang**, compilado via **`enginec`**, com arquivos de código **`.eng`**, manifesto **`Engine.toml`** e persistência **`engine.db`**.
4. **Regra Inegociável de Idioma (100% English)**:
   - Toda a documentação, livros técnicos (`book/`), especificações normativas (`docs/`), whitepapers, código-fonte (`.eng`, `.rs`), docstrings, comentários de código e exemplos DEVEM ser escritos exclusivamente em **inglês**. Nenhuma documentação técnica ou código pode conter português ou outro idioma. O diálogo de chat com o usuário pode ocorrer em pt-BR, mas todo artefato persistido no repositório é rigorosamente em inglês.
5. **Regra Inegociável de Soberania Autoral e Descentralização (Sem Dono / Apache 2.0)**:
   - A linguagem é uma tecnologia pública descentralizada, distribuída livremente sob a **Licença Apache 2.0**, sem proprietário corporativo, monopólio institucional ou autor pessoal singular (evolução como o Bitcoin e Bourbaki).
   - O único pseudônimo literário/canônico admitido é **Lysander Vance** (ou a entidade coletiva **The Vance Protocol Group / Engine Lang Core Working Group**).
   - É **expressamente proibida qualquer menção a nomes reais de desenvolvedores ou pessoas físicas** (em particular qualquer citação ao nome **Antonio Santos** ou variações) no código, na documentação, nos metadados ou nos exemplos.
6. **Validação Obrigatória em Container (Regra 0.1)**:
   - Todo resultado de teste, prova matemática SMT e inspeção de storage deve ser coletado a partir do container `engine-genesis-node`:
     ```bash
     docker exec engine-genesis-node sh -lc 'enginec check /app/examples/04_causal_blog.eng && enginec inspect /app/data/engine.db'
     ```
   - Resultados gerados apenas no ambiente host são considerados `UNVERIFIED`.

---

## 2. Mapa de Papéis e Agentes no Workspace

| Assento | Papel | Agente / Especialidade |
|---|---|---|
| `claude` | Arquiteto & Revisor | [`engine-lang-architect`](.agents/agents/engine-lang-architect.md) (Design de Sistemas, AST, SMT e Memória) |
| `gemini` | Pesquisador & Redator | Documentação, especificações normativas (`ENG-*`), pesquisa e EBNF |
| `qwen` | Operador Local / Testes | Microtarefas, correções pontuais, testes e lint |

---

## 3. Skills Registradas

- [`engine-lang-architecture`](.agents/skills/engine-lang-architecture/SKILL.md): Guia operacional do toolchain, sintaxe `.eng`, comandos do `enginec` e persistência em `.db`.
- [`agent-handoff`](~/.claude/agent-team/skills/agent-handoff/SKILL.md): Protocolo de transição assíncrona entre agentes via `.agents/handoffs/`.

