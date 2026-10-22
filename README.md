# C++ to WebAssembly subset compiler

This repository contains a dependency-free Rust compiler for a defined subset of C++17. It preprocesses,
lexes, parses, semantically checks, and emits WebAssembly directly, with a native CLI and a browser/Node C ABI.
The project is correctness-focused and intentionally does not implement templates, standard containers,
exceptions, RTTI, or other Phase 3/4 features.

Build and run the current corpus:

```sh
cargo build --release
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

Try it in a browser:

```sh
cargo build --release --target wasm32-unknown-unknown --lib
tools/build_site.sh
python3 -m http.server --directory site 8000
# open http://localhost:8000/
```

The playground compiles a program in the page and runs the result; the compiler runs in a worker, so a deep
program never blocks the page.  `web/host.js` is the shim that both the playground and the Node tools use, and
`web/pkg/` holds the JavaScript glue and the WebAssembly build of the compiler that the site serves.

The site is deployed to GitHub Pages by `.github/workflows/pages.yml` on every push to `main`.  The repository
needs "Settings -> Pages -> Build and deployment -> Source: GitHub Actions" once.

The architecture and ABI contracts are documented in [`docs/`](docs/README.md). The embedded headers are in
[`stdlib/`](stdlib/), and the JavaScript host shim is [`web/host.js`](web/host.js).
