# Browser Integration & Host Interface

`cppwasm` is designed to run both natively and inside modern WebAssembly environments (browsers and Node.js). To achieve complete independence from external runtimes like Emscripten, `cppwasm` defines two clear interfaces:

1. **Compiler C ABI**: Exported functions that allow JavaScript host code to pass C++ source strings into the compiler WebAssembly instance and retrieve compiled WebAssembly binaries or error messages.
2. **Execution Host Interface (`web/host.js`)**: Import functions required by compiled WebAssembly programs to perform console I/O, format numbers, and exit.

---

## 1. Compiler WebAssembly C ABI

When built with `--target wasm32-unknown-unknown --lib`, `cppwasm` exports a low-level C ABI implemented in [`src/ffi.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/ffi.rs). Memory management is explicit: each compilation result is an independent boxed slice that must be allocated and released through matching functions.

### Exported Functions

| Export | Signature | Description |
|---|---|---|
| `alloc_buffer` | `(size: i32) -> i32` | Allocates an uninitialized input buffer of `size` bytes in the compiler's linear memory. |
| `dealloc_buffer` | `(ptr: i32, size: i32) -> void` | Frees an allocated input buffer if compilation is aborted before calling `compile_wasm`. |
| `compile_wasm` | `(ptr: i32, len: i32) -> i32` | Compiles UTF-8 source text. Returns a pointer to the emitted `.wasm` binary on success, or `0` on error. |
| `get_result_len` | `() -> i32` | Returns the byte length of the successful compilation result. |
| `free_result` | `(ptr: i32, len: i32) -> void` | Releases the compiled result buffer back to the compiler allocator. |
| `get_error_ptr` | `() -> i32` | Returns a pointer to the static null-terminated error string if compilation failed. |
| `get_error_len` | `() -> i32` | Returns the byte length of the static error string. |

### JavaScript Embedding Example

```javascript
// Step 1: Instantiate the compiler WebAssembly module
const compilerBytes = await fetch("pkg/cppwasm_bg.wasm").then(r => r.arrayBuffer());
const { instance } = await WebAssembly.instantiate(compilerBytes, {});
const wasm = instance.exports;

// Step 2: Encode source text and copy into compiler memory
const sourceText = 'int main() { return 42; }';
const encoded = new TextEncoder().encode(sourceText);
const inputPtr = wasm.alloc_buffer(encoded.length);
new Uint8Array(wasm.memory.buffer, inputPtr, encoded.length).set(encoded);

// Step 3: Run compilation
const resultPtr = wasm.compile_wasm(inputPtr, encoded.length);

if (resultPtr === 0) {
    // Compilation failed: read static error message
    const errLen = wasm.get_error_len();
    const errPtr = wasm.get_error_ptr();
    const errBytes = new Uint8Array(wasm.memory.buffer, errPtr, errLen);
    const message = new TextDecoder().decode(errBytes);
    console.error("Compilation error:\n" + message);
} else {
    // Compilation succeeded: copy out emitted WebAssembly bytecode
    const resultLen = wasm.get_result_len();
    const outputBytes = new Uint8Array(wasm.memory.buffer, resultPtr, resultLen).slice();
    
    // Explicitly release the result buffer
    wasm.free_result(resultPtr, resultLen);

    // outputBytes is now a complete, standalone .wasm binary ready to run!
}
```

The error buffer is private static storage owned by the compiler; failed compilations never leak linear memory. Bounded memory consumption across repeated compilations is verified by [`tests/ffi_lifecycle.js`](https://github.com/cppwasm/cpp_web/blob/main/tests/ffi_lifecycle.js).

---

## 2. Program Execution Host Runtime (`web/host.js`)

When a compiled C++ program runs, its WebAssembly module expects an imported `env` namespace providing standard character and numeric I/O. [`web/host.js`](https://github.com/cppwasm/cpp_web/blob/main/web/host.js) implements this environment for both browser and Node.js hosts:

```javascript
import { createHostEnv } from "./web/host.js";

const host = createHostEnv({
    onStdout: (text) => console.log(text),
    onStderr: (text) => console.error(text),
});

// host.importObject provides the imports needed by the compiled C++ program
const program = await WebAssembly.instantiate(outputBytes, host.importObject);
program.instance.exports.main();
```

### Imported Environment Functions

| Import | Signature | Behavior |
|---|---|---|
| `env.print_char` | `(ch: i32) -> void` | Writes a single ASCII/UTF-8 character byte to stdout buffer. |
| `env.print_int` | `(val: i64, radix: i32, width: i32, prec: i32, flags: i32) -> i32` | Formats integer with specified width, precision, and flags. Returns character count. |
| `env.print_double` | `(val: f64, width: i32, prec: i32, mode: i32, flags: i32) -> i32` | Formats double in general (`0`), fixed (`1`), or scientific (`2`) mode. Returns character count. |
| `env.read_char` | `() -> i32` | Reads next character byte from stdin buffer (returns `-1` on EOF). |
| `env.exit` | `(code: i32) -> void` | Terminates execution with the given status code. |

#### Formatting Flags Bitmask
For `print_int` and `print_double`:
- **Bit 0 (`0x01`)**: Zero-padding (`'0'`).
- **Bit 1 (`0x02`)**: Left-alignment (`'-'`).
- **Bit 2 (`0x04`)**: Always show sign (`'+'`).
- **Bit 3 (`0x08`)**: Space sign (`' '`).
- **Bit 4 (`0x10`)**: Uppercase hex/exponent (`'X'`, `'E'`).
- **Bit 5 (`0x20`)**: Unsigned integer conversion (`print_int` only).
- **Bit 6 (`0x40`)**: Alternate format (`'#'`).

---

## 3. Web Worker Architecture in the Playground

Compiling complex C++ programs with deep recursion can consume significant CPU cycles. To ensure smooth browser UI responsiveness, the browser playground ([`web/playground.html`](https://github.com/cppwasm/cpp_web/blob/main/web/playground.html)) offloads all compiler work to a Web Worker ([`web/playground-worker.js`](https://github.com/cppwasm/cpp_web/blob/main/web/playground-worker.js)):

```text
 Playground Page (UI)              Web Worker (Compiler)           Compiled C++ Instance
        │                                  │                                  │
        │── postMessage({compile, code}) ─>│                                  │
        │                                  │── compile_wasm(ptr, len)         │
        │                                  │                                  │
        │<─ postMessage({success, bytes}) ─│                                  │
        │                                                                     │
        │── WebAssembly.instantiate(wasmBytes, hostEnv) ─────────────────────>│
        │── exports.main() ──────────────────────────────────────────────────>│
        │                                                                     │
        │<── host.print_char / print_int (stdout) ────────────────────────────│
        │                                                                     │
```

### Shareable URLs
The playground supports URL query parameters so interactive code snippets can be shared:
- `code`: URL-encoded C++ source code.
- `run=1`: Automatically compiles and executes the program upon loading without requiring a button click.
- `compiler`: Custom URL path to a `cppwasm_bg.wasm` compiler binary.

Example:
```text
http://localhost:8000/?run=1&code=int%20main()%20%7B%20return%200%3B%20%7D
```

Execution flow in Node.js and headless browser flows is covered by [`tests/browser_flow.rs`](https://github.com/cppwasm/cpp_web/blob/main/tests/browser_flow.rs).
