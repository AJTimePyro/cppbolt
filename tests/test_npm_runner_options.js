const assert = require("assert");

async function testOptions() {
    const pkg = require("../npm/dist/index.js");

    assert.strictEqual(typeof pkg.TimeoutError, "function", "TimeoutError should be exported");
    assert.strictEqual(typeof pkg.AbortError, "function", "AbortError should be exported");
    const err = new pkg.TimeoutError("Execution timed out", "partial output", 150);
    assert.strictEqual(err.name, "TimeoutError");
    assert.strictEqual(err.stdout, "partial output");
    assert.strictEqual(err.elapsedMs, 150);
    assert.match(err.message, /timed out/);

    const abortErr = new pkg.AbortError("aborted output", 75);
    assert.strictEqual(abortErr.name, "AbortError");
    assert.ok(abortErr instanceof pkg.TimeoutError, "AbortError must inherit TimeoutError");
    assert.ok(abortErr instanceof pkg.AbortError, "AbortError must be instance of AbortError");
    assert.strictEqual(abortErr.stdout, "aborted output");
    assert.strictEqual(abortErr.elapsedMs, 75);

    // CompileError structured fields
    assert.strictEqual(typeof pkg.CompileError, "function", "CompileError should be exported");
    const compErr = new pkg.CompileError("main.cpp:42:15: error: expected ';' before '}' token");
    assert.strictEqual(compErr.file, "main.cpp");
    assert.strictEqual(compErr.line, 42);
    assert.strictEqual(compErr.column, 15);
    assert.match(compErr.message, /expected ';'/);

    // UTF-8 multi-byte stdout decoding (streaming split bytes)
    const hostUtf8 = pkg.createWasmHost({});
    const emojiBytes = [0x63, 0x61, 0x66, 0xc3, 0xa9, 0x20, 0xf0, 0x9f, 0x98, 0x80]; // "café 😀"
    for (const b of emojiBytes) {
        hostUtf8.imports.env.print_char(b);
    }
    assert.strictEqual(hostUtf8.getStdout(), "café 😀", "Streaming multi-byte UTF-8 bytes must decode to 'café 😀'");

    // UTF-8 stdin byte-by-byte
    const hostStdin = pkg.createWasmHost({ stdin: "日" }); // 0xE6 0x97 0xA5
    const stdinGot = [
        hostStdin.imports.env.read_char(),
        hostStdin.imports.env.read_char(),
        hostStdin.imports.env.read_char(),
        hostStdin.imports.env.read_char()
    ];
    assert.deepStrictEqual(stdinGot, [0xe6, 0x97, 0xa5, -1], "UTF-8 stdin must yield exact bytes followed by -1");

    // Flushed incomplete UTF-8 tail notifies onStdout
    const trailingChunks = [];
    const hostTrailing = pkg.createWasmHost({ onStdout: (c) => trailingChunks.push(c) });
    hostTrailing.imports.env.print_char(0xc3);
    assert.strictEqual(trailingChunks.length, 0, "Incomplete sequence must not be emitted mid-stream");
    const trailingOut = hostTrailing.getStdout();
    assert.strictEqual(trailingOut, "\uFFFD", "Flushed tail must return replacement character");
    assert.deepStrictEqual(trailingChunks, ["\uFFFD"], "onStdout must be notified of flushed tail character");

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
