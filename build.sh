#!/usr/bin/env bash
# build.sh — Build uubed-rs (Rust + Python/maturin bindings)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Formatting Rust code..."
cargo fmt --all

echo "==> Linting Rust code..."
cargo clippy --all-targets -- -D warnings

echo "==> Testing Rust code..."
cargo test --workspace

echo "==> Building Rust release..."
cargo build --release --workspace

echo "==> Building Python wheel via maturin..."
uvx maturin build --release

echo "==> Build complete."
