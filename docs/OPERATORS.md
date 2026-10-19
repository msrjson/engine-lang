# Official Operator Specification: Continuum
**Document ID**: `CTM-SPEC-02.1`
**Status**: Official Normative Specification
**Compiler Interface**: LL(1) Deterministic Operator Precedence Matrix
**Formal Verification**: Integrated Z3 SMT Algebraic Law Enforcement

---

## 1. Architectural Design Principles

The Continuum operator matrix is governed by four strict design laws:

1. **Deterministic LL(1) Parsing**: The grammar eliminates all backtracking, arbitrary lookahead, and contextual symbol ambiguity. Symbols have a single semantic meaning; the asterisk `*` denotes multiplication, while pointer dereference is syntactically distinct.
2. **Silicon-Direct Translation**: SIMD vector math, tensor operations, and persistence primitives compile directly to machine instructions (`clwb`, `sfence`, AVX-512, ARM NEON/SVE, FMA).
3. **Formal Algebraic Laws via SMT**: Operator overloading on custom types is permitted **only** when the implementation satisfies formal algebraic laws (e.g., associativity, commutativity, identity) proven by the Z3 SMT solver during `ctmc check`.
4. **Unit-Return Mutations**: Assignment operators (`=`, `+=`, etc.) return the unit type `()`, preventing accidental assignment in conditional statements (`if x = y`).

---

## 2. The 14-Level Precedence and Associativity Table

The table below defines precedence from Level 1 (highest) to Level 14 (lowest).

| Level | Operators | Associativity | Syntax Description | Hardware & Runtime Translation |
|---|---|---|---|---|
| **1** | `.`, `?.`, `::` | Left-to-Right | Field access, safe null navigation, module resolution. | Compile-time offset resolution (`lea [base + offset]`); zero-copy direct pointer navigation. |
| **2** | `()`, `[]`, `{}` | Left-to-Right | Action call, indexed contiguous access, literal constructor. | Machine `call`, scaled base indexing (`lea [base + idx * size]`), stack frame allocation. |
| **3** | `?`, `?!` | Postfix | Error propagation, atomic transactional collapse. | `?` unwinds fiber stack; `?!` triggers instant rollback of shadow pages in NVRAM. |
| **4** | `-`, `!`, `~`, `&`, `*` | Right-to-Left (Unary) | Arithmetic negation, logical NOT, bitwise NOT, causal ref, dereference. | CPU register ops (`neg`, `not`); reference boundary validation against causal epoch horizon. |
| **5** | `**` | Right-to-Left | Arithmetic exponentiation. | Floating-point square-and-multiply routine / hardware transcendentals. |
| **6** | `*`, `/`, `%`, `@*` | Left-to-Right | Multiplication, division, modulo, tensor/matrix product. | Scalar `imul`/`idiv`; `@*` emits AVX-512/NEON packed vector matrix multiply instructions. |
| **7** | `+`, `-`, `@+`, `@-` | Left-to-Right | Scalar addition/subtraction, SIMD vector addition/subtraction. | Scalar `add`/`sub`; `@+` and `@-` emit packed vector instructions across vector lanes in parallel. |
| **8** | `<<`, `>>` | Left-to-Right | Bitwise left and right shift. | Machine `shl`/`shr` (arithmetic or logical based on operand signedness). |
| **9** | `<=>` | Left-to-Right | High-dimensional cosine metric, normalized dot product. | SIMD Fused Multiply-Add (FMA) kernel calculating cosine similarity in hardware registers. |
| **10** | `<`, `<=`, `>`, `>=`, `==`, `!=` | Non-chaining | Order comparisons, structural equivalence, identity. | Machine `cmp`/`test` setting condition flags. Prohibits chaining (`a < b < c` is invalid syntax). |
| **11** | `&`, `^`, `\|` | Left-to-Right | Bitwise AND, XOR, OR. | Machine word logical instructions (`and`, `xor`, `or`). |
| **12** | `&&`, `\|\|` | Left-to-Right | Short-circuit logical conjunction and disjunction. | Branch probability-weighted conditional jumps (`je`, `jne`). |
| **13** | `\|>`, `<\|`, `~>`, `->` | Left-to-Right | Functional pipeline, backpipe, async fiber pipe, causal graph edge. | Fiber continuation dispatch, zero-lock DAG insertion in persistent causal graph. |
| **14** | `=`, `+=`, `-=`, `*=`, `/=` | Right-to-Left | Value assignment, compound mutations. | Memory write. In `@persistent entity`, requires `atomic` block; emits `clwb` + `sfence`. Returns `()`. |

---

## 3. In-Depth Operator Semantics

### 3.1. High-Dimensional Vector & SIMD Operators (`@*`, `@+`, `@-`, `<=>`)
Continuum provides first-class support for `Vector(Dimension)` primitives:

