# cppbolt

cppbolt compiles a subset of C++17 directly to WebAssembly without LLVM, Emscripten, or a native toolchain. This package contains the compiler built for `wasm32-unknown-unknown`; it exposes a C ABI and has no imports, so you can instantiate it with an empty import object in Node.js, browsers, or Web Workers.

## Installation

```sh
npm install cppbolt
```

## Usage

The package provides three entry points. Node.js and bundlers select the right
one automatically; native browsers and classic scripts use the file directly.

| Environment                            | Entry point               | How to load it                                                           |
| -------------------------------------- | ------------------------- | ------------------------------------------------------------------------ |
| Node.js (CommonJS or ES modules)       | `dist/index.js`           | `require("cppbolt")` or `import ... from "cppbolt"`                      |
| Browser bundlers and native ES modules | `dist/index.browser.mjs`  | `import ... from "cppbolt"` (bundler) or the module URL (native browser) |
| Classic browser `<script>`             | `dist/cppbolt.browser.js` | `<script src="..."></script>`, then use the global `CppBolt`             |

### Node.js

`createCompiler()` loads `wasm/cppbolt.wasm` from the installed package. The API
is identical from CommonJS and ES modules:

```js
// CommonJS
const { createCompiler, compileAndRun } = require("cppbolt");

// ES modules
import { createCompiler, compileAndRun } from "cppbolt";
```

#### Compile only

`compile()` turns C++ source into a complete WebAssembly module as a
`Uint8Array`.

```js
const compiler = await createCompiler();

const wasm = compiler.compile(`
    #include <iostream>
    int main() {
        std::cout << "cppbolt says hello!" << std::endl;
        return 0;
    }
`);
```

#### Compile and run

`compileAndRun()` compiles the source and immediately executes it, returning
its stdout and exit status.

```js
const compiler = await createCompiler();

const { stdout, exitCode } = await compileAndRun(
	compiler,
	`
    #include <iostream>
    int main() {
        std::cout << "cppbolt says hello!" << std::endl;
        return 0;
    }
`,
);

console.log(stdout); // "cppbolt says hello!\n"
console.log(exitCode); // 0
```

#### Run with timeouts and worker isolation (`compileAndRunAsync`)

Run code in an isolated worker with timeouts, cancellation, stdin input, and streaming stdout:

```js
const { compileAndRunAsync, TimeoutError } = require("cppbolt");

try {
	const res = await compileAndRunAsync(
		compiler,
		`
        #include <iostream>
        int main() {
            int x;
            std::cin >> x;
            std::cout << "Received: " << x << std::endl;
            return 0;
        }
    `,
		{
			stdin: "42\n",
			timeoutMs: 2000,
			onStdout: (chunk) => process.stdout.write(chunk),
		},
	);
	console.log("Exit status:", res.exitCode);
} catch (err) {
	if (err instanceof TimeoutError) {
		console.error(err.message);
		console.log("Partial output:", err.stdout);
	} else {
		throw err;
	}
}
```

### Browser: bundlers and native ES modules

In Vite, webpack 5, Rollup, or esbuild, use the normal package import. The
`browser` export condition selects `dist/index.browser.mjs`.

```js
import { createCompiler, compileAndRun } from "cppbolt";

const compiler = await createCompiler();
const { stdout, exitCode } = await compileAndRun(
	compiler,
	`
  #include <iostream>
  int main() {
      std::cout << "cppbolt says hello!" << std::endl;
      return 0;
  }
`,
);
```

`createCompiler()` resolves `wasm/cppbolt.wasm` relative to the module URL.
Bundlers that understand `new URL(..., import.meta.url)` bundle the binary
automatically. If the wasm file is served elsewhere, pass an explicit URL:
`createCompiler("https://example.com/cppbolt.wasm")`.

Native browser ES modules do not support bare package names. Import the module
file directly, or add a matching import map:

```html
<script type="module">
  import { createCompiler, compileAndRun } from "https://cdn.jsdelivr.net/npm/cppbolt@latest/dist/index.browser.mjs";

  const compiler = await createCompiler();
  const { stdout, exitCode } = await compileAndRun(compiler, `
    #include <iostream>
    int main() {
        std::cout << "cppbolt says hello!" << std::endl;
        return 0;
    }
  `);
  console.log(stdout, exitCode);
</script>
```

### Browser: classic `<script>` tag

The classic build exposes the global `CppBolt`. Load the script over HTTP(S),
not `file://`, so it can fetch the wasm binary next to it.

```html
<script src="https://cdn.jsdelivr.net/npm/cppbolt@latest/dist/cppbolt.browser.js"></script>
<script>
  (async () => {
    const compiler = await CppBolt.createCompiler();
    const { stdout, exitCode } = await CppBolt.compileAndRun(compiler, `
      #include <iostream>
      int main() {
          std::cout << "cppbolt says hello!" << std::endl;
          return 0;
      }
    `,
    );
    console.log(stdout, exitCode);
  })();
</script>
```

`CppBolt.createCompiler()` resolves `wasm/cppbolt.wasm` relative to the script
URL.

### How the entry points are selected

The npm manifest uses conditional exports, evaluated in declaration order:

| Condition | Entry point              | Selected by                      |
| --------- | ------------------------ | -------------------------------- |
| `types`   | `dist/index.d.ts`        | TypeScript                       |
| `browser` | `dist/index.browser.mjs` | Browser-targeting bundlers       |
| `node`    | `dist/index.js`          | Node.js (`require` and `import`) |
| `default` | `dist/index.browser.mjs` | Other ES module environments     |

