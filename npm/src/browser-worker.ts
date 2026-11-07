import type { WasmHost } from "./host.js";
import { type RunOptions, type RunResult, TimeoutError } from "./runner.js";

export class AsyncCppBoltRunner {
	constructor(readonly host?: WasmHost) {}

	async run(wasmBytes: Uint8Array, options?: RunOptions): Promise<RunResult> {
		if (typeof Worker === "undefined") {
			throw new Error(
				"AsyncCppBoltRunner requires Web Worker support in this environment",
			);
		}

		const blob = new Blob([__WORKER_BUNDLE__], {
			type: "application/javascript",
		});
		const workerUrl = URL.createObjectURL(blob);
		let worker: Worker;
		try {
			worker = new Worker(workerUrl);
		} catch (err) {
			URL.revokeObjectURL(workerUrl);
			throw new Error(
				`Failed to construct Web Worker. If Content Security Policy (CSP) is active, ensure 'worker-src blob:' (or 'script-src blob:') is allowed: ${(err as Error).message}`,
			);
		}

		const started = performance.now();
		let accumulatedStdout = "";

		return new Promise<RunResult>((resolve, reject) => {
			let timer: ReturnType<typeof setTimeout> | null = null;
			let abortHandler: (() => void) | null = null;
			let settled = false;

			const cleanup = () => {
				if (settled) return;
				settled = true;
				if (timer !== null) clearTimeout(timer);
				if (options?.signal && abortHandler) {
					options.signal.removeEventListener("abort", abortHandler);
				}
				worker.terminate();
				URL.revokeObjectURL(workerUrl);
			};

			if (options?.timeoutMs && options.timeoutMs > 0) {
				timer = setTimeout(() => {
					cleanup();
					reject(
						new TimeoutError(
							`Execution timed out after ${options.timeoutMs}ms`,
							accumulatedStdout,
							options.timeoutMs,
						),
					);
				}, options.timeoutMs);
			}

			if (options?.signal) {
				if (options.signal.aborted) {
					cleanup();
					reject(
						new TimeoutError(
							"Execution aborted by signal",
							accumulatedStdout,
							0,
						),
					);
					return;
				}
				abortHandler = () => {
					cleanup();
					reject(
						new TimeoutError(
							"Execution aborted by signal",
							accumulatedStdout,
							performance.now() - started,
						),
					);
				};
				options.signal.addEventListener("abort", abortHandler);
			}

			worker.onmessage = (event: MessageEvent) => {
				const msg = event.data;
				if (msg.type === "stdout") {
					accumulatedStdout += msg.chunk;
					options?.onStdout?.(msg.chunk);
					return;
				}
				if (msg.type === "done") {
					cleanup();
					resolve({
						stdout: msg.stdout,
						exitCode: msg.exitCode,
						elapsedMs: msg.elapsedMs,
					});
					return;
				}
				if (msg.type === "error") {
					cleanup();
					reject(new Error(msg.error));
				}
			};

			worker.onerror = (err) => {
				err.preventDefault?.();
				cleanup();
				const msg =
					err.message ||
					"Web Worker execution failed. If using Content Security Policy, verify 'worker-src blob:' is allowed.";
				reject(new Error(msg));
			};

			worker.postMessage({ wasmBytes, stdin: options?.stdin ?? "" });
		});
	}
}
