# Purpose

`canon-rustc-v3` is a `RUSTC_WRAPPER` prototype.
It runs the real compiler and emits a deterministic semantic graph.
The graph records local Rust items, spans, calls, imports, and risk facts.
Downstream agents use those byte-anchored facts to plan safe refactors.
The project’s core value is compiler-backed evidence, not text scraping.
Current work is to harden validation, receipts, and native toolchain proof.
