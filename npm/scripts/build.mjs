import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const tscBin = path.join(root, "node_modules", "typescript", "bin", "tsc");

async function build() {
	const repoRoot = path.resolve(root, "..");
	const licenseSrc = path.join(repoRoot, "LICENSE");
	if (fs.existsSync(licenseSrc)) {
		fs.copyFileSync(licenseSrc, path.join(root, "LICENSE"));
	}

	execFileSync(process.execPath, [tscBin, "-p", "tsconfig.json"], {
		cwd: root,
		stdio: "inherit",
	});
	execFileSync(process.execPath, [tscBin, "-p", "tsconfig.browser.json"], {
		cwd: root,
		stdio: "inherit",
	});

	await esbuild.build({
		entryPoints: [path.join(root, "src/worker/node-worker.entry.ts")],
		outfile: path.join(root, "dist/worker.node.js"),
		bundle: true,
		platform: "node",
		format: "cjs",
		target: "node18",
	});

	const workerBuild = await esbuild.build({
		entryPoints: [path.join(root, "src/worker/browser-worker.entry.ts")],
		bundle: true,
		format: "iife",
		platform: "browser",
		target: "es2020",
		minify: true,
		write: false,
	});
	const workerBundle = JSON.stringify(workerBuild.outputFiles[0].text);

	await esbuild.build({
		entryPoints: [path.join(root, "src/browser.ts")],
		outfile: path.join(root, "dist/index.browser.mjs"),
		bundle: true,
		format: "esm",
		platform: "browser",
		target: "es2020",
		define: {
			__WORKER_BUNDLE__: workerBundle,
		},
	});

	await esbuild.build({
		entryPoints: [path.join(root, "src/script.ts")],
		outfile: path.join(root, "dist/cppbolt.browser.js"),
		bundle: true,
		format: "iife",
		globalName: "CppBolt",
		platform: "browser",
		target: "es2020",
		define: {
			__WORKER_BUNDLE__: workerBundle,
		},
	});
}

build().catch((err) => {
	console.error(err);
	process.exit(1);
});
