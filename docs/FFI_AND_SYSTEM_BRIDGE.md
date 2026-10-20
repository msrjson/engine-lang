# Engine Lang Foreign Function Interface (FFI) & System Bridge Specification
**Document ID**: `ENG-FFI-01.1`
**Status**: Official Normative Specification
**Interoperability**: C / POSIX ABI, Linux Kernel Direct Access (DAX / PMDK), PostgreSQL Wire Protocol v3.0

---

## 1. Overview and Interoperability Philosophy

While Engine Lang unifies memory and persistence to eliminate cloud middleware, enterprise adoption requires seamless integration with existing operating systems, hardware drivers, and analytical tooling.

Engine Lang provides two primary bridge interfaces:
1. **Low-Level C ABI FFI**: Direct interaction with POSIX system calls, hardware security modules (HSM), and kernel DAX drivers.
2. **PostgreSQL Wire Protocol Bridge (`engine-bridge`)**: Emulates a standard PostgreSQL database server on port 5432, enabling business intelligence tools (DBeaver, Power BI, Metabase, Tableau) to query the `.db` Single-Level Store directly using standard SQL queries without installing client drivers.

---

## 2. Low-Level C ABI Foreign Function Interface

Foreign functions from C/Rust libraries are declared using the `boundary extern "C"` block:

```engine
module System::Crypto;

boundary extern "C" {
    fn secp256k1_ecdsa_sign(
        ctx: *const Void,
        sig: *mut u8,
        msg: *const u8,
        seckey: *const u8
    ) -> Int;
}
```

### Safety and Invariant Rules for FFI:
1. **Memory Isolation**: External C libraries are never passed direct mutable pointers (`&mut T`) to Region 3 persistent heap entities. Data must be copied to volatile buffers before passing to foreign code.
2. **Crash-Consistency Confinement**: Foreign C calls are strictly prohibited inside `atomic { ... }` blocks, as external code cannot guarantee `clwb` + `sfence` hardware ordering.

---

## 3. Linux Kernel Direct Access (DAX) & Hardware PMDK Layer

To achieve sub-microsecond latency, Engine Lang bypasses standard OS page caches using Linux DAX (Direct Access):

```
+─────────────────────────────────────────────────────────────+
|           ENGINE LANG RUNTIME DIRECT HARDWARE ACCESS          |
|                                                             |
|  [ enginec Process Virtual Memory Space ]                      |
|                  │                                          |
|                  ▼ (mmap with MAP_SHARED_VALIDATE | MAP_SYNC)
|  [ /dev/pmem0 or ext4 -o dax Mount Point ]                  |
|                  │                                          |
|                  ▼                                          |
|  [ NVRAM / Intel Optane / CXL 2.0 Physical Memory Controller]
+─────────────────────────────────────────────────────────────+
```

### System Call Configuration
When mounting an `.db` container:
```c
int fd = open("/mnt/pmem/engine.db", O_RDWR | O_DIRECT);
void* sls_addr = mmap(
    NULL,
    size_bytes,
    PROT_READ | PROT_WRITE,
    MAP_SHARED_VALIDATE | MAP_SYNC, // Bypasses page cache, guarantees direct flush
    fd,
    0
);
```

---

## 4. The PostgreSQL Wire Protocol Bridge (`engine-bridge`)

To eliminate the objection that *"Engine Lang is an isolated silo incompatible with enterprise reporting"*, the runtime includes an embedded PostgreSQL v3.0 wire protocol responder:

```
[ DBeaver / Power BI / Tableau ] ────► [ TCP Port 5432: PostgreSQL Wire v3.0 ]
                                                      │
                                                      ▼
                                       [ engine-bridge Virtual Catalog ]
                                                      │
                                                      ▼ (Zero-Copy Offset Read)
                                       [ .db Single-Level Store ]
```

### Supported Operations:
- **`StartupMessage` & `SSLRequest`**: Authenticates connecting clients and maps DB users to Engine Lang `@TenantGuard` tenant IDs.
- **SQL-to-Graph Translation**: Translates simple `SELECT * FROM Vehicle WHERE plate = 'ABC1D23'` into direct binary graph offset lookups (`lea`).
- **Data Type Streaming**: Emits binary tuple messages directly from the physical NVRAM layout without intermediate serialization:
  - `Text` $\to$ `VARCHAR` (OID 1043)
  - `Int` $\to$ `INT8` (OID 20)
  - `Decimal(18, 4)` $\to$ `NUMERIC` (OID 1700)
  - `Vector(1536)` $\to$ `pgvector` compatible array (OID 16400)

### Security:
The bridge operates in **read-only mode** by default, ensuring that external SQL clients cannot bypass compile-time SMT refinement checks.

