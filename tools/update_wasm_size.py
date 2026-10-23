#!/usr/bin/env python3
"""Updates the compiler wasm size in docs/architecture.md.

The script builds the release compiler for `wasm32-unknown-unknown`, measures
the module and rewrites the paragraph in `docs/architecture.md`. The current
size is measured; the growth is computed against the pinned baseline below.

Usage: python3 tools/update_wasm_size.py [--check]

With `--check` the script only reports the measured size and whether the
document is up to date; it does not write anything.
"""
import argparse
import gzip
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WASM = os.path.join(ROOT, "target/wasm32-unknown-unknown/release/cppwasm.wasm")
DOC = os.path.join(ROOT, "docs/architecture.md")

# The size baseline the growth is measured against.
BASELINE_BYTES = 772391
BASELINE_COMMIT = "45153ff"

# The paragraph that carries the measured size.
PARAGRAPH = """The release compiler module built for `wasm32-unknown-unknown` is currently
{raw:,} bytes raw and {gzip:,} bytes with `gzip -9 -n`.  The module has grown {growth:+.1f} %
from {baseline:,} bytes at commit `{baseline_commit}`.
"""

# The paragraph that the script replaces.
PATTERN = re.compile(
    r"The release compiler module built for `wasm32-unknown-unknown` is currently\s.*?"
    r"(?=The core does not spawn processes)",
    re.S,
)


def build():
    subprocess.run(
        ["cargo", "build", "--release", "--target", "wasm32-unknown-unknown"],
        cwd=ROOT, check=True,
    )


def measure():
    with open(WASM, "rb") as fh:
        data = fh.read()
    raw = len(data)
    compressed = len(gzip.compress(data, 9, mtime=0))
    return raw, compressed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    build()
    raw, compressed = measure()
    print(f"compiler wasm: {raw:,} bytes raw, {compressed:,} bytes with gzip -9 -n")

    with open(DOC) as fh:
        text = fh.read()
    replacement = PARAGRAPH.format(
        raw=raw,
        gzip=compressed,
        baseline=BASELINE_BYTES,
        baseline_commit=BASELINE_COMMIT,
        growth=100.0 * (raw - BASELINE_BYTES) / BASELINE_BYTES,
    )
    updated, count = PATTERN.subn(replacement, text)
    if count == 0:
        print("the size paragraph was not found", file=sys.stderr)
        return 1
    if args.check:
        if updated != text:
            print("docs/architecture.md is out of date")
            return 1
        print("docs/architecture.md is up to date")
        return 0
    with open(DOC, "w") as fh:
        fh.write(updated)
    print(f"updated {os.path.relpath(DOC, ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
