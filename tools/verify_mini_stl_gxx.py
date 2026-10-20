#!/usr/bin/env python3
"""
Oracle gate for the embedded C++ standard headers.

Compiles every positive corpus test with a real compiler using only the
repository headers:

  <compiler> -std=c++17 -nostdinc++ -fno-exceptions -fsanitize=address,undefined
             -fvisibility=hidden -I stdlib <test.cpp>

and checks that the resulting native program reproduces the recorded
`.expected` output. The gate fails loudly when a required compiler, header
directory, test directory, or `.expected` file is missing.
"""
import glob
import os
import shutil
import subprocess
import sys

POSITIVE_DIRS = [
    "c_basics",
    "c_basics_variant",
    "classes_oop",
    "classes_oop_variant",
    "memory_model",
    "memory_model_variant",
    "templates_operators",
    "mini_stl",
    "mini_stl_variant",
]

# These tests intentionally observe the custom WebAssembly allocator, so a
# native libc allocator cannot reproduce their recorded output.
NATIVE_ORACLE_SKIPS = {
    "free_list_reuse.cpp": "output depends on the WebAssembly allocator",
    "malloc_failure.cpp": "Sanitizers abort instead of returning null for impossible sizes",
}


def run_suite(compiler, files, header_dir):
    passed = 0
    failed = 0
    skipped = 0
    print(
        f"Gate: compiling {len(files)} positive tests with {compiler} "
        f"-nostdinc++ -fsanitize=address,undefined..."
    )
    for f in files:
        tname = os.path.basename(f)
        if tname in NATIVE_ORACLE_SKIPS:
            print(f"  [SKIP] {tname}: {NATIVE_ORACLE_SKIPS[tname]}")
            skipped += 1
            continue
        bin_path = f"/tmp/gate_{compiler}_{tname}.bin"
        cmd = [
            compiler,
            "-std=c++17",
            "-nostdinc++",
            "-fno-exceptions",
            "-fsanitize=address,undefined",
            "-fvisibility=hidden",
            "-I",
            header_dir,
            f,
            "-o",
            bin_path,
        ]
        cp = subprocess.run(cmd, capture_output=True, text=True)
        if cp.returncode != 0:
            print(f"  [FAIL COMPILE] {tname}:\n{cp.stderr}")
            failed += 1
            continue

        inp_file = f[:-4] + ".input"
        inp = open(inp_file).read() if os.path.exists(inp_file) else None
        rp = subprocess.run([bin_path], input=inp, capture_output=True, text=True)
        if os.path.exists(bin_path):
            os.remove(bin_path)

        if rp.returncode != 0:
            print(f"  [FAIL RUNTIME/SAN] {tname} exit={rp.returncode}\n{rp.stderr}")
            failed += 1
            continue

        exp_file = f[:-4] + ".expected"
        if not os.path.exists(exp_file):
            print(f"  [FAIL] {tname} missing {os.path.basename(exp_file)}")
            failed += 1
            continue

        exp = open(exp_file).read()
        if rp.stdout != exp:
            print(f"  [FAIL OUTPUT] {tname}:\nEXPECTED:\n{exp}\nACTUAL:\n{rp.stdout}")
            failed += 1
        else:
            passed += 1
    print(
        f"{compiler} Gate Result: {passed}/{len(files) - skipped} passed, "
        f"{failed} failed, {skipped} skipped.\n"
    )
    return failed


def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    tests_dir = os.path.join(root_dir, "tests")
    header_dir = os.path.join(root_dir, "stdlib")

    if not os.path.isdir(header_dir):
        print(f"[FAIL] missing header directory: {header_dir}")
        return 1

    files = []
    for name in POSITIVE_DIRS:
        directory = os.path.join(tests_dir, name)
        if not os.path.isdir(directory):
            print(f"[FAIL] missing test directory: {directory}")
            return 1
        files.extend(sorted(glob.glob(os.path.join(directory, "*.cpp"))))
    if not files:
        print(f"[FAIL] no tests found under {tests_dir}")
        return 1

    for compiler in ("g++", "clang++"):
        if shutil.which(compiler) is None:
            print(f"[FAIL] required compiler not found: {compiler}")
            return 1

    failed = 0
    for compiler in ("g++", "clang++"):
        failed += run_suite(compiler, files, header_dir)

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
