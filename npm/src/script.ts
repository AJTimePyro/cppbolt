import { type CppBoltCompiler, loadCompilerFromResponse } from "./compiler.js";

export * from "./api.js";

// Captured while the classic script is evaluating: by the time callers invoke
// createCompiler(), document.currentScript is null again.
const scriptSrc: string | null =
	typeof document !== "undefined" &&
	document.currentScript instanceof HTMLScriptElement
		? document.currentScript.src
		: null;

/**
 * Fetch cppbolt.wasm and return a CppBoltCompiler.
 *
 * This is the classic `<script>` entry point. The wasm binary is resolved
 * relative to the script URL.
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
