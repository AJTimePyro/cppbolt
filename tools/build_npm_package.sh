#!/usr/bin/env bash
# Assemble the raw-WASM npm package.
#
# Usage: tools/build_npm_package.sh [output-directory] [version]

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:-$root/dist/npm}"
version="${2:-}"
wasm="$root/target/wasm32-unknown-unknown/release/cppbolt.wasm"

if [ ! -s "$wasm" ]; then
    echo "WebAssembly compiler build is missing. Run:" >&2
    echo "  cargo build --release --target wasm32-unknown-unknown --lib" >&2
    exit 1
fi

rm -rf "$output"
mkdir -p "$output/wasm"

cp "$root/npm/package.json" "$output/package.json"
cp "$root/npm/README.md" "$output/README.md"
cp "$wasm" "$output/wasm/cppbolt.wasm"

if [ -n "$version" ]; then
    (cd "$output" && npm pkg set "version=$version")
fi

echo "Staged npm package at: $output"
