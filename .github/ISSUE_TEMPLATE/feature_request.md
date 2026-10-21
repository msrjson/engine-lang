---
name: Feature Proposal / Enhancement
about: Propose an enhancement, new compiler directive, or toolchain improvement for Engine Lang
title: "[FEATURE] "
labels: ["enhancement", "needs-triage"]
assignees: ""
---

## Summary of Proposal
A clear and concise description of what feature or capability you propose to add to Engine Lang.

## Problem Being Solved
Is your feature request related to a specific limitation, architectural bottleneck, or ergonomic issue? Please describe.

## Proposed Syntax / API / Behavior
Provide concrete examples of how the new syntax (`.eng`) or CLI command (`enginec`) should behave:

```engine
// Example proposed .eng syntax or directive
```

## Formal Verification & SMT Invariants
How will this feature interact with formal verification and Z3 SMT solver guarantees?
- [ ] No unbounded recursion in refinement types
- [ ] Deterministic LL(1) grammar compliance
- [ ] Preserves Single-Level Store (SLS) crash-consistency

## Alternatives Considered
Describe any alternative solutions or workarounds you considered and why they were rejected.

## Additional Context
Add any other context, benchmarks, or references to relevant specifications (`ENG-SPEC-03.1`, `ENG-SLS-01.1`, etc.).

