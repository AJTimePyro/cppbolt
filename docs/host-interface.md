# Host interface

`web/host.js` supplies the generated program imports: character output, integer output, floating output, character
input, and process exit. It buffers stdout and implements the compiler's number-formatting contract. Hosts use the
same shim in Node and in browsers; the browser must keep compiler execution off the UI thread.

`print_char(ch)` returns nothing. `print_int(value, radix, width, precision, flags)` and
`print_double(value, width, precision, mode, flags)` return the number of characters they wrote, including any
sign and field padding; the generated `printf` adds that count to its own return value. For `print_int`, flags
are bit 0 zero-pad, bit 1 left-align, bit 2 show `+`, bit 3 space sign, bit 4 uppercase, bit 5 unsigned, and
bit 6 alternate form. For `print_double`, modes are 0 general, 1 fixed, and 2 scientific; flags are bit 0
zero-pad, bit 1 left-align, bit 2 show `+`, bit 3 space sign, and bit 4 uppercase.

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
