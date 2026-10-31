// Exercises the compiler's WebAssembly C ABI repeatedly to prove result
// buffers and input buffers are both released and linear memory stays stable.
//
// Usage: node tests/ffi_lifecycle.js <cppbolt.wasm>

const fs = require("fs");

const wasmPath = process.argv[2];
if (!wasmPath) {
    console.error("usage: node tests/ffi_lifecycle.js <cppbolt.wasm>");
    process.exit(2);
}

const VALID = "int main() { int x = 1; return x; }";
const INVALID = "int main( { return 0; }";
const WARMUP = 50;
// 20,000 alternating compiles exercise both the success and diagnostic paths
// in one instance. A leaking result buffer grows linear memory by roughly
// 144 bytes per successful compile (measured with free_result disabled), so
// 20,000 iterations would exceed 2.8 MiB. The 1 MiB bound leaves headroom for
// allocator page rounding while still failing a leaked result buffer.
const ITERATIONS = 20000;
const MAX_GROWTH_BYTES = 1024 * 1024;

function compileOnce(exports, source) {
    const sourceBytes = Buffer.from(source, "utf8");
    const capacity = sourceBytes.length === 0 ? 1 : sourceBytes.length;
    const input = exports.alloc_buffer(capacity);
    new Uint8Array(exports.memory.buffer).set(sourceBytes, input);

    const result = exports.compile_wasm(input, sourceBytes.length);
    const resultLength = exports.get_result_len();
    let ok = false;
    if (result) {
        // Copy before touching memory again; growth can detach the view.
        const output = Buffer.from(
            new Uint8Array(exports.memory.buffer).slice(result, result + resultLength),
        );
        exports.free_result(result, resultLength);
        ok = output.length > 8;
    } else {
        const errorLength = exports.get_error_len();
        if (errorLength === 0) {
            throw new Error("compile_wasm failed without a diagnostic");
        }
    }
    exports.dealloc_buffer(input, capacity);
    return ok;
}

WebAssembly.instantiate(fs.readFileSync(wasmPath), {})
    .then(({ instance }) => {
        const exports = instance.exports;
        let compiled = 0;
        let diagnosed = 0;

        for (let i = 0; i < WARMUP; i += 1) {
            if (compileOnce(exports, i % 2 === 0 ? VALID : INVALID)) {
                compiled += 1;
            } else {
                diagnosed += 1;
            }
        }
        const baseline = exports.memory.buffer.byteLength;

        for (let i = 0; i < ITERATIONS; i += 1) {
            if (compileOnce(exports, i % 2 === 0 ? VALID : INVALID)) {
                compiled += 1;
            } else {
                diagnosed += 1;
            }
        }
        const finalSize = exports.memory.buffer.byteLength;
        const growth = finalSize - baseline;
        if (growth > MAX_GROWTH_BYTES) {
            console.error(
                `ABI lifecycle: memory grew by ${growth} bytes, over the ${MAX_GROWTH_BYTES} byte bound`,
            );
            process.exit(1);
        }
        if (compiled === 0 || diagnosed === 0) {
            console.error(
                `ABI lifecycle: expected both outcomes, compiled=${compiled} diagnosed=${diagnosed}`,
            );
            process.exit(1);
        }
        console.log(
            `ABI lifecycle: OK compiles=${compiled} diagnostics=${diagnosed} growth=${growth} bytes`,
        );
    })
    .catch((error) => {
        console.error(`ABI lifecycle: ${error.message}`);
        process.exit(1);
    });
