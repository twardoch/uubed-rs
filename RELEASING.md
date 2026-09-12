---
this_file: RELEASING.md
---
# Releasing uubed-rs

```bash
./test.sh                    # Tests; failures stop the command
./build.sh                   # Local build, with no formatting/source rewrites
./publish.sh --dry-run       # Tests + next-version artifacts in a disposable Git copy
./publish.sh                 # Commit changes, tag next patch, build, push, publish
./publish.sh --bump minor    # Or --bump major
./publish.sh --resume vX.Y.Z # Retry an existing release at a clean tagged HEAD
```

`publish.sh` includes **all non-ignored working-tree changes** in the release
commit. Review `git diff` and `git status` first. It requires an attached branch,
an `origin` remote (or `--remote NAME`), Git author configuration, uv, and Python
3.12 (uv can provide it). It fetches tags and stops if the remote branch contains
changes absent locally; integrate those changes before retrying.

`gitnextver` supplies the next semantic version from `vMAJOR.MINOR.PATCH` tags.
The wrapper controls commits and pushes itself so tests, archive checks, and
remote-ref verification cannot be bypassed by gitnextver's push behaviour.
The default is a patch increment; minor/major increments reset lower components.

A dry run uses local tags, changes no source-repo commits/tags/remotes, and makes
no registry/site upload. Tests may install dependencies and regenerate ignored
build files. Preview artifacts live in `dist/preview/vX.Y.Z/`.
Release artifacts live in `dist/releases/vX.Y.Z/`; their manifest records the
commit, version and SHA-256 hashes. Uploads use only these verified files.

A failed push/upload may leave a local release tag. Use `--resume` with that tag:
artifacts are hash-checked and reused, and already-published identical files are
accepted. If code changes are required, create a new version instead of moving a
published tag. A failed test creates no tag. A changed working tree during tests
also stops the release.

`PUBLISH_SKIP_UPLOAD=1` skips package/site uploads **but still commits, tags and
pushes**. Use `--dry-run` for a preview without those side effects.

## Native distributions

Cargo requires a literal package version. The release wrapper writes the Git tag
version into `[workspace.package]` and local dependency requirements, updates
`Cargo.lock`, and commits them before tagging. All three crates and Maturin's
Python metadata therefore use one version. No manual manifest bump is needed.

Install stable Rust with rustfmt/clippy. `cargo login` (or `CARGO_REGISTRY_TOKEN`)
authenticates crates.io; `UV_PUBLISH_TOKEN` authenticates PyPI. Core and TM crates
are packaged, compiled from their archives, and published first. The PyO3 binding
crate is `publish = false`; its wheel and source distribution publish to PyPI as
`uubed-rs`. The local command builds a CPython 3.12 wheel for the host platform.
Other platforms/Python versions can build the source distribution with Rust;
`wheels.yml` also supplies CI wheel artifacts without uploading them to PyPI.


## Private files and standalone clones

Corpora, TMX/PDF/EPUB inputs, databases, model weights, credentials, local configs,
agent state and generated version modules are ignored. Build inputs are explicitly
listed; the release gate rejects private filenames inside actual archives.
Public tests create synthetic fixtures in temporary directories. Never force-add
private data. Ignoring/untracking does not remove files from historical commits.

Local dependency paths belong in editable developer installs, not package metadata.
A private `.local-sources.toml` may record workstation paths; it is reference data,
not an automatically loaded uv config. Release builds use `uv build --no-sources`.
The Python/TM tests use sibling source checkouts when present, otherwise released
packages. Release native Uubed, then Python Uubed, before either consumer.

CI runs tests/builds and retains artifacts. It does not tag or upload packages;
`publish.sh` is the sole publishing path. Each clone includes its own release
helpers. Their canonical copy is in `twardoch/uubed/scripts/`; maintainers sync
copies with `python scripts/sync_release_tools.py /path/to/uubed-project` there.

Versioning/build references: [Hatch VCS](https://github.com/ofek/hatch-vcs),
[Cargo workspace metadata](https://doc.rust-lang.org/cargo/reference/workspaces.html),
[uv publishing](https://docs.astral.sh/uv/guides/package/).

## Failed v1.0.12 attempt

The packaging fix changes tracked files, so rerun `./publish.sh` normally to
release v1.0.13. The failed v1.0.12 tag is local only; do not resume that tag with
modified manifests. Source-directory include patterns now select Rust files, and
the regression test injects ignored local-data files before asking Cargo for the
actual package list.

## Completing the partially published v1.0.13

The v1.0.13 Git tag, host wheel and both Cargo crates are already published.
PyPI rejected only the source archive, whose metadata declared a missing root
LICENSE. A separate corrected archive adds that file from the immutable tag;
all other members are unchanged. The original release directory and manifest
remain preserved. Do not move the tag or upload the wheel rebuilt for testing.

From the **uubed-project workspace root**, upload only the repaired source archive:

```bash
uv publish research/releases/v1.0.13-license-recovery/uubed_rs-1.0.13.tar.gz
```

After that succeeds, continue the remaining repositories:

```bash
./publish.sh --from uubed-py
```

The repair SHA-256 is
`c2d3e6e9856109d2a353c08cbd1f2fe239e1f40e002f0a9451377ccf05a94cb9`.
The ignored recovery directory contains audit.json with the original and repaired
hashes, tagged license provenance, and verified registry hashes. Nothing was
uploaded during repair. The native source fix remains uncommitted for the next
normal release, v1.0.14. `--resume v1.0.13` would reuse the rejected saved archive
and requires a clean tagged checkout, so it is not the recovery command here.

Future releases explicitly include the workspace license using
[Maturin's sdist include setting](https://www.maturin.rs/config.html).
The artifact gate checks the declared paths required by the
[Python source-distribution specification](https://packaging.python.org/en/latest/specifications/source-distribution-format/).
