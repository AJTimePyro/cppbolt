import { createWasmHost, isWasmExit } from "../host.js";

export type WorkerMessage =
	| { type: "stdout"; chunk: string }
	| {
			type: "done";
			exitCode: number;
			stdout: string;
			elapsedMs: number;
			trap: string | null;
	  }
	| { type: "error"; error: string; elapsedMs: number };

export async function executeProgramInWorker(
	wasmBytes: Uint8Array,
	stdin: string,
	postMessage: (msg: WorkerMessage) => void,
): Promise<void> {
	const started = performance.now();
	try {
		let exitCode: number | null = null;

		const host = createWasmHost({
			stdin: stdin || "",
			onStdout: (chunk) => {
				postMessage({ type: "stdout", chunk });
			},
		});

		const { instance } = await WebAssembly.instantiate(
			wasmBytes as BufferSource,
			host.imports,
		);

		let trap: unknown = null;
		let returned = 0;

		try {
			const exports = instance.exports as {
				main?: () => number;
				_start?: () => number;
			};
			const fn = exports.main ?? exports._start;
			if (fn) {
				returned = fn();
			}
		} catch (error) {
			if (isWasmExit(error)) {
				exitCode = error.code;
			} else if (
				error instanceof WebAssembly.RuntimeError ||
				(error instanceof RangeError && /call stack/i.test(error.message))
			) {
				trap = error;
			} else {
				throw error;
			}
		}

		const elapsedMs = performance.now() - started;
		const code = trap ? 134 : (exitCode ?? returned ?? 0);

		postMessage({
			type: "done",
			exitCode: code,
			stdout: host.getStdout(),
			elapsedMs,
			trap: trap ? (trap instanceof Error ? trap.message : String(trap)) : null,
		});
	} catch (err: unknown) {
		postMessage({
			type: "error",
			error: err instanceof Error ? err.message : String(err),
			elapsedMs: performance.now() - started,
		});
	}
}
