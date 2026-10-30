// Exercises the wasm-hosted compiler on wide and deeply nested inputs to prove
// the shared depth limit produces a diagnostic rather than a WebAssembly trap,
// and that wide left-associative chains still compile.
//
// Usage: node tests/wasm_stack_safety.js <cppbolt.wasm> [limits.json]
//
// The limits file holds the per-construct nesting limits of the compiler as
// {"parentheses": N, "calls": N, "statements": N}; when it is given, the
// last accepted depth and the first rejected one are checked for every
// construct.

const fs = require("fs");

const wasmPath = process.argv[2];
if (!wasmPath) {
    console.error("usage: node tests/wasm_stack_safety.js <cppbolt.wasm> [limits.json]");
    process.exit(2);
}

const limits = process.argv[3] ? JSON.parse(fs.readFileSync(process.argv[3], "utf8")) : null;

const MAX_MS = 10000;

function binaryChain(terms) {
    const termsList = new Array(terms).fill("a").join("+");
    return `#include <cstdio>\nint main() { int a = 1; int s = ${termsList}; printf("%d\\n", s); return 0; }\n`;
}

function stringConcat(terms) {
    // Two string literals cannot be added, so the chain starts at a
    // `std::string` seed and appends one literal at a time.
    const termsList = new Array(terms).fill('"x"').join(" + ");
    return `#include <string>\nint main() { std::string s = "seed"; s = s + ${termsList}; return (int)s.size(); }\n`;
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

function nestedParentheses(depth) {
    return `int main() { int v = ${"(".repeat(depth)}1${")".repeat(depth)}; return v; }\n`;
}

function nestedParenthesizedTernary(depth) {
    let source = "int main() { int v = ";
    source += "(".repeat(depth);
    source += "1?1:";
    for (let index = 1; index < depth; index += 1) {
        source += "1?1:";
    }
    source += "0";
    source += ")".repeat(depth);
    source += "; return v; }\n";
    return source;
}

function nestedIfs(depth) {
    let source = "int main() { int v = 0;\n";
    for (let index = 0; index < depth; index += 1) {
        source += "  if (v >= 0) {\n";
    }
    source += "  v += 1;\n";
    for (let index = 0; index < depth; index += 1) {
        source += "  }\n";
    }
    source += " return v; }\n";
    return source;
}

const limitCases = [];
if (limits) {
    limitCases.push(
        {
            name: `nested parentheses ${limits.parentheses - 1}`,
            source: nestedParentheses(limits.parentheses - 1),
            expect: "ok",
        },
        {
            name: `nested parentheses ${limits.parentheses + 1}`,
            source: nestedParentheses(limits.parentheses + 1),
            expect: "diagnostic",
        },
        {
            name: `nested parenthesized ternaries ${limits.parentheses - 1}`,
            source: nestedParenthesizedTernary(limits.parentheses - 1),
            expect: "ok",
        },
        {
            name: `nested parenthesized ternaries ${limits.parentheses + 1}`,
            source: nestedParenthesizedTernary(limits.parentheses + 1),
            expect: "diagnostic",
        },
        {
            name: `nested calls ${limits.calls - 1}`,
            source: nestedCalls(limits.calls - 1),
            expect: "ok",
        },
        {
            name: `nested calls ${limits.calls + 1}`,
            source: nestedCalls(limits.calls + 1),
            expect: "diagnostic",
        },
    );
    // The innermost statement of an `if` nest is one level deeper than the
    // `if` statements themselves.
    const ifLimit = limits.statements - 1;
    limitCases.push(
        {
            name: `nested ifs ${ifLimit - 1}`,
            source: nestedIfs(ifLimit - 1),
            expect: "ok",
        },
        {
            name: `nested ifs ${ifLimit + 1}`,
            source: nestedIfs(ifLimit + 1),
            expect: "diagnostic",
        },
    );
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
    ...limitCases,
];

const { execFileSync } = require("child_process");
const os = require("os");
const path = require("path");

const caseScript = path.join(__dirname, "wasm_stack_safety_case.js");
let counter = 0;

// Every case runs in its own Node process: the engine reuses its stack budget
// across compiles, so a process that already compiled a deep program has
// less room for the next one.  A fresh process per case measures the
// compiler, not the state of the host.
function compileOne(source) {
    const file = path.join(os.tmpdir(), `cppwasm-stack-${process.pid}-${counter}.cpp`);
    counter += 1;
    fs.writeFileSync(file, source);
    try {
        const output = execFileSync(process.execPath, [caseScript, wasmPath, file], {
            encoding: "utf8",
            maxBuffer: 16 * 1024 * 1024,
        });
        return JSON.parse(output.trim().split("\n").pop());
    } catch (error) {
        return { kind: "trap", detail: String(error && error.message) };
    } finally {
        try {
            fs.unlinkSync(file);
        } catch (_) {}
    }
}

let failures = 0;
for (const testCase of cases) {
    const outcome = compileOne(testCase.source);
    if (outcome.kind === "trap" || outcome.kind === "rangeerror") {
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
    }
}
if (failures > 0) {
    console.error(`wasm stack safety: ${failures} case(s) failed`);
    process.exit(1);
}
console.log(`wasm stack safety: OK (${cases.length} cases)`);
