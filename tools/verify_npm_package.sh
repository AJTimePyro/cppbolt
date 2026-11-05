#!/usr/bin/env bash
set -euo pipefail

package_dir="${1:?Usage: verify_npm_package.sh <package-directory>}"

test -f "$package_dir/package.json"
test -f "$package_dir/README.md"
test -f "$package_dir/dist/index.js"
test -f "$package_dir/dist/index.d.ts"
test -f "$package_dir/dist/index.browser.mjs"
test -f "$package_dir/dist/cppbolt.browser.js"

"$(dirname "$0")/verify_cppbolt_wasm.sh" \
    "$package_dir/wasm/cppbolt.wasm"

echo "Checking npm package contents..."
(cd "$package_dir" && npm pack --dry-run)

echo "npm package verification passed"
