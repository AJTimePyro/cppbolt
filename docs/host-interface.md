# Host interface

`web/host.js` supplies the generated program imports: character output, integer output, floating output, character
input, and process exit. It buffers stdout and implements the compiler's number-formatting contract.

The compiler Wasm ABI exports `alloc_buffer`, `compile_wasm`, result/error length accessors, and an error
pointer accessor. Callers allocate UTF-8 input, invoke compilation, then copy the returned result or diagnostic
before releasing the buffer. Node and browser workers use the same ABI; the browser must keep compiler execution
off the UI thread.
