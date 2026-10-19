# Continuum Refinement Types & Formal SMT Invariants Specification
**Document ID**: `CTM-SMT-01.1`  
**Status**: Official Normative Specification  
**Formal Solver**: Z3 Theorem Prover Backend (SMT-LIB2 Standard)  
**Verification Target**: Compile-Time Invariant Proofs & Refinement Verification  

---

## 1. Overview and Theoretical Foundation

Continuum embeds formal methods directly into the programming language, eliminating runtime invariant checks, manual defensive programming, and runtime `assert()` crashes in production.

By coupling the static type checker with the **Z3 SMT solver**, types in Continuum represent not merely memory layouts, but **mathematical propositions** (Curry-Howard-Lambek correspondence applied to persistent systems):

$$\tau = \{ x : B \mid \phi(x) \}$$

Where $B$ is a base primitive type (`Int`, `Float`, `Decimal`, `Text`) and $\phi(x)$ is a decidable first-order logic predicate.

---

## 2. Refinement Type Syntax & Semantics

Refinement predicates are attached using the `where` keyword:

```continuum
// Scaled monetary type preventing negative balances
type PositiveMoney = Decimal(18, 4) where value > 0.0000;

// Constrained string enforcing exact length and alphanumeric character sets
type BrazilianCPF = Text where len(value) == 11 && is_numeric(value);

// Bounded integer for geographic coordinates
type Longitude = Float where value >= -180.0 && value <= 180.0;
```

### Compile-Time Verification Condition (VC) Generation
When assigning expression $E$ to a variable of refined type $\{ x : B \mid \phi(x) \}$:
1. The compiler tracks path conditions $\Gamma$ (guards from surrounding `if` blocks, invariants of input parameters).
2. The compiler emits the theorem:
   $$\Gamma \implies \phi(E)$$
3. The formula $\Gamma \land \neg \phi(E)$ is submitted to the Z3 SMT solver.
4. If the formula is **UNSAT**, the assignment is proven sound.
5. If the formula is **SAT**, the solver generates a concrete counterexample, and `ctmc` halts with an error.

---

## 3. Persistent Entity Invariants

Entities declare structural domain contracts using the `invariant { ... }` block:

```continuum
@persistent
entity TreasuryAccount {
    id: UUIDv7,
    balance: Decimal(18, 4),
    credit_limit: Decimal(18, 4),
    reserved: Decimal(18, 4),

    // Invariant: Balance cannot drop below negative credit limit,
    // and reserved funds cannot exceed active balance.
    invariant {
        self.balance >= -self.credit_limit &&
        self.reserved >= 0.0000 &&
        self.reserved <= (self.balance + self.credit_limit)
    }
}
```

Whenever an action mutates a `TreasuryAccount` inside an `atomic` block, the compiler proves that the post-state satisfies the invariant.

---

## 4. Permitted Predicate Grammar in Invariants

To guarantee that SMT solver verification remains **decidable** and executes in $<5000\text{ ms}$, expressions within `where` and `invariant` blocks are strictly restricted to:

| Category | Permitted Operators & Functions | Prohibited Constructs |
|---|---|---|
| **Arithmetic** | Linear arithmetic: `+`, `-`, scalar `*`, `/` with constant divisor. | Non-linear polynomial products (`x * y` where both are variables). |
| **Relational** | `<`, `<=`, `>`, `>=`, `==`, `!=` | Pointer arithmetic or unaligned offset comparisons. |
| **Logical** | `&&`, `\|\|`, `!` | Side effects, assignments, or network calls. |
| **Pure Functions** | `len(str)`, `is_alphanumeric(str)`, `is_numeric(str)` | Any non-deterministic function (`rand()`, `now()`). |

---

## 5. Solver Timeouts, `assume`, and `unsafe_invariant`

When reasoning about complex domains that exceed linear quantifier-free arithmetic:

### 5.1. Solver Timeout Policy
- The default Z3 solver timeout is configured in `Continuum.toml` (`smt_timeout_ms = 5000`).
- If Z3 does not reach a decision within the timeout, the compiler rejects the build with `CTM-SMT-091 (Solver Timeout / Incomplete Proof)`.

### 5.2. Explicit Axiomatic Assumptions (`assume`)
For mathematical properties proven outside the language (e.g., cryptographic hash pre-image resistance or external hardware properties), developers may introduce explicit lemmas:

```continuum
action verify_hash(data: Text, expected_hash: Text) {
    let computed = crypto::sha256(data);
    
    // Explicit assumption documented for compiler proof context
    assume(len(computed) == 64);
    
    let is_valid: Bool = computed == expected_hash;
    return is_valid;
}
```

### 5.3. Unsafe Escape Hatch (`unsafe_invariant`)
In performance-critical edge kernels where an invariant cannot be expressed in first-order logic, `unsafe_invariant` may be used:
```continuum
unsafe_invariant {
    // Relies on developer manual proof audit; compiler logs audit warning
    custom_c_math_valid(self.raw_ptr)
}
```
Use of `unsafe_invariant` generates a mandatory security warning in CI builds.
