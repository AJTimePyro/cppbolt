const assert = require("assert");
const fs = require("fs");
const path = require("path");

async function testBrowserBuild() {
    const browserMjsPath = path.resolve(__dirname, "../npm/dist/index.browser.mjs");
    const browserJsPath = path.resolve(__dirname, "../npm/dist/cppbolt.browser.js");

    assert.ok(fs.existsSync(browserMjsPath), "index.browser.mjs must exist");
    assert.ok(fs.existsSync(browserJsPath), "cppbolt.browser.js must exist");

    const mjsContent = fs.readFileSync(browserMjsPath, "utf8");
    const jsContent = fs.readFileSync(browserJsPath, "utf8");

    assert.ok(!mjsContent.includes("node:worker_threads"), "Browser MJS bundle must not import node:worker_threads");
    assert.ok(!jsContent.includes("node:worker_threads"), "Browser JS bundle must not import node:worker_threads");

    assert.ok(!mjsContent.includes("__WORKER_BUNDLE__"), "Browser MJS bundle must not contain unreplaced __WORKER_BUNDLE__");
    assert.ok(!jsContent.includes("__WORKER_BUNDLE__"), "Browser JS bundle must not contain unreplaced __WORKER_BUNDLE__");

    assert.ok(mjsContent.includes("createWasmHost"), "Browser MJS bundle must contain embedded worker with createWasmHost");
    assert.ok(jsContent.includes("createWasmHost"), "Browser JS bundle must contain embedded worker with createWasmHost");

    assert.match(mjsContent, /AsyncCppBoltRunner/, "AsyncCppBoltRunner should be in browser MJS bundle");
    assert.match(jsContent, /AsyncCppBoltRunner/, "AsyncCppBoltRunner should be in browser JS bundle");

    console.log("Browser build test passed!");
}

testBrowserBuild().catch((e) => {
    console.error("Test failed:", e);
    process.exit(1);
});
