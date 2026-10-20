// Exercises the wasm-hosted compiler on wide and deeply nested inputs to prove
// the shared depth limit produces a diagnostic rather than a WebAssembly trap,
// and that wide left-associative chains still compile.
//
// Usage: node tests/wasm_stack_safety.js <cppwasm.wasm>

const fs = require("fs");

const wasmPath = process.argv[2];
if (!wasmPath) {
    console.error("usage: node tests/wasm_stack_safety.js <cppwasm.wasm>");
    process.exit(2);
}

const MAX_MS = 10000;

function binaryChain(terms) {
    const termsList = new Array(terms).fill("a").join("+");
    return `#include <cstdio>\nint main() { int a = 1; int s = ${termsList}; printf("%d\\n", s); return 0; }\n`;
}

function stringConcat(terms) {
    const termsList = new Array(terms).fill('"x"').join("+");
    return `#include <string>\nint main() { std::string s; s = ${termsList}; return (int)s.size(); }\n`;
}

function elseIfChain(branches) {
    let source = "int main() { int x = 0; int r = -1; if (x == 0) r = 0;";
    for (let index = 1; index < branches; index += 1) {
        source += ` else if (x == ${index}) r = ${index};`;
    }
    return `${source} return r; }\n`;
}

function nestedCalls(depth) {
    return `int inc(int x) { return x + 1; }\nint main() { return ${"inc(".repeat(depth)}0${")".repeat(depth)}; }\n`;
}

function nestedTernary(depth) {
    return `int main() { return ${"1?1:".repeat(depth)}0; }\n`;
}

function nestedNamespaces(depth) {
    return `${"namespace N {".repeat(depth)}int v = 1;${"}".repeat(depth)}\nint main() { return 0; }\n`;
}

function nestedBlocks(depth) {
    return `int main() { ${"{".repeat(depth)}int x = 5; x += 1;${"}".repeat(depth)} return 0; }\n`;
}

function nestedCasts(depth) {
    return `int main() { return ${"(int)".repeat(depth)}1; }\n`;
}

const cases = [
    { name: "binary 100", source: binaryChain(100), expect: "ok" },
    { name: "binary 300", source: binaryChain(300), expect: "ok" },
    { name: "binary 1000", source: binaryChain(1000), expect: "ok" },
    { name: "binary 5000", source: binaryChain(5000), expect: "ok" },
    { name: "binary 20000", source: binaryChain(20000), expect: "ok" },
    { name: "string concat 100", source: stringConcat(100), expect: "ok" },
    { name: "string concat 1000", source: stringConcat(1000), expect: "ok" },
    { name: "string concat 4000", source: stringConcat(4000), expect: "ok" },
    { name: "else-if 100", source: elseIfChain(100), expect: "ok" },
    { name: "else-if 1000", source: elseIfChain(1000), expect: "ok" },
    { name: "else-if 5000", source: elseIfChain(5000), expect: "ok" },
    { name: "nested calls 1000", source: nestedCalls(1000), expect: "ok" },
    { name: "nested calls 5000", source: nestedCalls(5000), expect: "diagnostic" },
    { name: "nested ternaries 1000", source: nestedTernary(1000), expect: "ok" },
    { name: "nested ternaries 5000", source: nestedTernary(5000), expect: "diagnostic" },
    { name: "nested namespaces 1000", source: nestedNamespaces(1000), expect: "ok" },
    { name: "nested namespaces 5000", source: nestedNamespaces(5000), expect: "diagnostic" },
    { name: "nested blocks 1000", source: nestedBlocks(1000), expect: "ok" },
    { name: "nested blocks 5000", source: nestedBlocks(5000), expect: "diagnostic" },
    { name: "nested casts 1000", source: nestedCasts(1000), expect: "ok" },
    { name: "nested casts 4000", source: nestedCasts(4000), expect: "diagnostic" },
];

function compileOne(exports, source) {
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const bytes = encoder.encode(source);
    const capacity = bytes.length === 0 ? 1 : bytes.length;
    const input = exports.alloc_buffer(capacity);
    new Uint8Array(exports.memory.buffer).set(bytes, input);

    const started = Date.now();
    const result = exports.compile_wasm(input, bytes.length);
    const elapsed = Date.now() - started;
    let outcome;
    if (result) {
        const length = exports.get_result_len();
        // Copy before touching memory again; growth can detach the view.
        const output = new Uint8Array(exports.memory.buffer).slice(result, result + length);
        exports.free_result(result, length);
        outcome = output.length > 8 ? { kind: "ok" } : { kind: "trap", detail: "empty module" };
    } else {
        const errorLength = exports.get_error_len();
        if (errorLength === 0) {
            outcome = { kind: "trap", detail: "failure without a diagnostic" };
        } else {
            const pointer = exports.get_error_ptr();
            const message = decoder.decode(
                new Uint8Array(exports.memory.buffer).slice(pointer, pointer + errorLength),
            );
            outcome = { kind: "diagnostic", message };
        }
    }
    exports.dealloc_buffer(input, capacity);
    return { outcome, elapsed };
}

WebAssembly.instantiate(fs.readFileSync(wasmPath), {})
    .then(({ instance }) => {
        const exports = instance.exports;
        let failures = 0;
        for (const testCase of cases) {
            let outcome;
            let elapsed;
            try {
                const result = compileOne(exports, testCase.source);
                outcome = result.outcome;
                elapsed = result.elapsed;
            } catch (error) {
                outcome = { kind: "trap", detail: String(error) };
                elapsed = 0;
            }
            if (outcome.kind === "trap") {
                failures += 1;
                console.error(`[FAIL] ${testCase.name}: trapped (${outcome.detail})`);
                continue;
            }
            if (outcome.kind !== testCase.expect) {
                failures += 1;
                console.error(
                    `[FAIL] ${testCase.name}: expected ${testCase.expect}, got ${outcome.kind}${
                        outcome.message ? ` (${outcome.message})` : ""
                    }`,
                );
                continue;
            }
            if (elapsed > MAX_MS) {
                failures += 1;
                console.error(`[FAIL] ${testCase.name}: took ${elapsed}ms (over ${MAX_MS}ms)`);
            }
        }
        if (failures > 0) {
            console.error(`wasm stack safety: ${failures} case(s) failed`);
            process.exit(1);
        }
        console.log(`wasm stack safety: OK (${cases.length} cases)`);
    })
    .catch((error) => {
        console.error(`wasm stack safety: ${error.message}`);
        process.exit(1);
    });
