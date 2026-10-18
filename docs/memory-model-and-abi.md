# Memory model and ABI

Generated programs use 32-bit ILP32 values. Linear memory is divided into a reserved first 64 KiB, an 8 MiB
downward-growing stack, static data, and an upward-growing allocator heap. The constants live in
`src/memory_layout.rs`; code generation must use that module rather than literals.

Pointers are four-byte Wasm `i32` values. A null pointer is reserved by allocation policy but a raw load at
address zero is not a hardware trap. The indirect function table reserves slot zero for the deterministic trap
entry. Virtual objects contain a vtable pointer at offset zero; slots follow base-prefix order.

The generated module imports character/integer/floating output, character input, and exit from `env`, and
exports memory, stack/heap globals, and the program entry point. `malloc` returns null on allocation failure;
`new` follows the compiler's trapping allocation path.
