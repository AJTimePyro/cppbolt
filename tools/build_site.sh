#!/usr/bin/env bash
# Assembles the site of the playground.
#
# The site is the web directory of the repository plus the `pkg` directory
# that holds the WebAssembly build of the compiler.  The workflow of
# .github/workflows/pages.yml deploys the result to GitHub Pages.
#
# Usage: tools/build_site.sh [output-directory]

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:-$root/site}"
wasm="$root/target/wasm32-unknown-unknown/release/cppwasm.wasm"

if [ ! -f "$wasm" ]; then
    echo "the WebAssembly build of the compiler is missing." >&2
    echo "build it with:" >&2
    echo "  cargo build --release --target wasm32-unknown-unknown --lib" >&2
    exit 1
fi

rm -rf "$output"
mkdir -p "$output/pkg"

# The page is the index of the site, so the site is served at its root.
cp "$root/web/playground.html" "$output/index.html"
cp "$root/web/host.js" "$output/host.js"
cp "$root/web/playground-worker.js" "$output/playground-worker.js"

# The `pkg` directory holds the glue and the module of the compiler.
cp "$root/web/pkg/cppwasm.js" "$output/pkg/cppwasm.js"
cp "$root/web/pkg/cppwasm.d.ts" "$output/pkg/cppwasm.d.ts"
cp "$root/web/pkg/package.json" "$output/pkg/package.json"
cp "$wasm" "$output/pkg/cppwasm_bg.wasm"

# GitHub Pages would otherwise run the files through Jekyll.
touch "$output/.nojekyll"

echo "the site is in $output"
