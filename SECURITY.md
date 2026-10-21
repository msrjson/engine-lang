# Security Policy

## Supported Versions

The following versions of **Engine Lang** and the reference compiler (`enginec`) currently receive active security and verification updates:

| Version | Status |
| ------- | ------ |
| `v0.3.1-silicon` | :white_check_mark: Actively Supported |
| `< v0.3.0` | :x: Deprecated |

---

## Reporting a Vulnerability

The Engine Lang project is committed to mathematical soundness, memory safety, and crash-consistent data persistence.

If you discover a security vulnerability, soundess flaw in the SMT proof emitter, or potential memory safety violation:

1. **Do Not File a Public Issue**: Please do **not** disclose the issue publicly until it has been reviewed and patched.
2. **Contact**: Open a confidential security advisory directly on GitHub via [Security Advisories](https://github.com/msrjson/engine-lang/security/advisories) or notify the core maintainers through the repository issue tracker marked with the `[SECURITY]` prefix.
3. **Information to Include**:
   - Compiler version (`enginec --version`).
   - Minimal reproduction code (`.eng` source file).
   - Expected behavior vs. observed behavior.
   - Any crash dumps, SMT timeout traces, or Single-Level Store `.db` inspection headers.

### Response Timeline
- **Initial Acknowledgement**: Within 48 hours.
- **Triage & Reproducibility**: Within 5 business days.
- **Patch Release & Advisory Publication**: Coordinated with the reporter following verification on the genesis container.

---

## Secrets and Key Management
This repository strictly prohibits committing private keys, cloud access tokens, or sensitive environment secrets. Automated secret scanners run against all pull requests and branch pushes.
