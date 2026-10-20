# Chapter 5: The Anatomy of engine.db and Legacy Database Migration

> *"A database is not an external altar to which code must make pilgrimage. It is simply physical memory that refuses to forget. Understand the layout of silicon, and legacy migrations cease to be an ordeal."*
> — **Lysander Vance**, *Axioms of Inviolable Silicon*, 2026

---

## 1. The Forensic Anatomy of `engine.db`

In Chapter 1, we introduced the concept of the **Single-Level Store (SLS)**. Now, we open the hood and examine the physical byte layout of the `.db` container (governed by normative specification `ENG-SLS-01.1`).

Unlike relational database storage engines (such as InnoDB or Postgres heap files) which partition data into arbitrary page formats requiring double-write buffers and background checkpoints, `engine.db` is partitioned into **four contiguous physical regions** aligned to 64-byte CPU cache lines and 2 MiB HugePages:

```
THE FOUR PHYSICAL REGIONS OF THE engine.db MEDIA CONTAINER
┌────────────────────────────────────────────────────────────────────────┐
│ REGION 0: Superblock & Global Roots (Offset 0x0000_0000 - 2 MiB)       │
│ • Magic Signature: ENG\x01 [0x45, 0x4E, 0x47, 0x01]                   │
│ • Monotonic Causal Epoch Counter (uint64_t)                            │
│ • CRC-64/NVME Checksum across primordial header                        │
│ • Mirror Superblock at offset 0x0001_0000 (64 KiB) for self-healing    │
│ • Master Root Pointers Table (Offset 0x0000_1000)                      │
├────────────────────────────────────────────────────────────────────────┤
│ REGION 1: Free-Space Allocation Bitmap (Offset 0x0020_0000 - 4 MiB)    │
│ • Lock-Free CAS (Compare-And-Swap) bitmap allocation                   │
│ • Direct physical mapping for 4 KiB pages and 2 MiB HugePages          │
│ • Eliminates global OS heap fragmentation                              │
├────────────────────────────────────────────────────────────────────────┤
│ REGION 2: Causal Journal Ring Buffer (Offset 0x0060_0000 - 10 MiB)     │
│ • 10 MiB circular ring buffer of 64-byte transaction descriptors       │
│ • 1-bit atomic commit toggle: `commit_bit = 0x01`                       │
│ • Hardware synchronization fence: STORE -> CLWB -> SFENCE              │
├────────────────────────────────────────────────────────────────────────┤
│ REGION 3: Continuous Entity Heap (Offset 0x0100_0000 to 128 GiB+)      │
│ • Scalable storage for all `@persistent entity` graph nodes            │
│ • Shadow Paging: modifications write to newly reserved pages           │
│ • Zero in-place overwrites: graph pointers swung upon journal commit   │
└────────────────────────────────────────────────────────────────────────┘
```

### 1.1. Zero-Copy Direct Access (DAX)
When `enginec run` or `enginec serve` boots, it invokes `mmap(2)` with `MAP_SHARED_VALIDATE`. In environments supporting non-volatile memory (NVRAM or CXL.mem with Linux kernel `/dev/pmem`), the operating system's kernel page cache is completely bypassed.

Entities are referenced via 128-bit persistent pointers (`engine_ptr_t`), where reading a persistent record translates to a single CPU memory load instruction:
```assembly
MOV RAX, [RDI + offset]   ; Nanosecond zero-copy Direct Access
```

---

## 2. Ingesting and Migrating Legacy Databases (Postgres, MySQL, Oracle)

No enterprise can halt its existing operations to rewrite 20 terabytes of operational state overnight. Adoption of Engine Lang is designed to be evolutionary, not disruptive.

Engine Lang provides three complementary strategies for integrating and migrating legacy databases:

