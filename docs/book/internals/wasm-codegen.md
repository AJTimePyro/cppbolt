# WebAssembly Code Generation Internals

The code generation stage lowers the typed AST and semantic symbol table directly into WebAssembly binary sections, implemented across `src/codegen/` and encoded by `src/wasm/`.

---

## 1. Role in the Pipeline

1. **Direct AST-to-Wasm Lowering**: As established in [ADR 0002](../adr/0002-direct-wasm.md) and [Architecture](architecture.md), `cppwasm` does not translate AST nodes into an intermediate representation (IR) or execute bytecode optimization passes. It walks the typed AST and writes WebAssembly opcodes directly into byte buffers.
2. **Function Frame Management**: Allocates checked local stack slots, emits prologue stack overflow checks, and coordinates RAII destructor calls before every function return point (`src/codegen/function_context.rs`).
3. **Fixed-Width LEB128 Patching**: Because the final stack frame size is only known after traversing the entire function body, prologues and epilogues emit 5-byte fixed-width LEB128 placeholders that are patched in place after body lowering.
4. **Binary Section Assembly**: Packs types, imports, functions, tables, memory, globals, exports, elements, code bodies, and data segments into the standard WebAssembly binary module format (`src/wasm/module.rs`).

---

## 2. Key Data Structures

- `Codegen<'a>`: The top-level code generator state:
  - `sema: &'a Sema`: Reference to verified semantic layouts and symbols.
  - `module: WasmModule`: The in-memory WebAssembly module.
  - `string_table: HashMap<String, u32>`: Interned string literal offsets.
  - `next_data_offset: u32`: Allocation pointer for static data starting at `DATA_BASE`.
  - `func_indices: HashMap<String, u32>`: Function indices for direct calls.
  - `vtable_offsets: HashMap<String, u32>`: Vtable offsets in static linear memory.
- `FunctionContext`: Per-function lowering state:
  - `fp_local: u32`: Local variable holding the frame pointer.
  - `scratch1..2, scratch_i64, scratch_f64`: Reserved Wasm locals for temporary operand staging.
  - `var_offsets: HashMap<String, u32>`: Byte offsets from the frame pointer for each local variable.
  - `frame_size: i32`: Computed total frame size.
  - `frame_size_patches: Vec<usize>`: Byte offsets in the code buffer requiring 5-byte LEB128 frame size patching.
  - `scope_objects: Vec<Vec<(String, Type, u32)>>`: Active objects requiring RAII destruction at block/function exits.
- `WasmModule`: In-memory representation of WebAssembly sections before serialization.

---

## 3. Function Frame Protocol & LEB128 Patching

Each generated function follows a strict stack frame protocol:

```text
       Call Entry
           │
           ▼
[1. Prologue]
    - Read global stack pointer (global 0)
    - Check: new_sp < GUARD_PAGE_END (64 KiB)? -> unreachable trap
    - Decrement global stack pointer by frame_size
    - Set local fp = new_sp
           │
           ▼
[2. Function Body]
    - Lower expressions & statements
    - Allocate locals at [fp + offset]
    - Register RAII destructors
           │
           ▼
[3. Epilogue / Return]
    - Run active destructors in reverse construction order
    - Restore global stack pointer = fp + frame_size
    - Return value
```

### The 5-Byte LEB128 Placeholder Technique
Because the total frame size is only known after all nested blocks and temporaries in a function are compiled, the prologue and epilogue emit a fixed 5-byte LEB128 placeholder:

```rust
// In FunctionContext::emit_frame_size
code.push(OP_I32_CONST); // 0x41
self.frame_size_patches.push(code.len());
code.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x00]);
```

Once the body completes and `self.frame_size` is calculated, `FunctionContext::patch_frame_sizes` overwrites the 5 bytes with the true value padded with continuation bits. This guarantees that code offsets and branch destinations never shift during compilation.

---

## 4. Real Input & WebAssembly Opcode Output Example

Taken from `tests/c_basics/arithmetic.cpp`:

### Input C++ Source
```cpp
int main() {
    int a = 10 + 20;
    return a;
}
```

### Emitted WebAssembly Instructions (WAT equivalent)
```wat
(func $main (result i32)
  (local $fp i32)
  (local $scratch1 i32)
  (local $scratch2 i32)
  (local $scratch_i64 i64)
  (local $scratch_f64 f64)
  (local $scratch_f32 f32)
  (local $ret_i32 i32)
  (local $ret_i64 i64)
  (local $ret_f64 f64)

  ;; Prologue: allocate 16-byte aligned frame
  global.get 0
  i32.const 16
  i32.sub
  local.tee $fp
  global.set 0

  ;; Guard check
  local.get $fp
  i32.const 65536
  i32.lt_u
  if
    unreachable
  end

  ;; int a = 10 + 20; (store at [fp + 0])
  local.get $fp
  i32.const 0
  i32.add
  i32.const 10
  i32.const 20
  i32.add
  i32.store offset=0

  ;; return a; (load from [fp + 0])
  local.get $fp
  i32.const 0
  i32.add
  i32.load offset=0
  local.set $ret_i32

  ;; Epilogue: restore stack pointer
  global.get 0
  i32.const 16
  i32.add
  global.set 0

  local.get $ret_i32
  return
)
```
