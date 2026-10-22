// The compiler worker of the playground.
//
// The compiler runs in a worker so that a deep program never blocks the
// page.  The worker owns the WebAssembly build of the compiler and answers
// one request at a time: the source goes in, the compiled module or the
// diagnostic comes out.
//
// Every answer carries the time the step took, so the page can show how long
// the compilation was without the compiler having to measure anything.

let compiler = null;
let loader = null;

self.onmessage = async (event) => {
    const { source, loaderUrl, wasmUrl, id } = event.data;
    try {
        if (!loader) {
            // The glue of the `pkg` directory, next to the module.
            importScripts(loaderUrl);
            loader = self.CppWasmCompiler;
        }
        let loadMs = null;
        if (!compiler) {
            const started = performance.now();
            compiler = await loader.init(wasmUrl);
            loadMs = performance.now() - started;
        }
        const compiling = performance.now();
        const result = compiler.compile(source);
        const compileMs = performance.now() - compiling;
        if (!result.ok) {
            self.postMessage({
                id,
                ok: false,
                diagnostic: result.diagnostic,
                compileMs,
                loadMs,
            });
            return;
        }
        const bytes = result.bytes;
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
