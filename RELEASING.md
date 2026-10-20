# Engine Lang Release Engineering Handbook

**Document ID**: `ENG-REL-01.1`
**Governance**: The Vance Protocol Group / Engine Lang Core Working Group
**Status**: Canonical Production Release Standard
**License**: Apache License 2.0

---

## 1. Overview & Release Philosophy

Engine Lang adheres to rigorous aerospace- and systems-grade engineering release principles. Because Engine Lang functions as a compiler, formal verification engine, and direct hardware runtime (Single-Level Store), releases must guarantee mathematical soundness, crash durability, and backward binary compatibility.

Every release published to GitHub or container registries must pass all **Production Readiness Gates** defined in [`BACKLOG.md`](BACKLOG.md).

---

## 2. Versioning Specification (SemVer 2.0.0)

Engine Lang follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) in the format:

$$\text{v}\langle\text{MAJOR}\rangle.\langle\text{MINOR}\rangle.\langle\text{PATCH}\rangle[-\langle\text{PRERELEASE}\rangle]$$

### 2.1 Component Version Alignment Matrix

Every official release synchronizes three foundational specifications:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       ENGINE LANG VERSION ARCHITECTURE                      │
├──────────────────────────┬──────────────────────┬───────────────────────────┤
│ Component                │ Current Specification│ SemVer Impact             │
├──────────────────────────┼──────────────────────┼───────────────────────────┤
│ Bootstrap Compiler       │ enginec v0.3.0       │ Minor (Feature additions) │
│ Language Grammar         │ ENG-SPEC-03.1        │ Major (Syntax breaks)     │
│ Storage Layout (SLS)     │ ENG-SLS-01.1         │ Major (Superblock format) │
│ Causal Mesh Protocol     │ ENG-WIRE-01.1        │ Minor (Wire compatibility)│
│ Bare-Metal Specification │ ENG-UNIKERNEL-01.1   │ Experimental              │
└──────────────────────────┴──────────────────────┴───────────────────────────┘
```

- **MAJOR**: Incremented when backward-incompatible grammar breaks occur or when the physical binary layout of the Single-Level Store (`engine.db` Superblock magic or alignment) changes without automated lazy migration.
- **MINOR**: Incremented when new language capabilities (e.g., `@datasource` polyglot connectors, new SMT refinement solvers, or LLVM backends) are added in a backward-compatible manner.
- **PATCH**: Incremented for backward-compatible bugfixes, SMT solver performance tuning, parser diagnostic enhancements, or CLI ergonomic improvements.

---

## 3. Release Lifecycle & Channel Progression

Engine Lang maintains four release channels:

```
Trunk (main) ──> Alpha (genesis) ──> Beta (feature freeze) ──> Release Candidate (RC) ──> General Availability (GA)
```

1. **Nightly / Trunk (`main`)**: Continuously tested via GitHub Actions CI. May contain experimental syntax or unstable features.
2. **Alpha (`vX.Y.Z-alpha.N` / `v0.3.0-genesis`)**: Feature-complete preview of a major/minor milestone; formal SMT prover and storage format stabilized.
3. **Beta (`vX.Y.Z-beta.N`)**: API and syntax frozen. Testing focused on compiler edge cases, fuzzing, and external SQL client compatibility (DBeaver, psql).
4. **Release Candidate (`vX.Y.Z-rc.N`)**: Production candidate undergoing final Gate 0–3 sign-off.
5. **General Availability (`vX.Y.Z`)**: Public production release tagged on GitHub and published to registries.

---

## 4. Pre-Release Quality Checklist (Release Candidate Sign-Off)

Before tagging any release, the release engineer must verify each item:

- [ ] **1. Formal SMT Verification Suite (Gate 0)**:
  - All canonical examples in `examples/` verified with Z3 SMT:
    ```bash
    for f in examples/*.eng; do enginec check "$f" || exit 1; done
    ```
- [ ] **2. Single-Level Store Durability & CRC64 Integrity (Gate 1)**:
  - Superblock validated with zero corruption flags:
    ```bash
    enginec inspect data/engine.db
    ```
- [ ] **3. Mandatory Container Validation (Rule 0.1)**:
  - Tests verified inside Docker container `engine-genesis-node`:
    ```bash
    docker exec engine-genesis-node sh -lc 'for f in /app/examples/*.eng; do enginec check "$f" || exit 1; done'
    docker exec engine-genesis-node sh -lc 'enginec inspect /app/data/engine.db'
    ```
- [ ] **4. Code Formatting & Static Analysis**:
  - Zero warnings permitted:
    ```bash
    cd compiler && cargo fmt --check && cargo clippy -- -D warnings
    ```
- [ ] **5. Interoperability & SQL Wire Protocols (Gate 2)**:
  - Verify PostgreSQL wire protocol responds to queries:
    ```bash
    enginec bridge --test-mode
    ```
- [ ] **6. Version Bump Synchronization**:
  - `compiler/Cargo.toml`: `version = "X.Y.Z"`
  - `Engine.toml`: `version = "X.Y.Z"`
  - `compiler/src/main.rs`: `#[command(version = "X.Y.Z")]`
- [ ] **7. Changelog Finalization**:
  - Move changes from `[Unreleased]` into `[X.Y.Z] - YYYY-MM-DD` in `CHANGELOG.md`.

---

## 5. Automated Build & Distribution Pipeline

When a semantic tag (e.g., `v0.3.0`) is pushed to the repository, GitHub Actions (`.github/workflows/release.yml`) triggers the automated release pipeline:

### 5.1 Artifact Cross-Compilation Matrix

| Platform Target | Binary Name | Archive Format |
|---|---|---|
| Linux x86_64 (glibc / DAX) | `enginec` | `enginec-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64 (ARM64) | `enginec` | `enginec-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 (Intel) | `enginec` | `enginec-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS aarch64 (Apple Silicon) | `enginec` | `enginec-vX.Y.Z-aarch64-apple-darwin.tar.gz` |

### 5.2 Cryptographic Provenance & Checksums

Every release archive is hashed using SHA-256 and recorded into a canonical `SHA256SUMS` file:

```bash
sha256sum enginec-v*.tar.gz > SHA256SUMS
```

Developers can verify downloaded release archives:

```bash
sha256sum -c SHA256SUMS --ignore-missing
```

### 5.3 Container Image Tagging

Official container images are published to GitHub Packages (`ghcr.io`):
- `ghcr.io/engine-lang/enginec:latest`
- `ghcr.io/engine-lang/enginec:vX.Y.Z`
- `ghcr.io/engine-lang/enginec:vX.Y`

---

## 6. Emergency Patch & Hotfix Policy (P0 Defects)

In the event of a P0 defect (SLS data corruption, memory safety breach, or SMT solver unsoundness):
1. A dedicated hotfix branch `hotfix/vX.Y.Z+1` is cut directly from the affected release tag.
2. The fix must include a minimal reproducible test case in `tests/` demonstrating both the bug and its resolution.
3. The patch release increments only the `PATCH` digit (e.g., `v0.3.1`).
4. An immediate cryptographic security advisory is published detailing the affected invariant and mitigation steps.

