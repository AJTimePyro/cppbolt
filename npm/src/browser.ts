import { registerAsyncRunnerFactory } from "./api.js";
import { AsyncCppBoltRunner } from "./browser-worker.js";
import { type CppBoltCompiler, loadCompilerFromResponse } from "./compiler.js";

export * from "./api.js";
export { AsyncCppBoltRunner } from "./browser-worker.js";

registerAsyncRunnerFactory((host) => new AsyncCppBoltRunner(host));

/**
 * Instantiates a `CppBoltCompiler` in browser ES module and bundler environments.
 * Resolves `wasm/cppbolt.wasm` relative to the module URL by default, or fetches from `wasmUrl`.
 */
export async function createCompiler(
	wasmUrl: string | URL = new URL("../wasm/cppbolt.wasm", import.meta.url),
): Promise<CppBoltCompiler> {
	return loadCompilerFromResponse(await fetch(wasmUrl));
}
