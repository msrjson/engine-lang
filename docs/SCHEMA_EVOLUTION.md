# Technical Specification: Continuous Schema Evolution & Zero-Downtime Migration
**Document ID**: `CTM-MIG-01`
**Status**: Official Normative Specification
**Architecture**: Lazy Structural Evolution with Causal Redirection (LSE-CRC)
**Verification Layer**: SMT Backward-Compatibility Contract Checking

---

## 1. The Zero-Downtime Migration Problem

In legacy software architectures, schema changes require table locks (`ALTER TABLE`), offline migration scripts (Alembic, Flyway, Prisma), or dual-writing pipelines across decoupled database instances.

In Continuum, where persistent entities reside directly in physical non-volatile memory (NVRAM/CXL), mass rewriting of billions of bytes on application startup would produce severe tail latency spikes and degrade hardware write endurance.

Continuum solves this with **Lazy Structural Evolution with Causal Redirection (LSE-CRC)**, guaranteed by **compile-time SMT backward-compatibility proofs**.

---

## 2. Core Operational Principles

1. **Instantaneous Deployment**: New binary releases take traffic immediately. The `.nvram` file is never batch-migrated on startup.
2. **Structural Coexistence**: Past entity layouts ($v_1$) and new entity layouts ($v_2$) coexist safely in the same physical address space.
3. **On-Demand Upgrade (Read-Synthesize / Write-Reallocate)**:
   - **Read Operations**: The CPU reads legacy $v_1$ bytes and synthesizes missing fields into CPU registers using default expressions without writing to NVRAM.
   - **Write Operations**: When an entity is mutated inside an `atomic { ... }` block, the runtime reallocates the instance in a free page using the $v_2$ layout, updating the causal root pointer atomically via CAS (Compare-And-Swap).
4. **Gentle Background Compaction**: An idle-priority Causal Garbage Collection (CGC) thread migrates cold, untouched legacy entities during off-peak hours without impacting $p_{99}$ latency.

```
[ Layout v1 in NVRAM ] ─────────── (Read-Only) ────────────────► CPU synthesizes v2
        │                                                               ▲
        ▼ (On: atomic { vehicle.field = updated_value; })              │
[ New v2 Allocation ] ────► [ Atomic CAS Pointer Update ] ──────────────┘
```

---

## 3. Entity Headers and Layout Fingerprints

Each persistent entity is prefixed by a 32-byte `EntityHeader` (defined in `CTM-SLS-01.1`). The first 8 bytes contain the **Type Fingerprint**:

```
Offset 0x00: [ Type Fingerprint (64 bits) ] -> Truncated SHA-256 of layout + invariants
Offset 0x08: [ Epoch ID (64 bits)         ] -> Allocation causal epoch
Offset 0x10: [ Tenant Ref ID (64 bits)    ] -> Cryptographic tenant security context
Offset 0x18: [ Object Flags & Status (64b)] -> Bit 3: REQUIRES_UPGRADE
```

The `ctmc` compiler embeds historical layout descriptors into the `.rodata` section of the generated executable:

| Fingerprint Hash | Version | Field Offsets & Descriptors |
|---|---|---|
| `0xA4F1_99B2_108E` | $v_1$ (Legacy) | `[plate: 8b, year: 4b]` |
| `0x3BC8_D011_EE45` | $v_2$ (Current) | `[plate: 8b, year: 4b, renavam: 16b]` |

When a dereference occurs:
- If the fingerprint matches current version `0x3BC8...`, fields are read at direct static offsets.
- If the fingerprint matches legacy version `0xA4F1...`, the runtime activates the **Layout Translation Vector**.

---

## 4. The 4 Declarative Evolution Directives

Continuum enforces declarative directives to guide lazy schema evolution. The compiler rejects structural modifications that lack an explicit migration pathway.

### Rule 1: Field Addition Requires `@default` or `Option<T>`
Adding a field without a default value or an optional type is a compile-time error.

