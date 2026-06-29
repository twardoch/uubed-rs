#!/usr/bin/env bash
# install.sh — Install uubed-rs Python bindings via maturin
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Installing Python bindings (maturin develop)..."
uvx maturin develop --release

echo "==> Install complete."
