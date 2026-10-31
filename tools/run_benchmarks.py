#!/usr/bin/env python3
import subprocess
import time
import os
import sys
import statistics

BENCHMARKS = [
    ("1e8 Loop Iterations", "benchmarks/loop_1e8.cpp"),
    ("Sieve 1e7 Primes", "benchmarks/sieve_1e7.cpp"),
    ("Sort 1e6 Integers", "benchmarks/sort_1e6.cpp"),
    ("AVL Map 2e5 Inserts", "benchmarks/map_2e5.cpp"),
]

def main():
    print("=========================================================================================")
    print(" Performance Baseline Benchmark (Median of 5): Native vs WebAssembly (Our Compiler)")
    print("=========================================================================================")
    header = f"{'Benchmark':<24} | {'Native -O3':<12} | {'Native -O0':<12} | {'Wasm (Node)':<12} | {'Wasm/O3 Ratio':<12}"
    print(header)
    print("-" * 89)

    for name, src in BENCHMARKS:
        # 1. Native -O3
        bin_o3 = "/tmp/bench_native_o3"
        subprocess.run(["g++", "-std=c++17", "-O3", src, "-o", bin_o3], check=True)
        times_o3 = []
        for _ in range(5):
            t0 = time.perf_counter()
            p_o3 = subprocess.run([bin_o3], capture_output=True, text=True)
            t1 = time.perf_counter()
            times_o3.append((t1 - t0) * 1000.0)
        time_o3_ms = statistics.median(times_o3)

        # 2. Native -O0
        bin_o0 = "/tmp/bench_native_o0"
        subprocess.run(["g++", "-std=c++17", "-O0", src, "-o", bin_o0], check=True)
        times_o0 = []
        for _ in range(5):
            t0 = time.perf_counter()
            p_o0 = subprocess.run([bin_o0], capture_output=True, text=True)
            t1 = time.perf_counter()
            times_o0.append((t1 - t0) * 1000.0)
        time_o0_ms = statistics.median(times_o0)

        # 3. WebAssembly with our compiler
        wasm_file = "/tmp/bench.wasm"
        subprocess.run(["target/release/cppbolt-cli", src, "-o", wasm_file], check=True)
        
        js_cmd = f"""
        const fs = require('fs');
        const {{ createWasmImports }} = require('./web/host.js');
        const bytes = fs.readFileSync('{wasm_file}');
        const host = createWasmImports();
        const t0 = performance.now();
        WebAssembly.instantiate(bytes, host.imports).then(m => {{
            m.instance.exports.main();
            const t1 = performance.now();
            console.log((t1 - t0).toFixed(2));
        }});
        """
        times_wasm = []
        for _ in range(5):
            p_wasm = subprocess.run(["node", "-e", js_cmd], capture_output=True, text=True)
            try:
                times_wasm.append(float(p_wasm.stdout.strip()))
            except ValueError:
                pass
        time_wasm_ms = statistics.median(times_wasm) if times_wasm else float('nan')

        ratio = f"{time_wasm_ms / time_o3_ms:.2f}x" if time_o3_ms > 0 else "N/A"
        print(f"{name:<24} | {time_o3_ms:>9.2f} ms | {time_o0_ms:>9.2f} ms | {time_wasm_ms:>9.2f} ms | {ratio:>12}")

    print("=========================================================================================")

if __name__ == "__main__":
    main()
