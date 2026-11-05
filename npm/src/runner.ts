import type { WasmHost } from "./host.js";

/**
 * The outcome of running a compiled WebAssembly program.
 */
export interface RunResult {
	/** Everything the program wrote to stdout. */
	stdout: string;
	/** The program's exit status, or 134 if it trapped. */
	exitCode: number;
}

/**
 * Instantiates and executes a compiled cppbolt module against a host.
 *
 * The runner knows nothing about how the bytes were produced, so it works
 * equally with bytes from `CppBoltCompiler.compile()` and with a `.wasm` file
 * loaded from disk or fetched from a URL.
 */
export class CppBoltRunner {
	constructor(private readonly host: WasmHost) {}

	/**
	 * Instantiates `wasmBytes`, calls the program entry point (`main`, falling
	 * back to `_start`), and collects its stdout and exit status.
	 *
	 * A WebAssembly trap is reported as exit status 134 rather than thrown, so
	 * callers can inspect partial output the same way a shell would.
	 */
	async run(wasmBytes: Uint8Array): Promise<RunResult> {
		const { instance } = await WebAssembly.instantiate(
			wasmBytes as BufferSource,
			this.host.imports,
		);

		let exitCode = 0;
		try {
			const exports = instance.exports as {
				main?: () => number;
				_start?: () => number;
			};
			const fn = exports.main ?? exports._start;
			if (fn) exitCode = fn() ?? 0;
		} catch {
			// A WebAssembly trap — treat it like SIGABRT (exit 134), matching the
			// standalone host runner.
			return { stdout: this.host.getStdout(), exitCode: 134 };
		}

		return { stdout: this.host.getStdout(), exitCode };
	}
}
