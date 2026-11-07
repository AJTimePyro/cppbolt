import { spawn } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const browserBundlePath = path.join(root, "npm", "dist", "cppbolt.browser.js");

if (!fs.existsSync(browserBundlePath)) {
	console.error(`Browser bundle not found at ${browserBundlePath}. Run 'npm run build' first.`);
	process.exit(1);
}

const CSP_FRAME_HTML = `<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body>
<script src="/dist/cppbolt.browser.js"></script>
<script>
(async () => {
    let errorMsg = null;
    try {
        const host = window.CppBolt.createWasmHost({});
        const runner = new window.CppBolt.AsyncCppBoltRunner(host);
        await runner.run(new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]));
    } catch (err) {
        errorMsg = err.message || String(err);
    }
    window.parent.postMessage({ type: "csp-result", errorMsg }, "*");
})();
</script>
</body>
</html>`;

const HTML_CONTENT = `<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Browser Worker Test</title></head>
<body>
<script src="/dist/cppbolt.browser.js"></script>
<script>
const infiniteLoopWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x0a, 0x0b, 0x01, 0x09, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b
]);

const normalWasmBytes = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b
]);

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

async function runTests() {
    const results = [];
    try {
        const CppBolt = window.CppBolt;
        if (!CppBolt || typeof CppBolt.AsyncCppBoltRunner !== "function") {
            throw new Error("CppBolt.AsyncCppBoltRunner is not exported");
        }

        const host = CppBolt.createWasmHost({});
        const runner = new CppBolt.AsyncCppBoltRunner(host);

        // Test 1: Normal execution
        const res1 = await runner.run(normalWasmBytes, { timeoutMs: 1000 });
        if (res1.exitCode !== 42) {
            throw new Error("Expected exitCode 42, got " + res1.exitCode);
        }
        results.push("Normal execution: PASS");

        // Test 2: Infinite loop timeout
        let threwTimeout = false;
        try {
            await runner.run(infiniteLoopWasmBytes, { timeoutMs: 150 });
        } catch (err) {
            threwTimeout = true;
            if (!err.message.includes("timed out")) {
                throw new Error("Expected timeout message, got: " + err.message);
            }
        }
        if (!threwTimeout) {
            throw new Error("Infinite loop did not time out");
        }
        results.push("Timeout execution: PASS");

        // Test 3: Abort execution
        const controller = new AbortController();
        setTimeout(() => controller.abort(), 80);
        let threwAbort = false;
        try {
            await runner.run(infiniteLoopWasmBytes, { signal: controller.signal });
        } catch (err) {
            threwAbort = true;
            if (!err.message.includes("aborted")) {
                throw new Error("Expected abort message, got: " + err.message);
            }
        }
        if (!threwAbort) {
            throw new Error("Aborted execution did not throw");
        }
        results.push("Abort execution: PASS");

        // Test 4: CSP restriction diagnostics
        const cspResult = await new Promise((resolve) => {
            window.addEventListener("message", function onMsg(e) {
                if (e.data && e.data.type === "csp-result") {
                    window.removeEventListener("message", onMsg);
                    resolve(e.data);
                }
            });
            const iframe = document.createElement("iframe");
            iframe.src = "/csp-frame.html";
            document.body.appendChild(iframe);
        });

        if (!cspResult.errorMsg || !cspResult.errorMsg.includes("worker-src blob:")) {
            throw new Error("Expected CSP diagnostic mentioning 'worker-src blob:', got: " + cspResult.errorMsg);
        }
        results.push("CSP restriction diagnostics: PASS");

        // Test 5: Rejection of invalid wasm
        let threwInvalid = false;
        try {
            await runner.run(new Uint8Array([0, 1, 2, 3]));
        } catch (err) {
            threwInvalid = true;
        }
        if (!threwInvalid) {
            throw new Error("Invalid wasm did not reject promise");
        }
        results.push("Invalid wasm rejection: PASS");

        // Test 6: Optional host constructor
        const runnerNoHost = new CppBolt.AsyncCppBoltRunner();
        const resNoHost = await runnerNoHost.run(normalWasmBytes);
        if (resNoHost.exitCode !== 42) {
            throw new Error("No-host runner failed with exitCode: " + resNoHost.exitCode);
        }
        results.push("Optional host constructor: PASS");

        // Test 7: Stdin forwarding
        const resStdin = await runner.run(readStdinWasmBytes, { stdin: "Z" });
        if (resStdin.exitCode !== 90) {
            throw new Error("Expected stdin exitCode 90, got: " + resStdin.exitCode);
        }
        results.push("Stdin forwarding: PASS");

        // Test 8: Streaming onStdout callback
        const chunks = [];
        const resStdout = await runner.run(printStdoutWasmBytes, {
            onStdout: (c) => chunks.push(c)
        });
        if (resStdout.exitCode !== 0 || chunks.join("") !== "?") {
            throw new Error("Streaming onStdout failed. Chunks: " + chunks.join(""));
        }
        results.push("Streaming onStdout: PASS");

        await fetch("/report", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ ok: true, results })
        });
    } catch (err) {
        await fetch("/report", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ ok: false, error: err.message || String(err), results })
        });
    }
}

window.addEventListener("load", runTests);
</script>
</body>
</html>`;

