# Host interface

`web/host.js` supplies the generated program imports: character output, integer output, floating output, character
input, and process exit. It buffers stdout and implements the compiler's number-formatting contract. Hosts use the
same shim in Node and in browsers; the browser must keep compiler execution off the UI thread.

`print_char(ch)` returns nothing. `print_int(value, radix, width, precision, flags)` and
`print_double(value, width, precision, mode, flags)` return the number of characters they wrote, including any
sign and field padding; the generated `printf` adds that count to its own return value. For `print_int`, flags
are bit 0 zero-pad, bit 1 left-align, bit 2 show `+`, bit 3 space sign, bit 4 uppercase, bit 5 unsigned, and
bit 6 alternate form. For `print_double`, modes are 0 general, 1 fixed, and 2 scientific; flags are bit 0
zero-pad, bit 1 left-align, bit 2 show `+`, bit 3 space sign, bit 4 uppercase, and bit 6 alternate form.

## Compiler WebAssembly ABI

The compiler module exports the following C entry points. Each compiled result is an independent boxed slice, so
hosts allocate inputs with `alloc_buffer`, read results, and then release both the input and the result:

| Export | Purpose |
| --- | --- |
| `alloc_buffer(size)` | Allocates an uninitialised input buffer. Release it with `dealloc_buffer` or pass it to `compile_wasm`. |
| `dealloc_buffer(pointer, size)` | Releases an input buffer. `size` must be the value passed to `alloc_buffer`. |
| `compile_wasm(pointer, length)` | Compiles UTF-8 source. Returns a result pointer, or null with a diagnostic. |
| `get_result_len()` | Length of the successful result; pass it to `free_result`. |
| `free_result(pointer, length)` | Releases a result buffer. Both arguments must come from the same `compile_wasm` call. |
| `get_error_ptr()` / `get_error_len()` | Static diagnostic buffer. Error text is never freed by the host. |

The error buffer is private static storage owned by the module, so a failed compilation does not leak and needs no
host cleanup. A successful compilation owns exactly one boxed slice; dropping it with `dealloc_buffer` is undefined,
and `free_result` is the only supported release path. `tests/test_wasm_compiler.js` shows the full sequence, and
`tests/ffi_lifecycle.js` proves repeated compilations keep linear memory bounded.

## The playground

`web/playground.html` is a complete host: it compiles a program in the page and runs the result with the shim.
Build the WebAssembly build of the compiler and serve the repository over HTTP:

```sh
cargo build --release --target wasm32-unknown-unknown --lib
python3 -m http.server 8000
```

Then open `http://localhost:8000/web/playground.html`. The page keeps the compiler in a worker, so a deep program
never blocks the page, and it runs the compiled program with the shim of this directory.

The page takes three parameters: `compiler` is the URL of the WebAssembly build of the compiler, `code` is the
program to compile, and `run` compiles and runs the program without a click. A link that carries a program is
therefore shareable:

```text
http://localhost:8000/web/playground.html?run=1&code=int%20main()%20%7B%20return%200%3B%20%7D
```

`tests/browser_flow.rs` runs the same sequence in Node, so the flow of a browser host stays working.
