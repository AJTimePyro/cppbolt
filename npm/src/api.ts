import type { CppBoltCompiler } from "./compiler.js";
import { createWasmHost, type WasmHost, type WasmHostOptions } from "./host.js";
import { CppBoltRunner, type RunOptions, type RunResult } from "./runner.js";

// Shared public surface, re-exported by each environment entry point.
export { CompileError, CppBoltCompiler } from "./compiler.js";
export type { WasmHost, WasmHostOptions } from "./host.js";
export { createWasmHost, isWasmExit, WasmExit } from "./host.js";
export type { RunOptions, RunResult } from "./runner.js";
export { CppBoltRunner, TimeoutError } from "./runner.js";

/** Runner interface implemented by sync and async runners. */
export interface RunnerInstance {
	run(wasmBytes: Uint8Array, options?: RunOptions): Promise<RunResult>;
}

export type AsyncRunnerFactory = (host: WasmHost) => RunnerInstance;
let defaultAsyncRunnerFactory: AsyncRunnerFactory | null = null;

export function registerAsyncRunnerFactory(factory: AsyncRunnerFactory): void {
	defaultAsyncRunnerFactory = factory;
}

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

/**
 * Compiles `source` and executes the module in an isolated worker with timeout support.
 */
export async function compileAndRunAsync(
	compiler: CppBoltCompiler,
	source: string,
	options?: WasmHostOptions & RunOptions,
): Promise<RunResult> {
	const host = createWasmHost(options);
	const bytes = compiler.compile(source);
	if (!defaultAsyncRunnerFactory) {
		throw new Error(
			"No AsyncRunner registered for this environment. Import from 'cppbolt' (Node) or 'cppbolt/browser' (Browser).",
		);
	}
	const runner = defaultAsyncRunnerFactory(host);
	return runner.run(bytes, options);
}
