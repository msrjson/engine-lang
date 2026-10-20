# Regras de Arquitetura e Engenharia: Engine Lang

**Escopo**: Obrigatório para todos os agentes de IA, engenheiros e contribuidores que operam no workspace `engine-lang`.

---

## 1. Princípios Arquiteturais Inegociáveis

1. **Identidade Oficial Única**: O nome oficial e exclusivo do projeto é **Engine Lang**. O compilador é **`enginec`**, os arquivos-fonte têm extensão **`.eng`**, o arquivo de manifesto de workspace é **`Engine.toml`**, e os arquivos de armazenamento físico têm extensão **`.db`** (com o padrão **`engine.db`**). Qualquer menção a nomes legados ou extensões depreciadas é estritamente proibida.
2. **Idioma Obrigatório (English Only — Inegociável)**:
   - Todo arquivo rastreado no repositório — código-fonte (`.eng`, `.rs`), documentação (`docs/`), livro oficial (`book/`), whitepapers, manifestos, exemplos (`examples/`) e comentários de código — DEVE ser integralmente redigido em **inglês**. Nenhuma documentação técnica no codebase é aceita em outro idioma.
3. **Neutralidade Autoral e Modelo Descentralizado (Sem Dono / Apache 2.0)**:
   - O Engine Lang é governado como um bem público digital aberto sob a **Licença Apache 2.0**, descentralizado como o protocolo Bitcoin.
   - Qualquer atribuição autoral canônica ou mítica utiliza estritamente o pseudônimo **Lysander Vance** ou o coletivo **Engine Lang Core Working Group**.
   - **É terminantemente vedada a menção a nomes reais de indivíduos ou desenvolvedores** (em especial qualquer menção a **Antonio Santos** ou correlatos) em qualquer parte do codebase, documentação ou artefatos do projeto.
4. **Eliminação de Middlewares Externos**: O código Engine Lang elimina dependências de bancos relacionais externos (PostgreSQL/MySQL), camadas de cache (Redis/Memcached) e mensageria (Kafka/RabbitMQ). O estado é persistido em silício através da **Single-Level Store (SLS)** em `engine.db`.
5. **Persistência Física Baseada em Silício**:
   - Mapeamento direto via Direct Access (DAX) ou memória virtual no arquivo `.db`.
   - Magic Header obrigatório: `ENG\x01` (`[0x45, 0x4E, 0x47, 0x01]`).
   - Alinhamento estrito a 64 bytes (`#[repr(C, align(64))]`) para linhas de cache da CPU e 2 MiB para HugePages.
   - Durabilidade atômica via instruções `clwb` (Cache Line Write Back) + `sfence` (Store Fence) no encerramento de blocos `atomic { ... }`.
4. **Verificação Formal Estática (Z3 SMT)**:
   - Todo contrato (`invariant`), tipo refinado (`where`) e pré/pós-condição deve ser traduzível para SMT-LIB2 e provado pelo Z3 solver antes da compilação.
   - Código com violação formal de contrato não compila. Nenhuma alteração pode desativar ou contornar as provas formais.
5. **Multi-Tenancy e Isolamento Matemático**:
   - Entidades anotadas com `@TenantGuard` possuem separação de dados criptográfica e contextual verificada formalmente. Não é permitido criar rotas públicas (`@locality(public: true)`) que vazem entidades isoladas sem autenticação.
6. **Evolução de Esquema sem Downtime (LSE-CRC)**:
   - Toda alteração em entidades persistentes deve utilizar evolução estrutural preguiçosa com hash CRC-64 (`ENG-MIG-01`). Proibido criar scripts manuais de migração SQL ou travar o processo para reescrita de dados.

---

## 2. Validação e Ambiente de Testes (Regra 0.1 Incondicional)

1. **Dois ambientes, apenas o container conta**:
   - Host local: Apenas para checagens preliminares de edição no editor.
   - **Container Docker (`engine-genesis-node` / `engine-test`)**: O único ambiente de validação oficial.
2. Todo teste reportado ou usado para fechar tarefas deve ser executado no container:
   ```bash
   docker exec engine-genesis-node sh -lc 'enginec check /app/examples/04_causal_blog.eng && enginec inspect /app/data/engine.db'
   ```
3. Se o container estiver inativo, suba-o com `docker compose up -d`. Nunca marque uma validação como concluída sem a evidência do container.

---

## 3. Padrões de Código e Ferramental

1. **Compilação**:
   - Binário principal: `enginec`
   - Alias permitido: `engc`
   - Verificação SMT: `enginec check <arquivo.eng>`
   - Execução: `enginec run <arquivo.eng> --db <caminho.db>`
   - Inicialização de storage: `enginec init-db --path <caminho.db> --size-mib 32`
   - Inspeção de storage: `enginec inspect <caminho.db>`
2. **Versionamento e Especificações**:
   - Todas as especificações técnicas usam a família `ENG-*` (`ENG-SPEC-03.1`, `ENG-SLS-01.1`, `ENG-OP-*`, `ENG-MIG-01`, `ENG-MEM-01.1`).
   - Versão de referência atual: `0.3.0`.