- **`v1 @+ v2` / `v1 @- v2`**: Element-wise addition and subtraction of vectors and tensors. Compiles to AVX-512 `_mm512_add_ps` or ARM SVE instructions.
- **`m @* v`**: Matrix-vector product executed across SIMD vector lanes without memory allocation.
- **`v1 <=> v2` (Cosine Similarity Operator)**:
  Computes the normalized cosine similarity metric:
  $$\text{sim}(u, v) = \frac{u \cdot v}{\|u\|_2 \, \|v\|_2}$$
  In hardware, this compiles to an unrolled FMA (Fused Multiply-Add) SIMD routine. It is used directly within persistent vector queries and HNSW index searches:

```continuum
let query_embedding: Vector(1536) = ai::embed("heavy duty electric vehicle");
let score: Decimal(5, 4) = vehicle.embedding <=> query_embedding;

if score >= 0.8200 {
    log::info("Semantic match confirmed");
}
```

### 3.2. Causal Graph & Pipeline Operators (`->`, `~>`, `|>`)
- **`->` (Causal Graph Edge)**:
  Establishes a directed dependency in the persistent causal graph between two entities. When entity $A \to B$ is recorded, $B$ immutably retains the causal cryptographic commit hash of $A$.
  ```continuum
  purchase_order -> tax_invoice -> shipping_receipt;
  ```
- **`~>` (Asynchronous Fiber Dispatch)**:
  Connects computation to an asynchronous worker fiber without blocking the current OS thread:
  ```continuum
  request ~> validate_payload() ~> persist_audit_log();
  ```
- **`|>` (Functional Pipeline Composition)**:
  Passes the left-hand expression as the first argument of the right-hand function:
  ```continuum
  let final_price = raw_amount
      |> apply_corporate_discount
      |> calculate_municipal_tax
      |> round_currency;
  ```

### 3.3. Failure & Atomic Collapse Operators (`?`, `?!`)
- **`?` (Standard Scope Error Propagation)**:
  Unwraps `Ok(v)` or `Some(v)`. On `Err(e)` or `None`, terminates the current function and returns the error to the caller, unwinding local volatile stack frames.
- **`?!` (Atomic Transactional Collapse)**:
  Exclusively valid inside `atomic { ... }` blocks. When evaluating an `Err(e)`, `?!` not only unwinds the stack, but **aborts the active hardware transaction**, immediately discarding the allocated shadow pages in NVRAM and reverting mutated entities back to the prior causal epoch.

```continuum
atomic {
    source_wallet.balance -= transfer_amount;

    // If validation fails, atomic rollback occurs on source_wallet in NVRAM:
    verify_compliance(source_wallet, target_wallet)?!;

    target_wallet.balance += transfer_amount;
}
```

---

## 4. Formal SMT Verification of Operator Overloading

When implementing operator traits on custom structures, Continuum requires formal algebraic contracts within a `contract { ... }` block. During `ctmc check`, the compiler converts these methods into first-order logic formulas verified by Z3.

### 4.1. Formal `Add` Trait Contract Definition
```continuum
module Core::Algebra;

pub trait Add<Rhs = Self> {
    type Output;

    fn add(self, rhs: Rhs) -> Self::Output;

    contract {
        // Enforce strict associativity via SMT: (a + b) + c == a + (b + c)
        forall(a: Self, b: Rhs, c: Rhs) {
            (a.add(b)).add(c) == a.add(b.add(c))
        }
    }
}
```

### 4.2. Example: Refined Monetary Balance Implementation
```continuum
module Domain::Treasury;

use Core::Algebra::Add;

type Currency = Decimal(18, 4) where value >= 0.0000;

entity TreasuryLedger {
    balance: Currency,
    reserve: Currency
}

impl Add for TreasuryLedger {
    type Output = TreasuryLedger;

    fn add(self, rhs: TreasuryLedger) -> TreasuryLedger {
        return TreasuryLedger {
            balance: self.balance + rhs.balance,
            reserve: self.reserve + rhs.reserve
        };
    }
}
```

If an author attempts an implementation that violates associativity, introduces side effects, or panics on overflow, `ctmc check` refuses compilation and outputs the counterexample.

---

## 5. Static Error Diagnostics Matrix

| Error Code | Semantic Description | Mandatory Correction |
|---|---|---|
| `CTM-OP-001` | Chained relational operators (e.g., `a < b < c`). | Separate using explicit logical conjunction: `a < b && b < c`. |
| `CTM-OP-014` | Mutation of `@persistent entity` field outside `atomic` block. | Confine all mutations inside an `atomic { ... }` block. |
| `CTM-OP-032` | Use of atomic collapse operator `?!` outside `atomic` context. | Use `?` for standard return propagation or enclose within `atomic`. |
| `CTM-OP-078` | SMT proof failure of formal algebraic contract during operator overload. | Eliminate non-associativity, unhandled edge cases, or side effects. |
| `CTM-OP-099` | Attempt to define a custom or uncataloged operator symbol. | Use named functions; the Continuum operator set is closed. |

