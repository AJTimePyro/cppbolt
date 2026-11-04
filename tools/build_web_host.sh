#!/usr/bin/env bash
# Generate web/host.js from the single TypeScript implementation in
# npm/src/host.ts. web/host.js is a build artifact and is not checked in.
#
# Usage: tools/build_web_host.sh

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
tsc="$root/node_modules/.bin/tsc"

if [ ! -x "$tsc" ]; then
    echo "TypeScript is not installed. Run \`npm ci\` in $root first." >&2
    exit 1
fi

"$tsc" -p "$root/npm/tsconfig.json"
node "$root/tools/bundle_web_host.js" \
    "$root/npm/dist/host.js" "$root/web/host.js"

echo "generated web/host.js from npm/src/host.ts"
