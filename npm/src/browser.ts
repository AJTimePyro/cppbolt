import { registerAsyncRunnerFactory } from "./api.js";
import { AsyncCppBoltRunner } from "./browser-worker.js";
import { type CppBoltCompiler, loadCompilerFromResponse } from "./compiler.js";

export * from "./api.js";
export { AsyncCppBoltRunner } from "./browser-worker.js";

registerAsyncRunnerFactory((host) => new AsyncCppBoltRunner(host));

/**
 * Fetch cppbolt.wasm and return a CppBoltCompiler.
 *
 * This is the browser ES module entry point. By default the wasm binary is
 * resolved relative to this module, which works both from a CDN and through a
 * bundler that understands `new URL(..., import.meta.url)`. Pass `wasmUrl` to
 * load it from somewhere else.
 */
export async function createCompiler(
	wasmUrl: string | URL = new URL("../wasm/cppbolt.wasm", import.meta.url),
): Promise<CppBoltCompiler> {
	return loadCompilerFromResponse(await fetch(wasmUrl));
}
