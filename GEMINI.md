# AGENTS for `uubed-rs` (Rust Code)

This repository houses the high-performance native Rust implementation of the `uubed` encoding algorithms.

## Role of this Repository:
- **Native Encoding Core:** Implements the core QuadB64 family of encoders in Rust for maximum performance.
- **Performance Optimization:** Focuses on SIMD vectorization, parallel processing, and memory efficiency within the Rust code.
- **FFI Interface:** Provides the C-compatible interface for Python bindings.
- **Rust-specific Testing & Benchmarking:** Develops and runs tests and benchmarks for the native Rust code.

## Key Agents and Their Focus:
- **Rust Developer:** Implements and optimizes the core encoding algorithms in Rust, ensuring high performance and memory safety.
- **Performance Engineer (Rust):** Specializes in low-level optimizations, including SIMD and multi-threading, for the Rust codebase.
- **FFI Specialist:** Designs and maintains the Foreign Function Interface for seamless integration with other languages, particularly Python.

If you work with Python, use 'uv pip' instead of 'pip', and use 'uvx hatch test' instead of 'python -m pytest'. 

When I say /report, you must: Read all `./TODO.md` and `./PLAN.md` files and analyze recent changes. Document all changes in `./CHANGELOG.md`. From `./TODO.md` and `./PLAN.md` remove things that are done. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics, while `./TODO.md` is its flat simplified itemized `- [ ]`-prefixed representation. When I say /work, you must work in iterations like so: Read all `./TODO.md` and `./PLAN.md` files and reflect. Work on the tasks. Think, contemplate, research, reflect, refine, revise. Be careful, curious, vigilant, energetic. Verify your changes. Think aloud. Consult, research, reflect. Then update `./PLAN.md` and `./TODO.md` with tasks that will lead to improving the work you’ve just done. Then '/report', and then iterate again.