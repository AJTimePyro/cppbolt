// The JavaScript glue of the WebAssembly build of the compiler.
//
// The module is the WebAssembly build of the compiler with the C ABI of
// src/ffi.rs.  This directory has the layout of a `pkg` directory: this file
// is the glue, `cppwasm_bg.wasm` is the module, and `cppwasm.d.ts` describes
// the API.
//
// The compiler has no dependencies, so there is no wasm-bindgen glue: the
// eight exported functions of the C ABI are the whole interface, and this
// file wraps them in one call that returns either the module or the
// diagnostic.
(function (root, factory) {
    if (typeof module === "object" && module.exports) {
        module.exports = factory();
    } else {
        root.CppWasmCompiler = factory();
    }
}(typeof self !== "undefined" ? self : this, function () {
    "use strict";

    const encoder = new TextEncoder();
    const decoder = new TextDecoder();

    /// Reads the bytes of the compiler module.
    async function readBytes(source) {
        if (source instanceof ArrayBuffer) {
            return source;
        }
        if (ArrayBuffer.isView(source)) {
            return source.buffer.slice(source.byteOffset, source.byteOffset + source.byteLength);
        }
        const response = await fetch(source);
        if (!response.ok) {
            throw new Error(`cannot fetch the compiler (${response.status})`);
        }
        return response.arrayBuffer();
    }

    /// Loads the compiler and returns its API.
    ///
    /// The source is the URL of the module or its bytes.
    async function init(source) {
        const bytes = await readBytes(source);
        const module = await WebAssembly.instantiate(bytes, {});
        const api = module.instance.exports;

        return {
            /// Compiles a C++ source.
            ///
            /// The result holds the bytes of the module when the compilation
            /// succeeded, and the diagnostic when it did not.
            compile(source) {
                const sourceBytes = encoder.encode(source);
                const input = api.alloc_buffer(sourceBytes.length);
                new Uint8Array(api.memory.buffer).set(sourceBytes, input);
                const result = api.compile_wasm(input, sourceBytes.length);
                api.dealloc_buffer(input, sourceBytes.length);
                if (!result) {
                    const length = api.get_error_len();
                    const pointer = api.get_error_ptr();
                    const diagnostic = decoder.decode(
                        new Uint8Array(api.memory.buffer).slice(pointer, pointer + length)
                    );
                    return { ok: false, diagnostic };
                }
                const length = api.get_result_len();
                const compiled = new Uint8Array(api.memory.buffer).slice(result, result + length);
                api.free_result(result, length);
                return { ok: true, bytes: compiled };
            },
        };
    }

    return { init };
}));
