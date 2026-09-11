---
this_file: WORK.md
---

## 2026-09-11 — Shared native workspace and translation memory

Consolidated native libraries into uubed-core, uubed-tm and uubed-python. Added
portable SQLite exact/pair storage, signed int8 search, streaming English TMX
import, atomic builds, model-space checks and `uubed tm` commands. Both book
alignment and Abersetz consume the shared Python model runtime. Removed duplicate
release runners, placeholder native tests and generated site output. Built-module
imports now use the actual uubed_native extension.

Verification is recorded in ../../research/consolidation: Rust/SIMD, built native
wheel, Python and consumer suites, real TMX retrieval, copied-book alignment,
recording-LLM integration and C API execution. No packages were published.

## Release and privacy cleanup

Implemented standalone tag-derived release commands, source/archive privacy gates, fresh artifact manifests, dry runs and same-tag retries. Verification is recorded in the workspace release report. Existing local data remains on disk.

### Release verification — 2026-09-12

148 Rust tests passed, 1 timing test ignored; 3 native Python tests passed; format/clippy passed; both v1.0.12 crates compiled from packaged archives; Maturin wheel/sdist verified. Shared release-flow suite: 11 passed. Source HEAD and latest tag remained unchanged. No live publication was performed.
