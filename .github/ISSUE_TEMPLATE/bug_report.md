---
name: Bug Report
about: Report a defect in the Engine Lang specification, runtime, or enginec compiler
title: "[BUG] "
labels: ["bug", "triage"]
assignees: ""
---

### Summary
A clear and concise description of the bug.

### Environment & Architecture
- **OS & Kernel**: (e.g., Linux 6.8.0-40-generic x86_64)
- **Engine Lang Spec Version**: (e.g., ENG-SPEC-03.1)
- **Bootstrap Compiler Version (`enginec --version`)**:
- **Hardware / NVRAM Mode**: (e.g., Physical PMEM DAX, Emulated POSIX mmap, CXL.mem)
- **Z3 Version**: (e.g., Z3 4.13.0)

### Minimal Reproducible Code Example
Provide a minimal `.eng` snippet that triggers the issue:

```engine
module Test::Repro;

@persistent
entity BugDemo {
    val: Int where value > 0
}

action trigger() {
    // Paste code here
}
```

### Steps to Reproduce
1. Run `cargo run -- check repro.eng`
2. Observe error output...

### Expected Behavior
A clear explanation of what should have occurred according to the language specification.

### Actual Output & Diagnostics
```
[Paste compiler panic, SMT counterexample, or binary inspection error here]
```

### Additional Context
Add any other context about the problem (e.g., cache-line flush failures, torn write in journal, or SMT solver timeout).