async function main() {
	let resolveReport;
	let rejectReport;
	const reportPromise = new Promise((resolve, reject) => {
		resolveReport = resolve;
		rejectReport = reject;
	});

	const server = http.createServer((req, res) => {
		if (req.url === "/" || req.url === "/index.html") {
			res.writeHead(200, { "Content-Type": "text/html" });
			res.end(HTML_CONTENT);
		} else if (req.url === "/csp-frame.html") {
			res.writeHead(200, {
				"Content-Type": "text/html",
				"Content-Security-Policy": "default-src 'self' 'unsafe-inline'; worker-src 'none'",
			});
			res.end(CSP_FRAME_HTML);
		} else if (req.url === "/dist/cppbolt.browser.js") {
			res.writeHead(200, { "Content-Type": "application/javascript" });
			res.end(fs.readFileSync(browserBundlePath));
		} else if (req.url === "/report" && req.method === "POST") {
			let body = "";
			req.on("data", (chunk) => {
				body += chunk;
			});
			req.on("end", () => {
				res.writeHead(200);
				res.end("OK");
				try {
					resolveReport(JSON.parse(body));
				} catch (e) {
					rejectReport(e);
				}
			});
		} else {
			res.writeHead(404);
			res.end("Not Found");
		}
	});

	await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
	const port = server.address().port;
	const testUrl = `http://127.0.0.1:${port}/`;

	const tempProfile = path.join(os.tmpdir(), `chromium-worker-test-${Date.now()}`);
	fs.mkdirSync(tempProfile, { recursive: true });

	const chromeProcess = spawn("chromium", [
		"--headless=new",
		"--no-sandbox",
		"--disable-gpu",
		`--user-data-dir=${tempProfile}`,
		testUrl,
	], { stdio: "ignore" });

	chromeProcess.on("error", (err) => {
		rejectReport(new Error(`Failed to launch Chromium: ${err.message}`));
	});

	const timeoutTimer = setTimeout(() => {
		rejectReport(new Error("Browser test timed out waiting for report"));
	}, 15000);

	try {
		const report = await reportPromise;
		clearTimeout(timeoutTimer);
		if (!report.ok) {
			console.error("Browser worker test failed:", report.error);
			process.exit(1);
		}
		console.log("Browser worker tests passed in Chromium:");
		for (const r of report.results) {
			console.log("  -", r);
		}
	} finally {
		clearTimeout(timeoutTimer);
		chromeProcess.kill();
		server.close();
		try {
			fs.rmSync(tempProfile, { recursive: true, force: true });
		} catch {}
	}
}

main().catch((err) => {
	console.error("Browser test failed:", err);
	process.exit(1);
});
