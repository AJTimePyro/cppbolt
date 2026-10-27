#!/usr/bin/env python3
"""Validates that all test paths referenced in the Supported C++ feature matrix exist on disk."""

import os
import re
import sys

def main() -> int:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    matrix_file = os.path.join(repo_root, "docs", "book", "user", "supported-cpp.md")

    if not os.path.isfile(matrix_file):
        print(f"Error: Matrix file not found at {matrix_file}", file=sys.stderr)
        return 1

    with open(matrix_file, "r", encoding="utf-8") as f:
        content = f.read()

    # Extract all test references: tests/<group>/<filename>.(cpp|h|rs|txt|js)
    test_paths = set(re.findall(r"tests/[a-zA-Z0-9_\-/\.]+\.(?:cpp|h|rs|txt|js)", content))

    missing = []
    for rel_path in sorted(test_paths):
        full_path = os.path.join(repo_root, rel_path)
        if not os.path.isfile(full_path):
            missing.append(rel_path)

    if missing:
        print(f"FAILURE: {len(missing)} test file(s) referenced in {matrix_file} do not exist on disk:", file=sys.stderr)
        for path in missing:
            print(f"  - {path}", file=sys.stderr)
        return 1

    print(f"SUCCESS: All {len(test_paths)} test files referenced in the feature matrix exist on disk.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