```continuum
// Version 1 (Legacy)
@persistent
entity Vehicle {
    plate: Text,
    year: Int
}

// Version 2 (Evolved)
@persistent
entity Vehicle {
    plate: Text,
    year: Int,

    // Synthesized in registers when reading v1 instances:
    @default("00000000000")
    renavam: Text where len(value) >= 9,

    // Or explicitly optional:
    notes: Option<Text> = None
}
```

### Rule 2: Field Deprecation Requires `@deprecated`
Fields cannot be deleted abruptly; they must be deprecated to ensure historical audit trails and safe rollback:

```continuum
@persistent
entity Vehicle {
    plate: Text,

    // Compiler rejects new code reading/writing this field,
    // but physical layout reserves offset for legacy nodes:
    @deprecated(since: "2.1.0", tombstone: true)
    legacy_code: Int
}
```

### Rule 3: Field Renaming Requires `@alias`
Renaming fields preserves physical offset alignment without binary re-indexing:

```continuum
@persistent
entity CheckingAccount {
    id: UUIDv7,

    // Maps to the physical offset of previous field 'client_name':
    @alias("client_name")
    account_holder: Text
}
```

### Rule 4: Structural Transformation via `@migrate`
When a field changes primitive types (e.g., from integer cents to a refined `Decimal(18, 4)`):

```continuum
@persistent
entity Invoice {
    id: UUIDv7,
    amount: Decimal(18, 4),

    // Invoked on-demand when a v1 node is loaded:
    @migrate(from: "v1")
    fn migrate_v1_to_v2(legacy: RawEntityLayout) -> Self {
        return Self {
            id: legacy.get("id"),
            amount: Decimal::from_cents(legacy.get("amount_cents"))
        };
    }
}
```

---

## 5. Compile-Time Compatibility Proof via SMT

Continuum uses the Z3 SMT solver during `ctmc check` to prove mathematically that a new schema version does not violate invariants established in historical data.

### Example: Invariant Restriction Detection
Suppose Version 1 allowed any positive integer:
```continuum
// Version 1:
entity Subscription {
    grace_period_days: Int where value >= 0
}
```

If Version 2 tightens the invariant:
```continuum
// Version 2 Attempt:
entity Subscription {
    grace_period_days: Int where value >= 30 // Tighter restriction!
}
```

During `ctmc check`, Z3 evaluates the containment formula:
$$\forall x \in \text{Domain}(v_1): \text{Invariant}_{v_1}(x) \implies \text{Invariant}_{v_2}(x)$$

Because $x \in [0, 29]$ satisfies $v_1$ but violates $v_2$, the compiler halts:

```
[SMT-MIG-088] Schema migration backward-incompatibility detected.
  File: src/domain/subscription.ctm:12:5
  New invariant 'value >= 30' rejects existing persistent instances.
  Counterexample:
    grace_period_days = 14
  Mandatory Resolution:
    1. Attach '@migrate_rule' normalizing historical instances < 30 to 30, or
    2. Retain original invariant 'value >= 0' and enforce '>= 30' in new constructor.
```

---

## 6. Zero-Downtime Deployment Lifecycle

| Stage | Application State | NVRAM Data State | Runtime Behavior |
|---|---|---|---|
| **$T_0$ (Pre-Deploy)** | Binary $v_1$ running. | 100% instances in Layout $v_1$. | Normal operations. |
| **$T_1$ (Deploy)** | Binary $v_2$ starts (<3 ms via clean restart). | 100% instances remain in Layout $v_1$. | Zero downtime. Binary $v_2$ loads embedded historical layout descriptors. |
| **$T_2$ (Active Traffic)** | Binary $v_2$ serving requests. | Mixed state: read-only nodes in $v_1$; mutated nodes reallocated as $v_2$. | Reads synthesize $v_2$ registers. Writes commit $v_2$ in NVRAM. |
| **$T_3$ (Compaction)** | Binary $v_2$ during off-peak hours. | Causal GC thread walks cold $v_1$ nodes and migrates them to $v_2$. | Background thread. Zero impact on user transaction $p_{99}$ latency. |
| **$T_4$ (Consolidated)** | Binary $v_2$ active. | 100% instances consolidated into Layout $v_2$. | Binary $v_3$ may safely retire $v_1$ translation descriptors. |

