#!/usr/bin/env bash
set -euo pipefail

wasm="${1:?Usage: verify_cppbolt_wasm.sh <wasm-file>}"

if [ ! -s "$wasm" ]; then
    echo "WASM file is missing or empty: $wasm" >&2
    exit 1
fi

magic="$(od -An -tx1 -N4 "$wasm" | tr -d ' \n')"

if [ "$magic" != "0061736d" ]; then
    echo "Invalid WebAssembly binary: $wasm" >&2
    exit 1
fi

echo "Valid WebAssembly binary: $wasm ($(wc -c < "$wasm") bytes)"
