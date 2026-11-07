const assert = require("assert");

async function testOptions() {
    const pkg = require("../npm/dist/index.js");

    assert.strictEqual(typeof pkg.TimeoutError, "function", "TimeoutError should be exported");
    const err = new pkg.TimeoutError("Execution timed out", "partial output", 150);
    assert.strictEqual(err.name, "TimeoutError");
    assert.strictEqual(err.stdout, "partial output");
    assert.strictEqual(err.elapsedMs, 150);
    assert.match(err.message, /timed out/);

    const host = pkg.createWasmHost({});
    const runner = new pkg.CppBoltRunner(host);
    const minimalWasmBytes = new Uint8Array([
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00,
        0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b
    ]);

    const result = await runner.run(minimalWasmBytes, {
        timeoutMs: 1000,
        stdin: "input data",
        onStdout: (chunk) => {},
    });
    assert.strictEqual(result.exitCode, 42);
    console.log("NPM runner options test passed!");
}

testOptions().catch((e) => {
    console.error("Test failed:", e);
    process.exit(1);
});