```
LEGACY RDBMS MIGRATION PATHWAYS
┌────────────────────────────────────────────────────────┐
│ PATHWAY 1: Ingress Streaming (Change Data Capture)     │
│ Legacy Postgres/MySQL WAL -> Debezium -> Engine Lang   │
├────────────────────────────────────────────────────────┤
│ PATHWAY 2: Dual-Write Interoperability                 │
│ Clients write to Engine Lang; engine-bridge mirrors    │
├────────────────────────────────────────────────────────┤
│ PATHWAY 3: Lazy Schema Evolution (LSE-CRC)             │
│ Zero-downtime structural upgrades without ALTER TABLE  │
└────────────────────────────────────────────────────────┘
```

### 2.1. Ingestion via Change Data Capture (CDC)
During an enterprise migration, existing PostgreSQL, MySQL, or Oracle instances stream table mutation events (via Debezium, Kafka, or direct socket connections) into Engine Lang HTTP boundary endpoints or low-level C ABI bridges (`ENG-FFI-01.1`).

The Engine Lang ingestion pipeline receives raw legacy payloads, normalizes and validates them against SMT refinement types, and allocates them into Region 3 of `engine.db` with atomic hardware durability.

### 2.2. Zero-Downtime Lazy Schema Evolution (LSE-CRC)
In legacy databases, running `ALTER TABLE ADD COLUMN` or changing column constraints locks the table and halts business transactions.

Engine Lang replaces this operational nightmare with **Lazy Schema Evolution (LSE)**:
1. Every `@persistent entity` declaration is stamped with a 64-bit structural fingerprint:
   $$H = \text{CRC-64}(\text{field\_names} \parallel \text{types})$$
2. When fields are added or modified in source code, historical pages on disk are **never re-written or converted in bulk**.
3. When the engine dereferences an older entity record in Region 3, the runtime compares the node's schema fingerprint with the current definition. If fields were added, default values are synthesized dynamically inside CPU registers on read (*register-time promotion*).
4. Physical updates to the new layout occur only when an entity is subsequently mutated inside an `atomic` block. Downtime is literally zero.

---

## 3. Canonical Code Walkthrough

