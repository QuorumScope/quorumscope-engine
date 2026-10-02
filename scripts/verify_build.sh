#!/bin/bash
set -e

echo "Running final sanity check..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
echo "All checks passed!"
