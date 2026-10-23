// The playground's compiler worker. See docs/host-interface.md for the
// message protocol.

let compiler = null;
let compilerUrl = null;

self.onmessage = async (event) => {
    const { source, url, id } = event.data;
    try {
        let loadMs = null;
        if (!compiler || compilerUrl !== url) {
            const started = performance.now();
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`cannot fetch the compiler (${response.status})`);
            }
            const bytes = await response.arrayBuffer();
            const module = await WebAssembly.instantiate(bytes, {});
            compiler = module.instance.exports;
            compilerUrl = url;
            loadMs = performance.now() - started;
        }
        const sourceBytes = new TextEncoder().encode(source);
        const input = compiler.alloc_buffer(sourceBytes.length);
        new Uint8Array(compiler.memory.buffer).set(sourceBytes, input);
        const compiling = performance.now();
        const result = compiler.compile_wasm(input, sourceBytes.length);
        const compileMs = performance.now() - compiling;
        compiler.dealloc_buffer(input, sourceBytes.length);
        if (!result) {
            const length = compiler.get_error_len();
            const pointer = compiler.get_error_ptr();
            const diagnostic = new TextDecoder().decode(
                new Uint8Array(compiler.memory.buffer).slice(pointer, pointer + length)
            );
            self.postMessage({ id, ok: false, diagnostic, compileMs, loadMs });
            return;
        }
        const length = compiler.get_result_len();
        const bytes = new Uint8Array(compiler.memory.buffer).slice(result, result + length);
        compiler.free_result(result, length);
        self.postMessage({ id, ok: true, bytes: bytes.buffer, compileMs, loadMs }, [bytes.buffer]);
    } catch (error) {
        self.postMessage({
            id,
            ok: false,
            diagnostic: String(error && error.message ? error.message : error),
            compileMs: 0,
            loadMs: null,
        });
    }
};