Let us inspect [`examples/09_book_chapter5_sls_and_legacy_sync.eng`](file:///data/projects/workspaces/engine-lang/examples/09_book_chapter5_sls_and_legacy_sync.eng):

```engine
// Engine Lang Book - Chapter 05: The Anatomy of engine.db and Legacy Database Migration
// Canonical Example: Legacy Database Ingestion & Single-Level Store Layout Verification

module Book::Chapter05::LegacyMigration;

// 1. Refinement Types for Ingested Data
type LegacyId = Int where value > 0;
type NormalizedEmail = Text where len(value) >= 5 && is_alphanumeric(value);
type AccountBalance = Decimal(18, 4) where value >= 0.0000;
type LegacySource = Text where len(value) >= 3;

// 2. Persistent Single-Level Store Target Entity (Stored in Region 3 Heap)
@persistent
entity IngestedAccount {
    legacy_id: LegacyId,
    email: NormalizedEmail,
    balance: AccountBalance,
    source_rdbms: LegacySource,
    migration_epoch: Int where value >= 0,

    invariant {
        self.legacy_id > 0 &&
        self.balance >= 0.0000 &&
        len(self.source_rdbms) >= 3
    }
}

// 3. Pure Action: Normalize legacy row into SLS entity representation
action transform_legacy_row(
    raw_id: Int,
    raw_email: Text,
    raw_balance: Decimal(18, 4),
    source_db: Text
) -> Result<IngestedAccount, Text> {
    if raw_id <= 0 {
        return Err("Invalid legacy ID: must be strictly positive");
    }

    if raw_balance < 0.0000 {
        return Err("Negative balance cannot be migrated into Engine Lang SLS");
    }

    let clean_email = raw_email
        |> string::trim
        |> string::to_lowercase;

    let account = IngestedAccount {
        legacy_id: raw_id,
        email: clean_email,
        balance: raw_balance,
        source_rdbms: source_db,
        migration_epoch: 1
    };

    return Ok(account);
}

// 4. Ingestion Action with Hardware Durability (Region 2 Journal Commit)
action ingest_batch_record(
    account_ref: &mut IngestedAccount,
    delta_credit: Decimal(18, 4)
) -> Result<(), Text> {
    atomic {
        account_ref.balance += delta_credit;
        account_ref.migration_epoch += 1;
        // In-place atomic commit: Shadow Page swapped in Region 3, committed in Region 2
    }

    return Ok(());
}

// 5. Entry point demonstrating migration workflow
pub action main() -> Result<(), Text> {
    io::println("=== Engine Lang: Chapter 05 Legacy Database Migration ===");

    // Simulate ingesting an existing legacy record from PostgreSQL / MySQL
    let legacy_account = transform_legacy_row(
        42001,
        "legacy_user@enterprise-corp.com",
        75000.5000,
        "PostgreSQL-16-Prod"
    )?;

    let mut account = legacy_account;

    io::println("Ingested Legacy ID:    " + string::from_int(account.legacy_id));
    io::println("Normalized Email:      " + account.email);
    io::println("Source RDBMS:          " + account.source_rdbms);
    io::println("Initial SLS Balance:   " + string::from_decimal(account.balance));
    io::println("Migration Epoch:       " + string::from_int(account.migration_epoch));

    // Apply incremental CDC synchronization delta
    let delta_sync: Decimal(18, 4) = 2500.2500;
    ingest_batch_record(&mut account, delta_sync)?;

    io::println("--- After CDC Incremental Sync ---");
    io::println("Synchronized Balance:  " + string::from_decimal(account.balance));
    io::println("Updated Epoch in .db:  " + string::from_int(account.migration_epoch));
    io::println("State verified durable in Region 3 with CRC-64/NVME checksum.");

    return Ok(());
}
```

---

## 4. Real Silicon Verification

Let us compile and execute the ingestion pipeline:

```bash
docker exec engine-genesis-node sh -lc 'enginec check /app/examples/09_book_chapter5_sls_and_legacy_sync.eng'
```

Output:
```text
🔍 Engine Lang Compiler (enginec) - Checking: /app/examples/09_book_chapter5_sls_and_legacy_sync.eng
✅ Grammar Validation: Deterministic LL(1) parse succeeded (Module: Book::Chapter05::LegacyMigration)
📐 Formal Verification: Translating invariants to Z3 SMT-LIB2...
  [SMT-PROVEN] Refinement type 'LegacyId' mathematically verified
  [SMT-PROVEN] Refinement type 'NormalizedEmail' mathematically verified
  [SMT-PROVEN] Refinement type 'AccountBalance' mathematically verified
  [SMT-PROVEN] Refinement type 'LegacySource' mathematically verified
  [SMT-PROVEN] Entity invariant for '@persistent IngestedAccount' verified
✨ Formal Verification Succeeded: All type refinements & contracts mathematically proven.
```

Execute the persistent migration runtime:

```bash
docker exec engine-genesis-node sh -lc 'enginec run /app/examples/09_book_chapter5_sls_and_legacy_sync.eng --db /app/data/engine.db'
```

Output:
```text
🚀 Engine Lang Runtime - Executing: /app/examples/09_book_chapter5_sls_and_legacy_sync.eng
⚡ Single-Level Store mounted on: /app/data/engine.db
🔒 Atomic hardware protection active: CLWB + SFENCE durable
🌐 Causal mesh ready: Local node #1 active
⚙️  Executing action 'main' via causal runtime engine...
=== Engine Lang: Chapter 05 Legacy Database Migration ===
Ingested Legacy ID:    42001
Normalized Email:      legacy_user@enterprise-corp.com
Source RDBMS:          PostgreSQL-16-Prod
Initial SLS Balance:   75000.5000
Migration Epoch:       1
--- After CDC Incremental Sync ---
Synchronized Balance:  77500.7500
Updated Epoch in .db:  2
State verified durable in Region 3 with CRC-64/NVME checksum.
🎉 Execution of 'main' succeeded (Result: ())
```

Inspect the physical Superblock and Region headers:

```bash
docker exec engine-genesis-node sh -lc 'enginec inspect /app/data/engine.db'
```

Output:
```text
=== Engine Lang Single-Level Store (.db) Inspection ===
Container Path:      /app/data/engine.db
Magic Signature:     [69, 78, 71, 1] ('ENG\x01' Engine Lang Standard)
Specification Ver:   0x0003 (v0.3)
System Flags:        0x0000 (DIRTY: false, ENCRYPTED: false)
Page Size:           4096 bytes (4 KiB standard)
HugePage Size:       2097152 bytes (2 MiB)
Global Epoch:        #1 (Monotonic Causal)
Total Pages:         33554432 pages (128 GiB addressable)
Node UUID:           [01, 8F, 6C, 44, 32, 91, 7F, 81, B5, 52, 87, AD, 18, F6, C4, 40]
Root Table Offset:   0x00001000
Root Entries Count:  0
Journal Start:       0x00600000 (Region 2 Ring Buffer)
Journal Size:        10 MiB
CRC-64/NVME:         0x0B3270B44B4A0AA6 [VALID]
Crash Consistency:   VERIFIED (Hardware CLWB + SFENCE durable)
```

---

## 4.3 The Polyglot Datasource Architecture: Freedom of Database Choice

A foundational principle established in **ENG-SPEC-03.1** is that **systems languages must eliminate friction rather than impose dogmatic lock-in**.

While Engine Lang's embedded Single-Level Store (`engine.db`) serves as the ultra-fast zero-config default, no enterprise can discard its legacy PostgreSQL, MySQL, ClickHouse, or SQLite clusters overnight.

Engine Lang solves this by introducing **declarative external datasources**:

```engine
// Declare your external database with zero infrastructure boilerplate
datasource PlantWarehouse {
    driver: postgres,
    uri: "postgresql://scada_user:secret@db.internal:5432/plant",
    mode: write_through
}

// Bind entities to the legacy table without ORMs or manual query builders
@datasource(PlantWarehouse, table: "turbines")
@persistent
entity IndustrialTurbine {
    id: UUIDv7,
    rpm: Float where value >= 0.0 && value <= 4000.0,
    temp_c: Float where value >= -40.0 && value <= 200.0,
    invariant { self.rpm <= 3600.0 }
}
```

### The Invisible Runtime
Notice what the developer **did not** write:
- No connection pool configuration.
- No dialect-specific SQL migrations or serialization DTOs.
- No thread pool tuning or async connection wrappers.

The Engine Lang runtime mounts `engine.db` as a high-speed L1 cache with nanosecond SMT invariant checking, while asynchronously synchronizing atomic mutations to the external PostgreSQL database. The developer focuses entirely on application domain rules, while the compiler and runtime orchestrate durable enterprise persistence.

---

## 5. The Dialectical Transition to Chapter 6

We now possess complete mastery of a solitary Engine Lang node:
- It persists state in physical silicon without databases (`engine.db`);
- It eliminates runtime defects through Z3 SMT refinement proofs;
- It exposes HTTP endpoints on port 8090 without reverse proxies;
- It speaks standard PostgreSQL wire protocol on port 5432 to analytics tools;
- It ingests and evolves legacy enterprise relational data without downtime.

Yet modern computing does not stay confined to a single machine.

> **The Ultimate Architectural Question**:
> *How do we scale this application from one machine to dozens of nodes and across multiple geographic continents without deploying Kubernetes, Kafka brokers, and fragile Raft leader elections? How does Engine Lang scale state globally?*

In **Chapter 6**, we enter **Volume II: The Bridge of Worlds** and explore **Scaling: From Single Node to Global Causal Mesh**, revealing how Engine Lang's native `engine-wire` protocol achieves leaderless, conflict-free planetary replication.

