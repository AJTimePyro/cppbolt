const assert = require("assert");

// (module (func (export "main") (result i32) (loop (br 0)) (i32.const 0)))
const infiniteLoopWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x0a, 0x0b, 0x01, 0x09, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b
]);

// (module (func (export "main") (result i32) (i32.const 42) (return)))
const normalWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b
]);

// (module (import "env" "read_char" (func (result i32))) (func (export "main") (result i32) (call 0)))
const readStdinWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x02, 0x11, 0x01,
    0x03, 0x65, 0x6e, 0x76,
    0x09, 0x72, 0x65, 0x61, 0x64, 0x5f, 0x63, 0x68, 0x61, 0x72,
    0x00, 0x00,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01,
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b
]);

// (module (import "env" "print_char" (func (param i32))) (func (export "main") (result i32) (call 0 (i32.const 63)) (i32.const 0)))
const printStdoutWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x09, 0x02,
    0x60, 0x00, 0x01, 0x7f,
    0x60, 0x01, 0x7f, 0x00,
    0x02, 0x12, 0x01,
    0x03, 0x65, 0x6e, 0x76,
    0x0a, 0x70, 0x72, 0x69, 0x6e, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x72,
    0x00, 0x01,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01,
    0x0a, 0x0a, 0x01, 0x08, 0x00, 0x41, 0x3f, 0x10, 0x00, 0x41, 0x00, 0x0b
]);

async function testNodeWorker() {
    const pkg = require("../npm/dist/index.js");

    assert.strictEqual(typeof pkg.AsyncCppBoltRunner, "function", "AsyncCppBoltRunner should be exported in Node");

    const host = pkg.createWasmHost({});
    const runner = new pkg.AsyncCppBoltRunner(host);

    const res1 = await runner.run(normalWasmBytes, { timeoutMs: 1000 });
    assert.strictEqual(res1.exitCode, 42);

    const started = Date.now();
    let threw = false;
    try {
        await runner.run(infiniteLoopWasmBytes, { timeoutMs: 150 });
    } catch (err) {
        threw = true;
        assert.ok(err instanceof pkg.TimeoutError, "Error must be instance of TimeoutError");
        assert.match(err.message, /timed out/i);
    }
    const elapsed = Date.now() - started;
    assert.strictEqual(threw, true, "Infinite loop must throw TimeoutError");
    assert.ok(elapsed >= 140 && elapsed < 800, `Elapsed ${elapsed}ms out of bounds`);

    const controller = new AbortController();
    setTimeout(() => controller.abort(), 80);
    threw = false;
    try {
        await runner.run(infiniteLoopWasmBytes, { signal: controller.signal });
    } catch (err) {
        threw = true;
        assert.ok(err instanceof pkg.TimeoutError, "Abort must throw TimeoutError");
        assert.match(err.message, /aborted/i);
    }
    assert.strictEqual(threw, true, "Aborted execution must throw TimeoutError");

    let invalidThrew = false;
    try {
        await runner.run(new Uint8Array([0, 1, 2, 3]));
    } catch (err) {
        invalidThrew = true;
        assert.ok(err instanceof Error, "Invalid wasm must reject with Error");
    }
    assert.strictEqual(invalidThrew, true, "Invalid wasm must reject Promise");

    // Test no-argument constructor
    const runnerNoHost = new pkg.AsyncCppBoltRunner();
    const resNoHost = await runnerNoHost.run(normalWasmBytes);
    assert.strictEqual(resNoHost.exitCode, 42);

    // Test stdin forwarding
    const resStdin = await runner.run(readStdinWasmBytes, { stdin: "Z" });
    assert.strictEqual(resStdin.exitCode, 90, "stdin 'Z' (90) must be read by wasm");

    // Test streaming onStdout callback
    const chunks = [];
    const resStdout = await runner.run(printStdoutWasmBytes, {
        onStdout: (c) => chunks.push(c)
    });
    assert.strictEqual(resStdout.exitCode, 0);
    assert.strictEqual(chunks.join(""), "?", "onStdout must receive streamed output chunks");

    console.log("Node worker test passed!");
    process.exit(0);
}

testNodeWorker().catch((e) => {
    console.error("Test failed:", e);
    process.exit(1);
});
