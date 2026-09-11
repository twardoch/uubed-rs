#!/usr/bin/env bash
# this_file: test.sh
set -euo pipefail
cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")"
cargo fmt --all --check
cargo clippy --workspace --all-targets --features simd -- -D warnings
cargo test --workspace --features simd
wheel_dir=$(mktemp -d)
trap 'rm -rf "$wheel_dir"' EXIT
uvx maturin build --release --out "$wheel_dir" --interpreter python3.12
uv run --no-project --python 3.12 --with pytest --with numpy --with "$wheel_dir"/*.whl python -m pytest tests
