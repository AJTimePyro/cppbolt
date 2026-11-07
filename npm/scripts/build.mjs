import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const tscBin = path.join(root, "node_modules", "typescript", "bin", "tsc");

async function build() {
	// 1. Run TypeScript declaration and CJS compilation
	execFileSync(process.execPath, [tscBin, "-p", "tsconfig.json"], {
		cwd: root,
		stdio: "inherit",
	});
	execFileSync(process.execPath, [tscBin, "-p", "tsconfig.browser.json"], {
		cwd: root,
		stdio: "inherit",
	});

	// 2. Bundle Node worker into dist/worker.node.js (CommonJS, node platform)
	await esbuild.build({
		entryPoints: [path.join(root, "src/worker/node-worker.entry.ts")],
		outfile: path.join(root, "dist/worker.node.js"),
		bundle: true,
		platform: "node",
		format: "cjs",
		target: "node18",
		minify: false,
	});

	// 3. Bundle Browser worker entry in memory (IIFE, browser platform, minified)
	const workerBuild = await esbuild.build({
		entryPoints: [path.join(root, "src/worker/browser-worker.entry.ts")],
		bundle: true,
		format: "iife",
		platform: "browser",
		target: "es2020",
		minify: true,
		write: false,
	});
	const workerBundleString = workerBuild.outputFiles[0].text;

	// 4. Bundle browser ESM (dist/index.browser.mjs) injecting __WORKER_BUNDLE__
	await esbuild.build({
		entryPoints: [path.join(root, "src/browser.ts")],
		outfile: path.join(root, "dist/index.browser.mjs"),
		bundle: true,
		format: "esm",
		platform: "browser",
		target: "es2020",
		define: {
			__WORKER_BUNDLE__: JSON.stringify(workerBundleString),
		},
	});

	// 5. Bundle browser CDN IIFE (dist/cppbolt.browser.js) injecting __WORKER_BUNDLE__
	await esbuild.build({
		entryPoints: [path.join(root, "src/script.ts")],
		outfile: path.join(root, "dist/cppbolt.browser.js"),
		bundle: true,
		format: "iife",
		globalName: "CppBolt",
		platform: "browser",
		target: "es2020",
		define: {
			__WORKER_BUNDLE__: JSON.stringify(workerBundleString),
		},
	});
}

build().catch((err) => {
	console.error(err);
	process.exit(1);
});