## Compiler API

Pointers are 32-bit offsets into the module's exported `memory`.

| Export           | Signature             | Description                                                                                                        |
| ---------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `alloc_buffer`   | `(size) -> ptr`       | Allocates an uninitialized input buffer.                                                                           |
| `dealloc_buffer` | `(ptr, size) -> void` | Releases an input buffer with the size passed to `alloc_buffer`.                                                   |
| `compile_wasm`   | `(ptr, len) -> ptr`   | Compiles UTF-8 source. Returns a pointer to the emitted module, or `0` on error.                                   |
| `get_result_len` | `() -> len`           | Returns the byte length of the result from the most recent `compile_wasm` call, or `0` if that call failed.        |
| `free_result`    | `(ptr, len) -> void`  | Releases a result buffer returned by `compile_wasm`.                                                               |
| `get_error_ptr`  | `() -> ptr`           | Returns the address of the static error buffer.                                                                    |
| `get_error_len`  | `() -> len`           | Returns the byte length of the diagnostic from the most recent failed `compile_wasm` call, or `0` if it succeeded. |

Ownership and lifetime rules:

- `compile_wasm` does not take ownership of the input buffer. Free it with `dealloc_buffer` after compiling.
- Call `get_result_len` before starting another compilation. Copy the bytes out, then call `free_result` once with that length.
- The error buffer is static storage owned by the compiler. The message stays valid until the next compilation and must not be freed. Messages are truncated to 4096 bytes.
- Linear memory growth can invalidate existing JavaScript memory views. Recreate `Uint8Array` and other typed-array views after calls that may grow memory. Numeric pointers remain offsets into linear memory.

Diagnostics use the format `<file>:line:column: error: message`. The ABI does not accept a filename, so the main source is reported as `<input>`; errors inside embedded headers use the header's name.

## Language coverage

cppbolt supports scalar types, pointers, arrays, functions, classes, non-static data member initializers (NSDMI), aggregate initialization, selection initializers (`if`/`switch`), comma operator expressions, single inheritance, virtual functions, operator overloading, lambdas, range-based `for`, and the preprocessor, along with an embedded mini standard library (`<iostream>`, `<string>`, `<cstdio>`, `<cmath>`, `<iomanip>`, and related headers).

Templates, standard containers and algorithms, exceptions, RTTI, and multiple or virtual inheritance are not supported.

Only the embedded headers can be included. The Wasm build has no filesystem access, so `#include` of any other file cannot be resolved.

See the [supported C++ subset](https://ajtimepyro.is-a.dev/cppbolt/book/user/supported-cpp.html) and [diagnostics catalog](https://ajtimepyro.is-a.dev/cppbolt/book/user/errors.html) for the full feature matrix and error list.

## Running compiled modules

Emitted modules are WebAssembly binaries that import standard I/O functions from an `env` namespace. Every emitted module imports all five functions:

| Import             | Signature                          | Purpose                                                      |
| ------------------ | ---------------------------------- | ------------------------------------------------------------ |
| `env.print_char`   | `(i32) -> void`                    | Write one character byte to stdout.                          |
| `env.print_int`    | `(i64, i32, i32, i32, i32) -> i32` | Format and write an integer. Returns the character count.    |
| `env.print_double` | `(f64, i32, i32, i32, i32) -> i32` | Format and write a double. Returns the character count.      |
| `env.read_char`    | `() -> i32`                        | Read one character byte from stdin, or `-1` at end of input. |
| `env.exit`         | `(i32) -> void`                    | Terminate with the given status code.                        |

Emitted modules export `memory`, `__stack_pointer`, and `__heap_base`, plus `main` if the program defines it. The compiler gives them a 256 MiB maximum linear memory, raised only when static data requires more.

### Host runtime API

The package ships a host for those imports, so you do not implement `env.*`
yourself.

- `createWasmHost(options?) -> WasmHost` builds the import environment. Options
  are `stdin` (a string, or `() => number` for streaming input), `onStdout(chunk)`,
  and `onExit(code)`. Calling `exit(code)` invokes `onExit` when supplied and
  always stops the wasm program; the runner returns that code without terminating
  the embedding Node.js process. The result exposes `imports` for
  `WebAssembly.instantiate` and `getStdout()`.
- `new CppBoltRunner(host).run(wasmBytes) -> Promise<RunResult>` instantiates a
  module, calls `main` (or `_start`), and returns `{ stdout, exitCode }`. A
  WebAssembly trap is reported as exit code 134 instead of throwing. Runs
  synchronously on the current thread and does not support watchdog timeouts.
- `new AsyncCppBoltRunner(host?).run(wasmBytes, options?) -> Promise<RunResult>` runs
  the module in an isolated background worker (`node:worker_threads` or browser Web Worker)
  with `timeoutMs`, `signal` (`AbortSignal`), `stdin`, and streaming `onStdout`.
  Invalid bytecode rejects with `CompileError`/`LinkError`, while program traps resolve
  with exit code 134.
- `compileAndRun(compiler, source, options?)` compiles and runs in one call.
- `compileAndRunAsync(compiler, source, options?)` compiles and runs in a background worker.

## Notes

- The compiler ABI is single-threaded and stores result state in one slot. Serialize calls and use one instance per worker.
- Compilation is synchronous. In a browser, run it in a Web Worker to keep the UI responsive.

## License

Apache-2.0. See the [LICENSE](https://github.com/AJTimePyro/cppbolt/blob/main/LICENSE) file.
