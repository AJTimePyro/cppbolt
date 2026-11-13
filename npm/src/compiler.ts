function decodeString(
	memory: WebAssembly.Memory,
	ptr: number,
	len: number,
): string {
	return new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
}

export class CompileError extends Error {
	readonly diagnostic: string;
	/** Source file reported in the diagnostic, if the message starts with one. */
	readonly file?: string;
	/** 1-based line number reported in the diagnostic, if present. */
	readonly line?: number;
	/** 1-based column number reported in the diagnostic, if present. */
	readonly column?: number;

	constructor(diagnostic: string) {
		super(diagnostic);
		this.name = "CompileError";
		this.diagnostic = diagnostic;
		const match = /^([^:]+):(\d+):(\d+):/.exec(diagnostic);
		if (match) {
			this.file = match[1];
			this.line = Number(match[2]);
			this.column = Number(match[3]);
		}
	}
}

interface CompilerExports extends WebAssembly.Exports {
	memory: WebAssembly.Memory;
	alloc_buffer(size: number): number;
	dealloc_buffer(ptr: number, size: number): void;
	compile_wasm(ptr: number, size: number): number;
	get_error_ptr(): number;
	get_error_len(): number;
	get_result_len(): number;
	free_result(ptr: number, size: number): void;
}

export class CppBoltCompiler {
	private readonly _exports: CompilerExports;

	constructor(instance: WebAssembly.Instance) {
		this._exports = instance.exports as CompilerExports;
	}

	compile(source: string): Uint8Array {
		const ex = this._exports;
		const mem = ex.memory;

		const srcBytes = new TextEncoder().encode(source);
		const srcPtr = ex.alloc_buffer(srcBytes.length);
		new Uint8Array(mem.buffer, srcPtr, srcBytes.length).set(srcBytes);

		const resultPtr = ex.compile_wasm(srcPtr, srcBytes.length);
		ex.dealloc_buffer(srcPtr, srcBytes.length);

		if (resultPtr === 0) {
			throw new CompileError(
				decodeString(mem, ex.get_error_ptr(), ex.get_error_len()),
			);
		}

		const resultLen = ex.get_result_len();
		const output = new Uint8Array(mem.buffer, resultPtr, resultLen).slice();
		ex.free_result(resultPtr, resultLen);
		return output;
	}
}

/**
 * Instantiates a compiler from raw wasm bytes.
 *
 * Entry points use this after reading the wasm binary from disk (Node.js) or
 * fetching it (browser).
 */
export async function loadCompilerFromBytes(
	bytes: BufferSource,
): Promise<CppBoltCompiler> {
	const { instance } = await WebAssembly.instantiate(bytes, {});
	return new CppBoltCompiler(instance);
}

/**
 * Instantiates a compiler from an HTTP response.
 *
 * Streams the module when the server sends `application/wasm`, so the binary
 * is compiled while it downloads.
 */
export async function loadCompilerFromResponse(
	response: Response,
): Promise<CppBoltCompiler> {
	if (!response.ok) {
		throw new Error(`cppbolt: failed to fetch wasm (HTTP ${response.status})`);
	}
	const contentType = response.headers.get("content-type") ?? "";
	if (contentType.includes("application/wasm")) {
		const { instance } = await WebAssembly.instantiateStreaming(response, {});
		return new CppBoltCompiler(instance);
	}
	return loadCompilerFromBytes(await response.arrayBuffer());
}
