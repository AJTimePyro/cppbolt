const fs = require("fs");

async function main() {
    const compilerWasmBytes = fs.readFileSync("target/wasm32-unknown-unknown/release/cppwasm.wasm");
    const module = await WebAssembly.instantiate(compilerWasmBytes, {});
    const instance = module.instance;

    const source = fs.readFileSync("tests/c_basics/types/arithmetic.cpp", "utf8");
    const sourceBytes = Buffer.from(source, "utf8");

    // Allocate memory in wasm
    const srcPtr = instance.exports.alloc_buffer(sourceBytes.length);
    const wasmMem = new Uint8Array(instance.exports.memory.buffer);
    wasmMem.set(sourceBytes, srcPtr);

    // Call compiler
    const outPtr = instance.exports.compile_wasm(srcPtr, sourceBytes.length);
    if (!outPtr) {
        const errorPtr = instance.exports.get_error_ptr();
        const errorLen = instance.exports.get_error_len();
        const message = Buffer.from(
            new Uint8Array(instance.exports.memory.buffer).slice(errorPtr, errorPtr + errorLen),
        ).toString("utf8");
        console.error("Compilation in WASM failed: " + message);
        process.exit(1);
    }
    const outLen = instance.exports.get_result_len();

    // Read emitted wasm
    const wasmMemAfter = new Uint8Array(instance.exports.memory.buffer);
    const outputWasm = Buffer.from(wasmMemAfter.slice(outPtr, outPtr + outLen));

    // Results are boxed slices with their own allocation; release both buffers.
    instance.exports.free_result(outPtr, outLen);
    instance.exports.dealloc_buffer(srcPtr, sourceBytes.length);

    fs.writeFileSync("/tmp/from_wasm_compiler.wasm", outputWasm);
    console.log(`Successfully compiled in WASM! Output size: ${outputWasm.length} bytes`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
