import { executeProgramInWorker } from "./worker-core.js";

self.onmessage = (event: MessageEvent) => {
	const { wasmBytes, stdin } = event.data;
	executeProgramInWorker(wasmBytes, stdin || "", (msg) => {
		self.postMessage(msg);
	});
};
