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

## 2026-09-12 — Cargo release packaging regression

The live v1.0.12 attempt exposed two gaps: Cargo's broad include globs admitted an ignored examples/.DS_Store, and cargo metadata --no-deps did not update workspace versions in Cargo.lock before tagging. Narrowed all three crate source/test/benchmark/example includes to Rust files; version synchronization now resolves full metadata before committing, and crate packaging requires --locked. The archive rejection guard remains in place.

Regression checks reproduced both failures before the fix. After the fix, all 12 shared release tests and the three-crate packaging canary test passed. Built the native wheel, source distribution, and both Cargo archives directly from the real working tree with the original .DS_Store present; archive checks and compilation of both packaged crates passed. No runtime Rust code changed.

The failed v1.0.12 tag remains local and unchanged; the remote has no such tag. No live push/upload was performed during repair. Because the fix changes source, rerun the normal ./publish.sh command to create the next patch version (v1.0.13); --resume v1.0.12 is inappropriate for this modified checkout. Verification artifacts are outside publish destinations in ignored research/releases/packaging-fix-h0cmw4d_/artifacts.
