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
 * Instantiates a `CppBoltCompiler` in Node.js environments.
 * Resolves `wasm/cppbolt.wasm` from the package directory by default, or loads from a
 * specified file path, URL, or preloaded `BufferSource`.
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
