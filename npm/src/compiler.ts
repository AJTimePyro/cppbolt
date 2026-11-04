function tryRequire<T>(id: string): T | null {
  if (typeof require !== "function") return null;
  try {
    return require(id);
  } catch {
    return null;
  }
}

const fs = tryRequire<typeof import("fs")>("fs");
const path = tryRequire<typeof import("path")>("path");

const scriptSrc: string | null =
  typeof document !== "undefined" &&
  document.currentScript instanceof HTMLScriptElement
    ? document.currentScript.src
    : null;

function decodeString(
  memory: WebAssembly.Memory,
  ptr: number,
  len: number,
): string {
  return new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
}

export class CompileError extends Error {
  readonly diagnostic: string;

  constructor(diagnostic: string) {
    super(diagnostic);
    this.name = "CompileError";
    this.diagnostic = diagnostic;
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
 * Load cppbolt.wasm and return a CppBoltCompiler.
 *
 * In Node.js, resolves wasm/cppbolt.wasm from the package directory.
 * In a browser <script> tag, fetches it relative to the script's own URL.
 */
export async function createCompiler(): Promise<CppBoltCompiler> {
  if (fs && path && typeof __dirname === "string") {
    const localPath = path.join(__dirname, "..", "wasm", "cppbolt.wasm");
    if (fs.existsSync(localPath)) {
      return new CppBoltCompiler(
        await loadFromBytes(fs.readFileSync(localPath)),
      );
    }
  }

  const wasmUrl = scriptSrc
    ? new URL("../wasm/cppbolt.wasm", scriptSrc).href
    : null;
  if (wasmUrl) {
    return new CppBoltCompiler(await loadFromResponse(await fetch(wasmUrl)));
  }

  throw new Error(
    "cppbolt: could not locate wasm binary. " +
      "Load this script from a URL (CDN / local server) or use the Node.js package.",
  );
}

async function loadFromResponse(
  response: Response,
): Promise<WebAssembly.Instance> {
  if (!response.ok) {
    throw new Error(`cppbolt: failed to fetch wasm (HTTP ${response.status})`);
  }
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/wasm")) {
    const { instance } = await WebAssembly.instantiateStreaming(response, {});
    return instance;
  }
  return loadFromBytes(await response.arrayBuffer());
}

async function loadFromBytes(
  bytes: BufferSource,
): Promise<WebAssembly.Instance> {
  const { instance } = await WebAssembly.instantiate(bytes, {});
  return instance;
}
