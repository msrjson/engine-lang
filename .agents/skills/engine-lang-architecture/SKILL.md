---
name: engine-lang-architecture
description: Guia e procedimentos arquiteturais para desenvolvimento, verificação formal e execução no ecossistema Engine Lang. Use ao criar ou editar arquivos-fonte .eng, configurar o storage Single-Level Store (.db / engine.db), verificar contratos formais com Z3 SMT, usar o compilador enginec ou rodar testes no container Docker.
---

# Skill: Engine Lang Architecture & Toolchain

## Visão Geral
O **Engine Lang** é uma linguagem de sistemas estaticamente tipada que unifica computação, persistência física em silício (Single-Level Store em arquivo `.db`) e replicação causal ponto a ponto em um único binário compilado (`enginec`), eliminando bancos relacionais externos, caches e brokers.

---

## 1. Estrutura do Workspace e Arquivos

- **Código-Fonte**: Arquivos com extensão canônica **`.eng`** (ex: `examples/01_hello_causal.eng`).
- **Manifesto**: **`Engine.toml`** na raiz do projeto.
- **Armazenamento Físico**: Contêiner binário **`.db`** (padrão: `./data/engine.db` ou `/app/data/engine.db`).
- **Assinatura Binária do Storage**: Superblock alinhado a 64 bytes com Magic Bytes **`ENG\x01`** (`[0x45, 0x4E, 0x47, 0x01]`).

---

## 2. Comandos Canônicos do Compilador (`enginec`)

```bash
# 1. Checagem sintática LL(1) e verificação formal estática Z3 SMT
enginec check examples/04_causal_blog.eng

# 2. Inicializar um contêiner Single-Level Store (.db) com 32 MiB
enginec init-db --path ./data/engine.db --size-mib 32

# 3. Inspecionar o layout físico binário, Superblock e CRC-64/NVME do .db
enginec inspect ./data/engine.db

# 4. Executar um programa com persistência SLS vinculada
enginec run examples/04_causal_blog.eng --db ./data/engine.db

# 5. Iniciar os servidores integrados (HTTP Boundary na 8090 e Postgres Bridge na 5432)
enginec serve --http-port 8090 --pg-port 5432 --host 0.0.0.0 --db ./data/engine.db

# 6. Emissão de LLVM IR com instruções de persistência (CLWB + SFENCE)
enginec emit-ir examples/06_industrial_plc_mmio.eng --target x86_64

# 7. Síntese de Assembly nativo com aceleração SIMD AVX2 (<=>)
enginec emit-asm examples/01_hello_causal.eng --target x86_64

# 8. Compilação AOT para binário executável nativo standalone
enginec build examples/01_hello_causal.eng --output ./hello.bin

# 9. Formatação canônica determinística de código-fonte (.eng)
enginec fmt examples/01_hello_causal.eng --write

# 10. Linting estático de tipos refinados e invariantes formais
enginec lint examples/04_causal_blog.eng

# 11. Servidor Language Server Protocol (LSP 3.17) via stdio
enginec lsp

# 12. Nó de Malha Causal Planetária P2P com relógios vetoriais Delta-CRDT
enginec mesh --node-id node-alpha-genesis

# 13. Síntese de imagem de boot UEFI soberana para Unikernel Bare-Metal
enginec boot-image --target x86_64 --output ./engine-boot.efi
```

---

## 3. Práticas de Modelagem e Sintaxe (`.eng`)

### 3.1. Tipos Refinados e Invariantes SMT
```engine
// Tipo refinado provado estaticamente em tempo de compilação
type Email = Text where len(value) >= 5 && is_alphanumeric(value);
type Balance = Decimal(18, 4) where value >= 0.0000;

// Entidade persistente armazenada diretamente na Região 3 do engine.db
@persistent
@TenantGuard(strict: true)
entity Account {
    id: UUIDv7,
    holder_email: Email,
    balance: Balance,

    invariant {
        self.balance >= 0.0000 &&
        len(self.holder_email) >= 5
    }
}
```

### 3.2. Mutações Atômicas e Durabilidade em Silício
```engine
action transfer(from_acc: &mut Account, to_acc: &mut Account, amount: Balance) -> Result<(), Text> {
    if from_acc.balance < amount {
        return Err("Saldo insuficiente");
    }

    atomic {
        from_acc.balance -= amount;
        to_acc.balance += amount;
        // Ao sair do bloco atomic, o hardware emite instruções CLWB + SFENCE
    }

    return Ok(());
}
```

---

## 4. Validação Obrigatória em Container Docker

Seguindo a política incondicional da equipe (Regra 0.1), todos os testes finais e verificações que encerram tarefas devem vir de dentro do container:

```bash
# Executar a verificação formal de todos os exemplos
docker exec engine-genesis-node sh -lc '
for f in /app/examples/*.eng; do
    enginec check "$f"
done
enginec inspect /app/data/engine.db
'
```

