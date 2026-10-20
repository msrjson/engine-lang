# Diretriz Oficial de Marca, Nomenclatura e Propriedade Intelectual (Engine Lang)

**Document ID**: `ENG-DOC-04.1`
**Status**: Normativo / Diretriz Corporativa
**Data de Ratificação**: Outubro de 2026
**Titularidade**: Engine Lang Core Working Group & Mantenedores

---

## 1. Contexto e Motivação do Batismo

O projeto de linguagem de sistemas anteriormente designado por nomes conceituais transitórios foi oficialmente e definitivamente batizado como **Engine Lang** (e na forma expandida, **Engine Programming Language**).

### 1.1. Justificativa Estratégica e Mercadológica
1. **Identidade com a Tese Arquitetural**: A linguagem elimina intermediários de infraestrutura (bancos relacionais externos, caches Redis, brokers Kafka, connection pools e ORMs). O binário compilado em silício atua diretamente como o próprio **motor unificado de computação, armazenamento persistente (SLS) e malha de replicação**.
2. **Autoridade Industrial**: O termo "Engine" transmite robustez industrial, determinismo e alta performance, posicionando a linguagem no mesmo patamar de seriedade que motores de alta engenharia (como *Storage Engines*, *V8 Engine*, *Unreal Engine* e motores de banco de dados nativos).

### 1.2. Proteção de Marcas e Patentes (Propriedade Intelectual)
1. **Mitigação de Conflitos Preexistentes**: Nomes genéricos ou amplamente registrados no ecossistema global de software (como *Continuum*) representavam vulnerabilidades jurídicas críticas e risco de litígio sob as Classes NIZA 09 (Software/Compiladores) e 42 (Serviços Tecnológicos).
2. **Distintividade Registrável**: A marca combinada **Engine Lang** possui distintividade necessária para registro no INPI, USPTO, EUIPO e órgãos internacionais de patente de software, protegendo:
   - A especificação formal do compilador (`enginec`);
   - O padrão binário de Single-Level Store em formato `.db` (`ENG-SLS-01.1`);
   - A matriz de operadores fechada (`ENG-SPEC-03.1`);
   - O protocolo de evolução estrutural preguiçosa sem migração SQL (LSE-CRC).

---

## 2. Padrões Canônicos de Identidade

| Elemento | Padrão Oficial | Variações Aceitas | Termos Proibidos / Obsoletos |
|---|---|---|---|
| **Nome da Linguagem** | **Engine Lang** | Engine Programming Language, Engine | Nomes anteriores, nomes genéricos soltos |
| **Extensão de Código** | **`.eng`** | — | `.ctm`, `.c`, `.rs` para módulos nativos |
| **Compilador / CLI** | **`enginec`** | `engc` (alias curto) | `ctmc` |
| **Formato de Persistência** | **`.db`** | — | `.nvram`, `.dat`, `.bin` |
| **Arquivo Padrão de Dados** | **`engine.db`** | `./data/engine.db` | `continuum.nvram` |
| **Manifesto do Workspace** | **`Engine.toml`** | — | `Continuum.toml`, `Cargo.toml` como raiz |
| **Magic Header de Storage** | **`ENG\x01`** (`[0x45, 0x4E, 0x47, 0x01]`) | — | `CTM\x01` |
| **Gateways e Pontes** | `engine-bridge` (PG 5432), `engine-web` (HTTP 8090) | — | `ctm-bridge`, `ctm-web` |
| **Protocolo de Malha P2P** | `engine-wire` (QUIC 9443) | — | `ctm-wire` |
| **Ponteiros de Memória SLS** | `engine_ptr_t` (128 bits) | — | `ctm_ptr_t` |

---

## 3. Diretriz de Redação e Apresentação Pública

1. Em documentações, comunicados, artigos técnicos e repositórios, mencione prioritariamente **"Engine Lang"** para assegurar máxima indexação orgânica (SEO) e consolidação da marca.
2. Todo módulo de código canônico deve utilizar a extensão **`.eng`**.
3. Toda referência a armazenamento persistente deve explicitar o arquivo **`.db`** (como `engine.db`), mantendo a explicação de que o arquivo é mapeado em memória persistente/DAX via Single-Level Store.

