/**
 * The WebAssembly build of the C++ to WebAssembly compiler.
 *
 * The module is the compiler with the C ABI of src/ffi.rs, wrapped in one
 * call that returns either the compiled module or the diagnostic.
 */

/** The outcome of a compilation. */
export type CompileResult =
    | { ok: true; bytes: Uint8Array }
    | { ok: false; diagnostic: string };

/** The API of the compiler. */
export interface Compiler {
    /** Compiles a C++ source into a WebAssembly module. */
    compile(source: string): CompileResult;
}

/**
 * Loads the compiler.
 *
 * The source is the URL of `cppwasm_bg.wasm` or its bytes.
 */
export function init(source: string | ArrayBuffer | ArrayBufferView): Promise<Compiler>;
