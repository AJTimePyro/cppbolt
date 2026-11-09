import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { registerAsyncRunnerFactory } from "./api.js";
import {
	type CppBoltCompiler,
	loadCompilerFromBytes,
	loadCompilerFromResponse,
} from "./compiler.js";
import { AsyncCppBoltRunner } from "./node-worker.js";

export * from "./api.js";
export { AsyncCppBoltRunner } from "./node-worker.js";

registerAsyncRunnerFactory((host) => new AsyncCppBoltRunner(host));

/**
 * Load cppbolt.wasm and return a CppBoltCompiler.
 *
 * Defaults to `wasm/cppbolt.wasm` in the package directory. An optional file
 * path, file/http URL, or WebAssembly bytecode buffer can be provided.
 */
export async function createCompiler(
	wasmLocation?: string | URL | BufferSource,
): Promise<CppBoltCompiler> {
	if (wasmLocation instanceof ArrayBuffer || ArrayBuffer.isView(wasmLocation)) {
		return loadCompilerFromBytes(wasmLocation as BufferSource);
	}

	if (wasmLocation) {
		const locStr = wasmLocation.toString();
		if (locStr.startsWith("http://") || locStr.startsWith("https://")) {
			return loadCompilerFromResponse(await fetch(wasmLocation));
		}
		const filePath =
			wasmLocation instanceof URL ? fileURLToPath(wasmLocation) : wasmLocation;
		if (!fs.existsSync(filePath)) {
			throw new Error(`cppbolt: could not find wasm binary at ${filePath}`);
		}
		return loadCompilerFromBytes(fs.readFileSync(filePath));
	}

	const wasmPath = path.join(__dirname, "..", "wasm", "cppbolt.wasm");

	if (!fs.existsSync(wasmPath)) {
		throw new Error(`cppbolt: could not find wasm binary at ${wasmPath}`);
	}

	return loadCompilerFromBytes(fs.readFileSync(wasmPath));
}
