#!/usr/bin/env bash
# this_file: build.sh
set -euo pipefail
cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")"
uvx maturin build --release --out "${1:-dist}" --interpreter python3.12
uvx maturin sdist --out "${1:-dist}"
