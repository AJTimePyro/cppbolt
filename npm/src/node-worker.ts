import path from "node:path";
import { Worker } from "node:worker_threads";
import type { WasmHost } from "./host.js";
import { type RunOptions, type RunResult, TimeoutError } from "./runner.js";

export class AsyncCppBoltRunner {
	/**
	 * Executes WebAssembly modules inside a dedicated Node.js `worker_threads` instance.
	 * Host I/O configuration (stdin, streaming stdout) is passed per invocation via `RunOptions`.
	 */
	constructor(readonly host?: WasmHost) {}

	async run(wasmBytes: Uint8Array, options?: RunOptions): Promise<RunResult> {
		const workerPath = path.join(__dirname, "worker.node.js");
		const worker = new Worker(workerPath);
		const started = performance.now();
		let accumulatedStdout = "";

		return new Promise<RunResult>((resolve, reject) => {
			let timer: ReturnType<typeof setTimeout> | null = null;
			let abortHandler: (() => void) | null = null;
			let settled = false;

			const cleanup = () => {
				if (settled) return;
				settled = true;
				if (timer) clearTimeout(timer);
				if (options?.signal && abortHandler) {
					options.signal.removeEventListener("abort", abortHandler);
				}
				worker.terminate();
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

			worker.on("message", (msg) => {
				if (settled) return;
				if (msg.type === "stdout") {
					accumulatedStdout += msg.chunk;
					if (options?.onStdout) {
						try {
							options.onStdout(msg.chunk);
						} catch (err) {
							cleanup();
							reject(err instanceof Error ? err : new Error(String(err)));
						}
					}
					return;
				}
				if (msg.type === "done") {
					cleanup();
					resolve({
						stdout: accumulatedStdout,
						exitCode: msg.exitCode,
						elapsedMs: msg.elapsedMs,
					});
					return;
				}
				if (msg.type === "error") {
					cleanup();
					reject(new Error(msg.error));
				}
			});

			worker.on("error", (err) => {
				cleanup();
				reject(err);
			});

			worker.on("exit", (code) => {
				if (!settled) {
					cleanup();
					reject(
						new Error(`Worker stopped unexpectedly with exit code ${code}`),
					);
				}
			});

			worker.postMessage({ wasmBytes, stdin: options?.stdin ?? "" });
		});
	}
}
