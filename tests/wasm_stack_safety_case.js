// Compiles one source file with the wasm-hosted compiler and prints the
// outcome.  It runs in its own Node process so that every case starts with a
// fresh engine stack budget.
//
// Usage: node tests/wasm_stack_safety_case.js <cppbolt.wasm> <source.cpp>

const fs = require("fs");

const wasmPath = process.argv[2];
const sourcePath = process.argv[3];
if (!wasmPath || !sourcePath) {
    console.error("usage: node tests/wasm_stack_safety_case.js <cppbolt.wasm> <source.cpp>");
    process.exit(2);
}

const sourceBytes = Buffer.from(fs.readFileSync(sourcePath, "utf8"), "utf8");

WebAssembly.instantiate(fs.readFileSync(wasmPath), {})
    .then(({ instance }) => {
        const exports = instance.exports;
        const capacity = sourceBytes.length === 0 ? 1 : sourceBytes.length;
        const input = exports.alloc_buffer(capacity);
        new Uint8Array(exports.memory.buffer).set(sourceBytes, input);
        const started = Date.now();
        let outcome;
        try {
            const result = exports.compile_wasm(input, sourceBytes.length);
            const elapsed = Date.now() - started;
            if (result) {
                const length = exports.get_result_len();
                // Copy before touching memory again: growth can detach the view.
                const output = new Uint8Array(exports.memory.buffer).slice(result, result + length);
                exports.free_result(result, length);
                outcome = output.length > 8 ? { kind: "ok" } : { kind: "trap", detail: "empty module" };
            } else {
                const errorLength = exports.get_error_len();
                if (errorLength === 0) {
                    outcome = { kind: "trap", detail: "failure without a diagnostic" };
                } else {
                    const pointer = exports.get_error_ptr();
                    const message = Buffer.from(
                        new Uint8Array(exports.memory.buffer).slice(pointer, pointer + errorLength),
                    ).toString("utf8");
                    outcome = { kind: "diagnostic", message };
                }
            }
            if (outcome.kind === "ok" && elapsed > 10000) {
                outcome = { kind: "trap", detail: `took ${elapsed}ms` };
            }
        } catch (error) {
            outcome = {
                kind: error instanceof RangeError ? "rangeerror" : "trap",
                detail: error && error.message ? error.message : String(error),
            };
        }
        try {
            exports.dealloc_buffer(input, capacity);
        } catch (_) {}
        console.log(JSON.stringify(outcome));
    })
    .catch((error) => {
        console.log(JSON.stringify({ kind: "rangeerror", detail: String(error && error.message) }));
    });
