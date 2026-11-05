import fs from "node:fs";
import path from "node:path";
import { type CppBoltCompiler, loadCompilerFromBytes } from "./compiler.js";

export * from "./api.js";

/**
 * Load cppbolt.wasm from the package directory and return a CppBoltCompiler.
 *
 * This is the Node.js entry point. Browsers use `index.browser.mjs` (ES
 * modules / bundlers) or `cppbolt.browser.js` (classic `<script>` tag).
 */
export async function createCompiler(): Promise<CppBoltCompiler> {
	const wasmPath = path.join(__dirname, "..", "wasm", "cppbolt.wasm");

	if (!fs.existsSync(wasmPath)) {
		throw new Error(`cppbolt: could not find wasm binary at ${wasmPath}`);
	}

	return loadCompilerFromBytes(fs.readFileSync(wasmPath));
}
