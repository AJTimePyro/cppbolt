# Getting Started with cppwasm

`cppwasm` is a dependency-free C++ to WebAssembly subset compiler written in Rust. It compiles a defined subset of C++17 directly to WebAssembly bytecode without requiring LLVM, Clang, or Emscripten.

---

## Prerequisites

To build and run `cppwasm`, you will need:
- **Rust Toolchain**: Stable Rust with the 2024 edition (`rustc` and `cargo`).
- **WebAssembly Target** (for browser build): `rustup target add wasm32-unknown-unknown`.
- **Node.js** (v18+ recommended) for running command-line WebAssembly tests.
- **Python 3** (optional, for local HTTP serving of the playground).
- **C++ Compiler** (optional, `g++` or `clang++` for running native oracle differential tests).

---

## 1. Building the Native Compiler

Clone the repository and build the release binary:

```sh
# Build the native CLI compiler (target/release/cppwasm-cli)
cargo build --release
```

The compiled binary will be placed at `target/release/cppwasm-cli`.

---

## 2. Compiling Your First C++ Program

Create a simple C++ program, for example `hello.cpp`:

```cpp
#include <iostream>

int main() {
    std::cout << "Hello from WebAssembly C++!" << std::endl;
    return 0;
}
```

Compile it to WebAssembly using the CLI:

```sh
target/release/cppwasm-cli hello.cpp -o hello.wasm
```

### Running the WebAssembly Module

The emitted `.wasm` binary imports character, integer, and double output functions along with `exit` from the host environment `env`. You can execute it using the repository's Node.js runner:

```sh
node tests/test_wasm_compiler.js hello.wasm
```

Expected output:
```text
Hello from WebAssembly C++!
```

---

## 3. Running the Test Suite

`cppwasm` includes a corpus test runner that verifies language behavior, golden binary hashes, and error diagnostics:

```sh
# Run the testrunner against the release CLI binary
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

The test runner will discover all test groups, verify the manifest in `tests/PASSING.txt`, and check that all positive tests match their expected output.

---

## 4. Running the Browser Playground Locally

You can try the compiler directly in the live [Online Playground](https://ajtimepyro.is-a.dev/cpp-web-compiler/) without local installation, or build and run it locally:

```sh
# 1. Build the compiler as a WebAssembly cdylib
cargo build --release --target wasm32-unknown-unknown --lib

# 2. Assemble the playground assets into site/
tools/build_site.sh

# 3. Serve the site locally
python3 -m http.server --directory site 8000
```

Open `http://localhost:8000/` in your browser.

The playground executes compilation inside a background Web Worker so deep compilations never block the main browser thread. You can write C++ code in the browser editor, click **Run**, and see stdout rendered in real time.

For more details on integrating the compiler into web applications, see [Browser Integration](browser-integration.md).
