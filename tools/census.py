#!/usr/bin/env python3
"""Code-size census of the Rust sources.

Counts the functions that are longer than 60, 80, and 100 lines, the
functions with more than five parameters, and the files with more than 500
lines.  A function is counted from the line that holds its signature to the
line that closes its body.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "src"

FUNCTIONS_OVER = (60, 80, 100)
PARAMETERS_OVER = 5
FILE_LINES_OVER = 500

SIGNATURE = re.compile(r"^\s*(pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?extern\s+\"[^\"]*\"\s+fn\s|^\s*(pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s")


def strip_strings_and_comments(text):
    """Replaces the contents of strings and comments with spaces."""
    out = []
    index = 0
    length = len(text)
    while index < length:
        char = text[index]
        if char == '"':
            out.append(" ")
            index += 1
            while index < length and text[index] != '"':
                if text[index] == "\\":
                    out.append(" ")
                    index += 1
                    if index < length:
                        out.append(" ")
                        index += 1
                    continue
                out.append("\n" if text[index] == "\n" else " ")
                index += 1
            if index < length:
                out.append(" ")
                index += 1
            continue
        if char == "/" and index + 1 < length and text[index + 1] == "/":
            while index < length and text[index] != "\n":
                out.append(" ")
                index += 1
            continue
        out.append(char)
        index += 1
    return "".join(out)


def parameter_count(signature):
    """Counts the parameters of a function signature.

    The receiver is not a parameter, so `self` does not count.
    """
    start = signature.find("(")
    if start < 0:
        return 0
    depth = 0
    count = 0
    seen = False
    for char in signature[start:]:
        if char == "(":
            depth += 1
            if depth == 1:
                continue
        if char == ")":
            depth -= 1
            if depth == 0:
                break
        if depth == 1 and char == ",":
            count += 1
        if depth == 1 and not char.isspace():
            seen = True
    if not seen:
        return 0
    count += 1
    if signature[start:].lstrip("( \t\n").startswith("&mut self") or signature[
        start:
    ].lstrip("( \t\n").startswith("mut self") or signature[start:].lstrip(
        "( \t\n"
    ).startswith("self"):
        count -= 1
    return count


def functions(lines):
    """Yields the start line, the end line, and the signature of each function."""
    index = 0
    while index < len(lines):
        line = lines[index]
        if not SIGNATURE.match(line):
            index += 1
            continue
        # Collect the signature, which may span several lines.
        signature = line
        end = index
        while "(" in signature and signature.count("(") > signature.count(")"):
            end += 1
            if end >= len(lines):
                break
            signature += " " + lines[end].strip()
        depth = 0
        body = end
        started = False
        while body < len(lines):
            for char in lines[body]:
                if char == "{":
                    depth += 1
                    started = True
                elif char == "}":
                    depth -= 1
            if started and depth == 0:
                break
            body += 1
        yield index + 1, body + 1, signature
        index = body + 1


def main():
    counts = {limit: 0 for limit in FUNCTIONS_OVER}
    many_parameters = 0
    long_files = 0
    total_functions = 0
    longest = (0, "")

    for path in sorted(SRC.rglob("*.rs")):
        text = path.read_text()
        lines = text.splitlines()
        if len(lines) > FILE_LINES_OVER:
            long_files += 1
        cleaned = strip_strings_and_comments(text).splitlines()
        for start, end, signature in functions(cleaned):
            total_functions += 1
            length = end - start + 1
            if length > longest[0]:
                longest = (length, f"{path.relative_to(ROOT)}:{start}")
            for limit in FUNCTIONS_OVER:
                if length > limit:
                    counts[limit] += 1
            if parameter_count(signature) > PARAMETERS_OVER:
                many_parameters += 1

    print(f"functions: {total_functions}")
    for limit in FUNCTIONS_OVER:
        print(f"functions over {limit} lines: {counts[limit]}")
    print(f"functions over {PARAMETERS_OVER} parameters: {many_parameters}")
    print(f"files over {FILE_LINES_OVER} lines: {long_files}")
    print(f"longest function: {longest[0]} lines at {longest[1]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
