#!/usr/bin/env bash
# this_file: run-tests.sh
set -euo pipefail
cd "$(dirname "$0")"
cargo test --workspace "$@"
