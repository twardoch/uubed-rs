#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Building uubed-rs (Rust core library)..."

# Build the Rust library in release mode from the current directory
cargo build --release

echo "uubed-rs (Rust core library) build complete. Artifacts are in target/release/."

# If this project also builds Python wheels via maturin, you might add:
# echo "Building Python wheels for uubed-rs..."
# maturin build --release --out dist
# echo "Python wheels for uubed-rs built in dist/."