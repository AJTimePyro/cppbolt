#!/usr/bin/env python3
"""
Validates that our stdlib C++ headers are 100% conforming standard C++
by compiling all tests/mini_stl/*.cpp tests with real g++ and clang++ using:
  g++ -std=c++17 -nostdinc++ -fsanitize=address,undefined -fvisibility=hidden -I stdlib/ <test.cpp>
"""
import glob
import os
import subprocess
import sys

def run_suite(compiler, files, header_dir):
    passed = 0
    failed = 0
    print(f"Gate: compiling {len(files)} mini_stl tests with {compiler} -nostdinc++ -fsanitize=address,undefined...")
    for f in files:
        tname = os.path.basename(f)
        bin_path = f"/tmp/gate_{compiler}_{tname}.bin"
        cmd = [
            compiler, "-std=c++17", "-nostdinc++",
            "-fno-exceptions", "-fsanitize=address,undefined", "-fvisibility=hidden",
            "-I", header_dir, f, "-o", bin_path
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
            print(f"  [FAIL] {tname} missing .expected")
            failed += 1
            continue
            
        exp = open(exp_file).read()
        if rp.stdout != exp:
            print(f"  [FAIL OUTPUT] {tname}:\nEXPECTED:\n{exp}\nACTUAL:\n{rp.stdout}")
            failed += 1
        else:
            passed += 1
    print(f"{compiler} Gate Result: {passed}/{len(files)} passed, {failed} failed.\n")
    return failed

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    test_dir = os.path.join(root_dir, "tests", "stdlib")
    header_dir = os.path.join(root_dir, "stdlib")
    
    files = sorted(glob.glob(os.path.join(test_dir, "*.cpp")))
    if not files:
        print("No tests found in tests/mini_stl")
        return 1
        
    gxx_fail = run_suite("g++", files, header_dir)
    clang_fail = run_suite("clang++", files, header_dir)
    
    return 0 if (gxx_fail == 0 and clang_fail == 0) else 1

if __name__ == "__main__":
    sys.exit(main())
