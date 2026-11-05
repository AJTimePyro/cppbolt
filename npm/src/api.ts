import type { CppBoltCompiler } from "./compiler.js";
import { createWasmHost, type WasmHostOptions } from "./host.js";
import { CppBoltRunner, type RunResult } from "./runner.js";

// Shared public surface, re-exported by each environment entry point.
export { CompileError, CppBoltCompiler } from "./compiler.js";
export type { WasmHost, WasmHostOptions } from "./host.js";
export { createWasmHost } from "./host.js";
export type { RunResult } from "./runner.js";
export { CppBoltRunner } from "./runner.js";

/**
 * Compiles `source` and runs the resulting module in one call.
 *
 * Builds a fresh host for the run, so streaming output and stdin are configured
 * through `options`. Throws `CompileError` if `source` does not compile.
 */
export async function compileAndRun(
	compiler: CppBoltCompiler,
	source: string,
	options?: WasmHostOptions,
): Promise<RunResult> {
	const host = createWasmHost(options);
	const runner = new CppBoltRunner(host);
	const bytes = compiler.compile(source);
	return runner.run(bytes);
}
