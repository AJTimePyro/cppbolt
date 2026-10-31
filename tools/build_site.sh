#!/usr/bin/env bash
# Assembles the site of the playground: the web directory of the repository
# plus the WebAssembly build of the compiler.
#
# The workflow of .github/workflows/pages.yml deploys the result to GitHub
# Pages.  Usage: tools/build_site.sh [output-directory]

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:-$root/site}"
wasm="$root/target/wasm32-unknown-unknown/release/cppbolt.wasm"

if [ ! -f "$wasm" ]; then
    echo "the WebAssembly build of the compiler is missing; build it with:" >&2
    echo "  cargo build --release --target wasm32-unknown-unknown --lib" >&2
    exit 1
fi

rm -rf "$output"
mkdir -p "$output/pkg"
cp "$root/web/playground.html" "$output/index.html"
cp "$root/web/host.js" "$root/web/playground-worker.js" "$output/"
cp "$wasm" "$output/pkg/cppbolt_bg.wasm"
# GitHub Pages would otherwise run the files through Jekyll.
touch "$output/.nojekyll"

echo "the site is in $output"
