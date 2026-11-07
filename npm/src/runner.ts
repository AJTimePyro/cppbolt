import { isWasmExit, type WasmHost } from "./host.js";

/**
 * The outcome of running a compiled WebAssembly program.
 */
export interface RunResult {
	/** Everything the program wrote to stdout. */
	stdout: string;
	/** The program's exit status, or 134 if it trapped. */
	exitCode: number;
	/** Execution time in milliseconds if measured. */
	elapsedMs?: number;
}

/**
 * Options configuring program execution.
 */
export interface RunOptions {
	/** Optional execution timeout in milliseconds. Supported only by `AsyncCppBoltRunner`. */
	timeoutMs?: number;
	/** Optional AbortSignal to cancel execution prematurely. Supported only by `AsyncCppBoltRunner`. */
	signal?: AbortSignal;
	/** Optional stdin string forwarded to the program. */
	stdin?: string;
	/** Optional callback invoked with each chunk written to stdout. */
	onStdout?: (chunk: string) => void;
}

/**
 * Error thrown when execution exceeds timeout limit or is cancelled.
 */
export class TimeoutError extends Error {
	readonly stdout: string;
	readonly elapsedMs: number;

	constructor(message: string, stdout = "", elapsedMs = 0) {
		super(message);
		this.name = "TimeoutError";
		this.stdout = stdout;
		this.elapsedMs = elapsedMs;
	}
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
	 *
	 * Note: Synchronous `CppBoltRunner` executes on the current thread and does not
	 * support watchdog `timeoutMs` or cancellation via `signal`. For watchdog timeouts
	 * and cancellation, use `AsyncCppBoltRunner` or `compileAndRunAsync`.
	 */
	async run(wasmBytes: Uint8Array, _options?: RunOptions): Promise<RunResult> {
		const started = performance.now();
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
		} catch (error) {
			const elapsedMs = performance.now() - started;
			if (isWasmExit(error)) {
				return {
					stdout: this.host.getStdout(),
					exitCode: error.code,
					elapsedMs,
				};
			}
			// A WebAssembly trap, treated like SIGABRT (exit 134), matching
			// the standalone host runner. V8 reports a wasm stack overflow as a
			// RangeError rather than a WebAssembly.RuntimeError.
			if (
				error instanceof WebAssembly.RuntimeError ||
				(error instanceof RangeError && /call stack/i.test(error.message))
			) {
				return { stdout: this.host.getStdout(), exitCode: 134, elapsedMs };
			}
			throw error;
		}

		return {
			stdout: this.host.getStdout(),
			exitCode,
			elapsedMs: performance.now() - started,
		};
	}
}
