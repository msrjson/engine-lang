# Continuum Memory Consistency Model & Concurrency Specification
**Document ID**: `CTM-MEM-01.1`  
**Status**: Official Normative Specification  
**Hardware Targets**: x86-64 (Total Store Order - TSO, CLWB, SFENCE), ARM64 (Weakly Ordered - Acquire/Release, DC CVAP, DSB)  
**Memory Subsystems**: NVRAM / CXL.mem / Direct Access (DAX)  

---

## 1. Overview and Core Philosophy

In traditional architectures, concurrency and persistence are managed by decoupled subsystems: the language memory model (e.g., C++11, Java Memory Model, Rust `std::sync`) governs volatile RAM, while database storage engines (WAL, redo/undo logs) manage disk durability.

Continuum unifies volatile and persistent concurrency into a single mathematical memory model:
1. **Unified Virtual Address Space**: Volatile stack frames and non-volatile heap entities reside in the same 64-bit flat address space.
2. **Causal Consistency with Sequential Transactional Equivalence**: Read operations observe a causally consistent prefix of global epochs. Atomic mutations within `atomic { ... }` blocks are isolated and commit with total store ordering across hardware cache-line flushes.
3. **Hardware Barrier Minimization**: Software memory barriers (`Acquire`, `Release`, `SeqCst`) are fused with hardware cache-line writebacks (`clwb`, `sfence`, `DC CVAP`) to eliminate redundant pipeline stalls.

---

## 2. Memory Access Taxonomy

Every memory access in Continuum falls into one of three physical tiers:

```
+--------------------------------------------------------------------------+
|                       CONTINUUM MEMORY ACCESS TIERS                       |
+-------------------+---------------------------+--------------------------+
| Tier 0: Volatile  | Tier 1: Read-Only Causal  | Tier 2: Persistent Atomic|
| Stack / Registers | Persistent Graph Nodes    | Shadow Allocations & Log |
| (CPU L1/L2 Cache) | (Zero-Copy DAX NVRAM)     | (CLWB + SFENCE Durable)  |
+-------------------+---------------------------+--------------------------+
```

### Tier 0: Volatile Thread-Local Memory
- Function parameters, local variables, and transient loop counters.
- Governed by standard register allocation and ephemeral stack frames.
- Reclaimed instantly upon stack unwinding; zero interaction with NVRAM.

### Tier 1: Read-Only Persistent Memory
- References to `@persistent entity` instances (`&T`).
- Reads occur directly in physical memory via static offset calculation (`lea [base + offset]`).
- Concurrent reads from millions of fibers require **zero lock acquisition** and cause **zero cache-line bouncing**, because persistent nodes committed in past epochs are immutable.

### Tier 2: Persistent Atomic Mutation
- Mutable references (`&mut T`) to `@persistent entity` fields.
- Strictly confined to `atomic { ... }` blocks.
- Uses **Shadow Paging** in Region 3 of the `.nvram` file: modifications write to a newly reserved shadow page, followed by a hardware commit in Region 2's journal.

---

## 3. The Causal Epoch Consistency Model

Continuum guarantees that all nodes in a cluster advance through discrete **Global Epochs** ($e \in \mathbb{N}$), coordinated by monotonically increasing 64-bit counters.

### Formal Ordering Definitions

1. **Happens-Before Relationship ($\prec_{hb}$)**:
   If event $A$ finishes before event $B$ starts in the same fiber, $A \prec_{hb} B$.
   If transaction $T_1$ commits in epoch $e_1$ and transaction $T_2$ reads a node mutated by $T_1$, then $e_1 < e_2$ and $T_1 \prec_{hb} T_2$.

2. **Causal Horizon**:
   A reader running in epoch $E_R$ observes state from all transactions committed at epoch $E \le E_R$. Any transaction in epoch $E > E_R$ is invisible to the reader, preventing torn reads and dirty reads without locking.

3. **Causal Graph Lineage ($A \to B$)**:
   The operator `A -> B` establishes an explicit causal graph edge. Entity $B$ immutably records the cryptographic hash and epoch ID of entity $A$ in its header.

---

## 4. Hardware Barrier Translation Rules

The `ctmc` compiler maps Continuum language constructs directly to CPU instructions:

| Continuum Language Construct | x86-64 Machine Code | ARM64 Machine Code | Hardware Effect |
|---|---|---|---|
| `let x = entity.field;` | `mov rax, [rbx + offset]` | `ldr x0, [x1, #offset]` | Zero-copy direct read from NVRAM. |
| `atomic { ... }` Enter | Register shadow page reservation. | Register shadow page reservation. | Lock-free CAS in Allocation Bitmap. |
| Entity field write | `mov [rdi + offset], rax` | `str x0, [x1, #offset]` | Write to private shadow page. |
| Shadow page flush | `clwb [rdi]` | `dc cvap, x1` | Flush cache line to memory controller. |
| Journal commit bit write | `mov byte ptr [rsi + 0x20], 1` | `strb w0, [x2, #0x20]` | Set `commit_bit = 0x01` in Region 2. |
| Transaction Commit Exit | `clwb [rsi + 0x20]`<br>`sfence` | `dc cvap, x2`<br>`dsb ish` | Fully serialize store to non-volatile domain. |
| Atomic Rollback (`?!`) | Free shadow page bit. | Free shadow page bit. | Atomic CAS bit reset; zero NVRAM flush. |

---

## 5. Concurrency Control: Lock-Free CAS & Exponential Backoff

When two concurrent fibers attempt to mutate the same persistent entity root pointer:

```
[ Fiber A: Prepare Shadow Page A' ]      [ Fiber B: Prepare Shadow Page B' ]
                │                                        │
                ▼                                        ▼
   [ Atomic CAS Root Pointer ]              [ Atomic CAS Root Pointer ]
         (Succeeds: A')                           (Fails: Collision)
                │                                        │
                ▼                                        ▼
      [ Commit Epoch A ]                   [ Adaptive Exponential Backoff ]
                                                         │
                                                         ▼
                                            [ Re-read A' as Base ]
                                                         │
                                                         ▼
                                            [ Retry Mutation as B'' ]
```

### Collision Resolution Algorithm
1. The colliding fiber detects CAS failure on the root table entry.
2. The fiber discards its uncommitted shadow page in Region 1's bitmap.
3. The runtime applies truncated binary exponential backoff with jitter:
   $$\text{Backoff}(attempt) = \min(2^{attempt} \times 10\,\mu\text{s} + \text{jitter}, 5\,\text{ms})$$
4. If collision persists beyond 32 retries, the transaction yields its fiber to prevent livelock.

---

## 6. Deadlock Prevention Guarantee

Continuum mathematically eliminates deadlocks through three structural compiler invariants:
1. **No Distributed Two-Phase Locking**: Continuum uses Multi-Version Causal Memory (MVCM). Readers never block writers; writers never block readers.
2. **Deterministic Resource Ordering**: In transactions that touch multiple entities, the compiler sorts targets by their physical 64-bit NVRAM offset before emitting CAS instructions.
3. **Bounded Atomic Execution**: Unbounded loops or blocking network I/O inside `atomic { ... }` are prohibited at compile time (`CTM-OP-014`).
