# cppbolt

cppbolt compiles a subset of C++17 directly to WebAssembly without LLVM, Emscripten, or a native toolchain. This package contains the compiler built for `wasm32-unknown-unknown`; it exposes a C ABI and has no imports, so you can instantiate it with an empty import object in Node.js, browsers, or Web Workers.

The package contains the raw WebAssembly module with no JavaScript wrapper or CLI. You call the exports directly.

## Installation

```sh
npm install cppbolt
```

## Usage

```js
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const wasmPath = require.resolve("cppbolt/wasm/cppbolt.wasm");
const { instance } = await WebAssembly.instantiate(await readFile(wasmPath), {});
const compiler = instance.exports;

function compile(source) {
  const input = new TextEncoder().encode(source);
  const inputPtr = compiler.alloc_buffer(input.length);
  new Uint8Array(compiler.memory.buffer, inputPtr, input.length).set(input);

  const resultPtr = compiler.compile_wasm(inputPtr, input.length);
  compiler.dealloc_buffer(inputPtr, input.length);

  if (resultPtr === 0) {
    const length = compiler.get_error_len();
    const bytes = new Uint8Array(compiler.memory.buffer, compiler.get_error_ptr(), length);
    throw new Error(new TextDecoder().decode(bytes));
  }

  const length = compiler.get_result_len();
  const output = new Uint8Array(compiler.memory.buffer, resultPtr, length).slice();
  compiler.free_result(resultPtr, length);
  return output;
}

const wasm = compile(`
  #include <iostream>

  int main() {
      std::cout << "Hello from WebAssembly C++!" << std::endl;
      return 0;
  }
`);
```

`wasm` is a complete WebAssembly module.

### Browser

Serve `cppbolt.wasm` from your site, then load it:

```html
<script type="module">
  const response = await fetch("/wasm/cppbolt.wasm");
  const { instance } = await WebAssembly.instantiateStreaming(response, {});
  const compiler = instance.exports;
</script>
```

`instantiateStreaming` needs the server to send `Content-Type: application/wasm`; otherwise use `WebAssembly.instantiate(await response.arrayBuffer(), {})`. The compiler ABI interaction inside the `compile` function can be reused in browsers. Replace the Node.js-specific WASM loading code with the browser loading example above.

## Compiler API

Pointers are 32-bit offsets into the module's exported `memory`.

| Export | Signature | Description |
|---|---|---|
| `alloc_buffer` | `(size) -> ptr` | Allocates an uninitialized input buffer. |
| `dealloc_buffer` | `(ptr, size) -> void` | Releases an input buffer with the size passed to `alloc_buffer`. |
| `compile_wasm` | `(ptr, len) -> ptr` | Compiles UTF-8 source. Returns a pointer to the emitted module, or `0` on error. |
| `get_result_len` | `() -> len` | Returns the byte length of the result from the most recent `compile_wasm` call, or `0` if that call failed. |
| `free_result` | `(ptr, len) -> void` | Releases a result buffer returned by `compile_wasm`. |
| `get_error_ptr` | `() -> ptr` | Returns the address of the static error buffer. |
| `get_error_len` | `() -> len` | Returns the byte length of the diagnostic from the most recent failed `compile_wasm` call, or `0` if it succeeded. |

Ownership and lifetime rules:

- `compile_wasm` does not take ownership of the input buffer. Free it with `dealloc_buffer` after compiling.
- Call `get_result_len` before starting another compilation. Copy the bytes out, then call `free_result` once with that length.
- The error buffer is static storage owned by the compiler. The message stays valid until the next compilation and must not be freed. Messages are truncated to 4096 bytes.
- Linear memory growth can invalidate existing JavaScript memory views. Recreate `Uint8Array` and other typed-array views after calls that may grow memory. Numeric pointers remain offsets into linear memory.

Diagnostics use the format `<file>:line:column: error: message`. The ABI does not accept a filename, so the main source is reported as `<input>`; errors inside embedded headers use the header's name.

## Language coverage

cppbolt supports scalar types, pointers, arrays, functions, classes, single inheritance, virtual functions, operator overloading, lambdas, range-based `for`, and the preprocessor, along with an embedded mini standard library (`<iostream>`, `<string>`, `<cstdio>`, `<cmath>`, `<iomanip>`, and related headers).

Templates, standard containers and algorithms, exceptions, RTTI, and multiple or virtual inheritance are not supported.

Only the embedded headers can be included. The Wasm build has no filesystem access, so `#include` of any other file cannot be resolved.

See the [supported C++ subset](https://ajtimepyro.is-a.dev/cppbolt/book/user/supported-cpp.html) and [diagnostics catalog](https://ajtimepyro.is-a.dev/cppbolt/book/user/errors.html) for the full feature matrix and error list.

## Running compiled modules

Emitted modules are WebAssembly binaries that import standard I/O functions from an `env` namespace. Every emitted module imports all five functions:

| Import | Signature | Purpose |
|---|---|---|
| `env.print_char` | `(i32) -> void` | Write one character byte to stdout. |
| `env.print_int` | `(i64, i32, i32, i32, i32) -> i32` | Format and write an integer. Returns the character count. |
| `env.print_double` | `(f64, i32, i32, i32, i32) -> i32` | Format and write a double. Returns the character count. |
| `env.read_char` | `() -> i32` | Read one character byte from stdin, or `-1` at end of input. |
| `env.exit` | `(i32) -> void` | Terminate with the given status code. |

Emitted modules export `memory`, `__stack_pointer`, and `__heap_base`, plus `main` if the program defines it. The compiler gives them a 256 MiB maximum linear memory, raised only when static data requires more.

The package does not include a host runtime. The repository's [`web/host.js`](https://github.com/AJTimePyro/cppbolt/blob/main/web/host.js) provides a reference implementation for Node.js and browsers. It handles the integer and floating-point formatting flags.

## Notes

- The compiler ABI is single-threaded and stores result state in one slot. Serialize calls and use one instance per worker.
- Compilation is synchronous. In a browser, run it in a Web Worker to keep the UI responsive.

## License

Apache-2.0. See the [LICENSE](https://github.com/AJTimePyro/cppbolt/blob/main/LICENSE) file.
