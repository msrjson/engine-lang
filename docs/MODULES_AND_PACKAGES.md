# Continuum Modules, Packages & Visibility Specification
**Document ID**: `CTM-PKG-01.1`
**Status**: Official Normative Specification
**Toolchain**: `ctm-pkg` / `ctmc pkg`
**Manifest Standard**: `Continuum.toml`

---

## 1. Overview and Package Philosophy

Continuum eliminates the dependency hell, bloated `node_modules` trees, and dynamic linking security vulnerabilities prevalent in legacy ecosystems.

The package and module system is built on three core tenets:
1. **Zero Dynamic Linker Dependencies**: Continuum binaries compile to hermetic static executables. Shared libraries (`.so`, `.dll`) are prohibited in the core persistence path.
2. **First-Class Architectural Visibility**: Visibility is not limited to simple public/private scoping; it natively accounts for multi-tenant isolation (`pub(tenant)`) and causal graph boundaries.
3. **Reproducible Causal Hash Verification**: Dependencies are addressed and locked by their immutable cryptographic content hash.

---

## 2. Module Hierarchy and Path Resolution

Continuum source files are organized into structured module trees:

```
src/
├── main.ctm                   # Root module entry point
├── domain/
│   ├── mod.ctm                # Submodule declaration: module Domain;
│   ├── fleet.ctm              # Submodule: module Domain::Fleet;
│   └── taxation.ctm           # Submodule: module Domain::Taxation;
└── gateway/
    └── api.ctm                # Submodule: module Gateway::Api;
```

### Module Declaration Syntax
```continuum
module Logistics::Fleet;

use Domain::Taxation::{TaxAssessment, TaxPaymentStatus};
use Core::Math::*;
```

### Path Resolution Rules
- `::` resolves absolute paths from the root package namespace.
- `super::` resolves one level up in the module hierarchy.
- `self::` refers to items within the current module.

---

## 3. Four-Tier Visibility Hierarchy

Continuum provides fine-grained encapsulation across multi-tenant persistent memory graphs:

| Visibility Modifier | Accessible Scope | Security & Persistence Guarantee |
|---|---|---|
| *(none / private)* | Current module only. | Internal helper actions and ephemeral variables. |
| `pub(module)` | Current module and its child submodules. | Package-internal implementation details. |
| `pub(tenant)` | Accessible only within the authenticated cryptographic tenant context. | Enforces data separation across multi-tenant organizations at compile time. |
| `pub` | Unrestricted visibility across all modules and importing packages. | Public API contracts, public types, and `@boundary` routes. |

### Example: Multi-Tenant Scoped Entity
```continuum
module Finance::Accounts;

@persistent
@TenantGuard(strict: true)
pub(tenant) entity BankAccount {
    pub id: UUIDv7,
    pub(tenant) balance: Decimal(18, 4),
    private_tax_id: Text // Private: never exposed outside Finance::Accounts
}
```

---

## 4. `Continuum.toml` Manifest Specification

Every Continuum package defines its dependencies, target memory limits, and hardware compilation parameters in `Continuum.toml`:

```toml
[package]
name = "enterprise-fleet"
version = "1.2.0"
authors = ["Dev Team <dev@company.com>"]
edition = "2026"
license = "Apache-2.0"

[dependencies]
# Cryptographically pinned remote causal package
"core-algebra" = { git = "https://github.com/continuum-lang/core-algebra.git", tag = "v0.5.0", hash = "sha256-8a9d...4f" }
"ai-embeddings" = { version = "0.3.1", registry = "https://pkg.continuum-lang.org" }

[compiler]
target_arch = "x86_64"
opt_level = 3
strict_invariants = true
smt_timeout_ms = 5000

[storage]
backend = "sls-nvram"
mount_path = "/dev/pmem0"
page_size_kib = 4
journal_size_mib = 10
sync_primitive = "clwb_sfence"

[network.mesh]
cluster_id = "018f6c44-3291-7f81-b552-87ad18f6c440"
node_id = 1
listen_addr = "0.0.0.0:9443"
```

---

## 5. Hermetic Dependency Resolution & Build Locking

When building a Continuum package with `ctmc pkg build`:
1. The dependency resolver constructs a deterministic dependency DAG.
2. Every downloaded source file is verified against the SHA-256 digest in `Continuum.lock`.
3. If any remote file diverges by a single bit, the build halts with `CTM-PKG-042 (Integrity Check Failure)`.
4. The compiler compiles dependencies into the static binary, eliminating external dynamic library lookups at runtime.

