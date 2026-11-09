import { registerAsyncRunnerFactory } from "./api.js";
import { AsyncCppBoltRunner } from "./browser-worker.js";
import { type CppBoltCompiler, loadCompilerFromResponse } from "./compiler.js";

export * from "./api.js";
export { AsyncCppBoltRunner } from "./browser-worker.js";

registerAsyncRunnerFactory((host) => new AsyncCppBoltRunner(host));

// Preserved at script evaluation time; document.currentScript is null during subsequent calls.
const scriptSrc: string | null =
	typeof document !== "undefined" &&
	document.currentScript instanceof HTMLScriptElement
		? document.currentScript.src
		: null;

/**
 * Instantiates a `CppBoltCompiler` in classic browser script environments.
 * Resolves `wasm/cppbolt.wasm` relative to the script URL.
 */
export async function createCompiler(): Promise<CppBoltCompiler> {
	if (!scriptSrc) {
		throw new Error(
			"cppbolt: could not locate wasm binary. " +
				"Load this script from a URL (CDN / local server) or use the Node.js package.",
		);
	}

	const wasmUrl = new URL("../wasm/cppbolt.wasm", scriptSrc).href;
	return loadCompilerFromResponse(await fetch(wasmUrl));
}
