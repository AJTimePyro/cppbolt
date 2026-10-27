#!/usr/bin/env python3
"""Validates that all internal and repo-relative links in the built mdBook HTML resolve."""

import os
import re
import sys
from urllib.parse import unquote

def main() -> int:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    book_html_dir = os.path.join(repo_root, "docs", "book", "book")

    if not os.path.isdir(book_html_dir):
        print(f"Error: Built book HTML directory not found at {book_html_dir}. Run 'mdbook build docs/book' first.", file=sys.stderr)
        return 1

    broken_internal = []
    broken_repo = []
    checked_internal = 0
    checked_repo = 0

    repo_prefix = "https://github.com/AJTimePyro/cpp-web-compiler/blob/main/"

    for root, _, files in os.walk(book_html_dir):
        for f in files:
            if not f.endswith(".html"):
                continue
            html_file = os.path.join(root, f)
            with open(html_file, "r", encoding="utf-8") as handle:
                content = handle.read()

            links = re.findall(r"<a\s+(?:[^>]*?\s+)?href=\"([^\"]*)\"", content)
            for link in links:
                if not link or link.startswith("#") or link.startswith("mailto:"):
                    continue

                if link.startswith(repo_prefix):
                    checked_repo += 1
                    subpath = link[len(repo_prefix):].split("#")[0]
                    target_file = os.path.join(repo_root, subpath)
                    if not os.path.exists(target_file):
                        broken_repo.append((html_file, link, target_file))
                    continue

                if link.startswith("http://") or link.startswith("https://"):
                    continue

                # Local relative link
                checked_internal += 1
                target = link.split("#")[0]
                if not target:
                    continue
                target_path = os.path.normpath(os.path.join(root, unquote(target)))
                if not os.path.exists(target_path):
                    broken_internal.append((html_file, link, target_path))

    errors = False
    if broken_internal:
        print(f"FAILURE: {len(broken_internal)} broken internal link(s) found in {book_html_dir}:", file=sys.stderr)
        for src, link, target in broken_internal:
            rel_src = os.path.relpath(src, book_html_dir)
            print(f"  {rel_src} -> {link} (resolved: {target})", file=sys.stderr)
        errors = True

    if broken_repo:
        print(f"FAILURE: {len(broken_repo)} broken repository link(s) found in {book_html_dir}:", file=sys.stderr)
        for src, link, target in broken_repo:
            rel_src = os.path.relpath(src, book_html_dir)
            print(f"  {rel_src} -> {link} (missing on disk: {target})", file=sys.stderr)
        errors = True

    if errors:
        return 1

    print(f"SUCCESS: Verified {checked_internal} internal links and {checked_repo} repository links. 0 broken links.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
