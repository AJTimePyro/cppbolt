const assert = require("assert");

async function testCompileAndRunAsync() {
    const pkg = require("../npm/dist/index.js");

    assert.strictEqual(typeof pkg.compileAndRunAsync, "function", "compileAndRunAsync should be exported");

    const compiler = {
        compile(source) {
            if (source.includes("infinite")) {
                return new Uint8Array([
                    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
                    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
                    0x03, 0x02, 0x01, 0x00,
                    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
                    0x0a, 0x0b, 0x01, 0x09, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b
                ]);
            }
            return new Uint8Array([
                0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
                0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
                0x03, 0x02, 0x01, 0x00,
                0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
                0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b
            ]);
        }
    };

    const res = await pkg.compileAndRunAsync(compiler, "int main() { return 42; }");
    assert.strictEqual(res.exitCode, 42);

    let caught = false;
    try {
        await pkg.compileAndRunAsync(compiler, "int main() { while(infinite); }", { timeoutMs: 120 });
    } catch (e) {
        caught = true;
        assert.ok(e instanceof pkg.TimeoutError);
    }
    assert.strictEqual(caught, true, "Infinite execution should time out");

    console.log("compileAndRunAsync test passed!");
    process.exit(0);
}

testCompileAndRunAsync().catch((e) => {
    console.error("Test failed:", e);
    process.exit(1);
});
