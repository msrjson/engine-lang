# Contributing to the Engine Lang Programming Language

Thank you for your interest in contributing to the **Engine Lang Programming Language** (`engine` / `.eng`) and its reference bootstrap compiler (`enginec`). Engine Lang is an infrastructure-agnostic, statically typed systems programming language designed to unify persistent memory (SLS), distributed causal graphs, and SMT-verified refinement types into a single compilation unit.

This document outlines the RFC (Request for Comments) process, compiler coding conventions, SMT formal verification guidelines, and core project governance.

---

## 1. Governance and Guiding Principles

Engine Lang follows an open-specification model maintained by the Core Working Group. All architectural decisions prioritize:
1. **Mathematical Soundness**: No feature enters the language without deterministic typing and formal verifiability.
2. **Infrastructure Elimination**: If a capability can be solved natively by the compiler and modern hardware primitives (NVRAM, CXL, SIMD, RDMA), we reject adding external middleware dependencies.
3. **Silicon-Direct Execution**: Language operators and directives must map directly to machine instructions (`clwb`, `sfence`, AVX-512, NEON) without runtime virtualization overhead.
4. **Deterministic LL(1) Parsing**: Operator ambiguity, dynamic operator creation, and unbounded backtracking are strictly prohibited.

---

## 2. The RFC Process

Substantial changes to the language grammar, compiler architecture, `.db` SLS layout, or causal mesh protocols must go through the Request for Comments (RFC) process.

### When to File an RFC
- Proposing new keywords, semantic directives (e.g., `@decorator`), or standard library traits.
- Modifying operator precedence, associativity, or SMT algebraic contracts (`ENG-SPEC-03.1`).
- Altering the physical layout of the `.db` container (`ENG-SLS-01.1`).
- Changing the lazy schema evolution protocol (LSE-CRC / `ENG-MIG-01`).

### RFC Lifecycle
1. **Drafting**: Copy `.github/ISSUE_TEMPLATE/rfc_proposal.md` and complete all sections.
2. **Submission**: Open a GitHub Issue or Pull Request with the tag `[RFC] <Title>`.
3. **Community Review**: A minimum discussion window of 14 days is required.
4. **Formal Verification Check**: Proposals involving operator traits or refinement types must include sample Z3 SMT translation rules.
5. **Consensus & Acceptance**: The Core Working Group votes on approval. Once merged into `docs/rfcs/`, the proposal is designated an official language target.

---

## 3. Formal SMT Proof Guidelines

Engine Lang enforces semantic domain invariants at compile time via an integrated SMT solver interface (Z3). When authoring standard library components or language extensions:

### 3.1. Proof Termination and Timeouts
- All refinement proofs (`where` predicates and `invariant { ... }` blocks) must terminate within **5000 milliseconds**.
- Non-linear arithmetic or unbounded quantifiers that cause solver divergence must be decomposed into linear lemmas or constrained refinement bounds.

### 3.2. Operator Law Verification
When overloading arithmetic or relational operators via traits (`Add`, `Sub`, `Mul`), you must specify the formal algebraic contract. For example:

```engine
pub trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;

    contract {
        // Required associativity proven via Z3 SMT: (a + b) + c == a + (b + c)
        forall(a: Self, b: Rhs, c: Rhs) {
            (a.add(b)).add(c) == a.add(b.add(c))
        }
    }
}
```

If the implementation allows side effects, non-associative edge cases, or division-by-zero panics, the compiler's `smt.rs` module must emit an unprovable counterexample and halt compilation.

---

## 4. Compiler (`compiler/`) Coding Standards

The reference bootstrap compiler (`enginec`) is written in standard, idiomatic Rust.

### 4.1. Rust Code Quality Rules
- **No Unsafe Code in Parser/AST**: `compiler/src/ast.rs`, `compiler/src/lexer.rs`, and `compiler/src/parser.rs` must not contain `unsafe` blocks.
- **Strict Byte Alignment in Memory Layer**: Structures in `compiler/src/memory.rs` that map directly to the `.db` file layout must explicitly use `#[repr(C, align(64))]` or `#[repr(C, align(16))]`.
- **Exhaustive Pattern Matching**: Compiler passes must avoid wildcard `_` catches when handling language tokens or AST expression variants.
- **Zero Panics in Compiler Passes**: Use `Result<T, CompilerError>` and propagate diagnostic errors gracefully with source line and column numbers.

### 4.2. Local Validation & Development Environments
Contributors can develop locally using any of three environments:
1. **Genesis Node Container (Zero-Setup)**: Run `docker compose up -d` and validate changes inside container `engine-genesis-node`.
2. **VS Code Devcontainer**: Open workspace in VS Code with the Remote Containers extension (configured via `.devcontainer/devcontainer.json`).
3. **Local Rust Toolchain**: Ensure `z3` and `libz3-dev` are installed locally.

Before submitting any pull request, ensure the complete test suite passes:

```bash
cd compiler
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo run -- check ../examples/01_hello_causal.eng
cargo run -- check ../examples/02_persistent_vehicle_fleet.eng
cargo run -- check ../examples/03_vector_ai_search.eng
```

---

## 5. Architectural Layers Guide

When contributing, ensure modifications strictly respect the architectural boundaries:
- **Layer 1 (Syntax & SMT)**: `compiler/src/lexer.rs`, `compiler/src/parser.rs`, `compiler/src/smt.rs`. Changes to tokens or grammar require corresponding updates to `docs/SPECIFICATION.md` and `docs/OPERATORS.md`.
- **Layer 2 (SLS Storage)**: `compiler/src/memory.rs`. Changes to data structures must maintain 64-byte alignment and CRC-64/NVME checksum validation as specified in `docs/ENGINE_DB_SLS_LAYOUT.md`.
- **Layer 3 (Mesh & Bridges)**: `compiler/src/mesh.rs`, `compiler/src/bridge.rs`, `compiler/src/web.rs`. Wire protocols must maintain backward-compatible framing as specified in `docs/DISTRIBUTED_MESH_WIRE.md`.
- **Layer 4 (Codegen & OS)**: `compiler/src/codegen.rs`, `compiler/src/unikernel.rs`. Emitted machine code and UEFI boot structures must comply with `docs/ENG_SILICON_CODEGEN_SPEC.md` and `docs/BARE_METAL_UNIKERNEL_SPEC.md`.
- **Layer 5 (Toolchain & MSR)**: `compiler/src/main.rs`, `compiler/src/tooling.rs`, `Dockerfile`, `docker-compose.yml`, `msr.json`.

---

## 6. Branching and Commit Policy

- **Main Branch**: `main` represents the stable specification and working reference compiler.
- **Feature Branches**: Use descriptive branch names: `feature/vector-simd`, `fix/parser-ll1-precedence`, `rfc/causal-crdt-wire`.
- **Commit Messages**: Follow Conventional Commits:
  - `feat(lexer): implement deterministic operator <=> tokenization`
  - `fix(memory): align journal entry commit_bit to cache-line flush`
  - `docs(spec): update EBNF grammar for boundary routes`
  - `spec(sls): define Region 2 ring buffer tail advance semantics`

---

## 7. Code of Conduct & Security Policy

All contributors are expected to uphold the [Code of Conduct](CODE_OF_CONDUCT.md).
For reporting security issues or vulnerabilities, please review our [Security Policy](SECURITY.md).

