#!/usr/bin/env python3
import os
import sys
import glob
import subprocess
import re

def parse_records(text):
    records = {}
    current_name = None
    current_lines = []
    in_ast_layout = False
    for line in text.splitlines():
        if line.startswith("*** Dumping AST Record Layout"):
            if current_name and current_lines:
                records[current_name] = "\n".join(current_lines)
            current_name = None
            current_lines = []
            in_ast_layout = True
            continue
        elif line.startswith("*** Dumping"):
            if current_name and current_lines:
                records[current_name] = "\n".join(current_lines)
            current_name = None
            current_lines = []
            in_ast_layout = False
            continue
            
        if in_ast_layout:
            m = re.match(r"^\s*0\s+\|\s+(?:struct|class)\s+([A-Za-z0-9_]+)", line)
            if m and current_name is None:
                current_name = m.group(1)
            if current_name is not None:
                current_lines.append(line)
    if current_name and current_lines:
        records[current_name] = "\n".join(current_lines)
    return records

def main():
    compiler_bin = sys.argv[1] if len(sys.argv) > 1 else "./target/release/cppbolt-cli"
    test_dir = sys.argv[2] if len(sys.argv) > 2 else "tests/classes_oop"
    files = sorted(glob.glob(os.path.join(test_dir, "**", "*.cpp"), recursive=True))
    
    total_records = 0
    failures = 0
    print(f"Verifying record layouts in {test_dir} against clang++ --target=wasm32...")
    
    for f in files:
        with open(f) as fp:
            src = fp.read()
        if "struct " not in src and "class " not in src:
            continue
            
        clean_src = src.replace("#include <cstdio>", "extern \"C\" int printf(const char*, ...);")
        clean_src = clean_src.replace("std::printf", "printf")

        p_clang = subprocess.run(
            ["clang++", "--target=wasm32", "-nostdinc++", "-I", "stdlib", "-x", "c++", "-Xclang", "-fdump-record-layouts", "-c", "-o", "/dev/null", "-"],
            input=clean_src, text=True, capture_output=True
        )
        p_our = subprocess.run(
            [compiler_bin, "--dump-record-layouts", f],
            capture_output=True, text=True
        )
        if p_clang.returncode != 0 or p_our.returncode != 0:
            failures += 1
            print(f"  [LAYOUT FAIL] {os.path.basename(f)}: compiler exited nonzero")
            if p_clang.returncode != 0:
                print(f"    clang++ exit {p_clang.returncode}: {p_clang.stderr.strip()}")
            if p_our.returncode != 0:
                print(f"    {compiler_bin} exit {p_our.returncode}: {p_our.stderr.strip()}")
            continue
        clang_recs = parse_records(p_clang.stdout)
        our_recs = parse_records(p_our.stdout)
        
        diffs = []
        for k, v in list(clang_recs.items()):
            clang_recs[k] = v.replace("_Bool", "bool").replace("class std::string", "class string").replace("size_t", "unsigned int")
        for rec_name, our_dump in our_recs.items():
            total_records += 1
            if rec_name not in clang_recs:
                diffs.append(f"Record {rec_name} missing from clang dump")
            elif clang_recs[rec_name].strip() != our_dump.strip():
                diffs.append(f"Record {rec_name} differs:\nOUR:\n{our_dump}\nCLANG:\n{clang_recs[rec_name]}")
        
        if diffs:
            failures += 1
            print(f"  [LAYOUT FAIL] {os.path.basename(f)}:")
            for d in diffs:
                print("    ", d)
        else:
            print(f"  [LAYOUT MATCH] {os.path.basename(f)} ({len(our_recs)} records)")
            
    if failures == 0:
        print(f"All {total_records} record layouts match clang++ --target=wasm32 exactly.")
        return 0
    else:
        print(f"{failures} tests had layout mismatches.")
        return 1

if __name__ == "__main__":
    sys.exit(main())
