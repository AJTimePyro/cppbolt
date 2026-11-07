import { parentPort } from "node:worker_threads";
import { executeProgramInWorker } from "./worker-core.js";

if (!parentPort) {
	throw new Error("node-worker.entry must run inside a worker_thread");
}

const port = parentPort;
port.on(
	"message",
	({ wasmBytes, stdin }: { wasmBytes: Uint8Array; stdin?: string }) => {
		executeProgramInWorker(wasmBytes, stdin || "", (msg) => {
			port.postMessage(msg);
		});
	},
);
