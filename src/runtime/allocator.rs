use super::opcodes::*;
use crate::wasm::*;

/// Emits `i32 malloc(i32 size)`.
///
/// Each allocation has an eight-byte header: payload size at offset 0, the
/// next-free pointer at offset 4, then the payload at offset 8. Free chunks are
/// kept in a singly linked free list; new memory is bump-allocated from
/// `global_heap_base`.
pub fn emit_malloc(global_heap_base: u32, global_free_list: u32) -> FuncBody {
    // Locals:
    // 0: size (param)
    // 1: alloc_size (i32)
    // 2: prev (i32)
    // 3: curr (i32)
    // 4: chunk_sz (i32)
    // 5: next (i32)
    // 6: ptr (i32)
    // 7: new_heap_base (i32)
    // 8: cur_bytes (i32)
    // 9: needed_pages (i32)
    let mut code = Vec::new();

    // Reject negative and overflow-adjacent sizes before rounding them.
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 0);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 0);
    code.push(OP_I32_LT_S);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);

    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 0);
    code.push(OP_I32_CONST);
    write_i32(&mut code, i32::MAX - 7);
    code.push(OP_I32_GT_S);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);

    // 1. alloc_size = (size + 7) & -8; if (alloc_size < 8) alloc_size = 8;
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 0);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 7);
    code.push(OP_I32_ADD);
    code.push(OP_I32_CONST);
    write_i32(&mut code, -8);
    code.push(OP_I32_AND);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 1);

    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_I32_LT_S);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 1);
    code.push(OP_END);

    // 2. Search free list:
    // prev = 0
    code.push(OP_I32_CONST);
    write_i32(&mut code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 2);

    // curr = global.get global_free_list
    code.push(OP_GLOBAL_GET);
    write_u32(&mut code, global_free_list);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 3);

    // block $break (depth 1)
    code.push(OP_BLOCK);
    code.push(0x40);
    // loop $loop (depth 0)
    code.push(OP_LOOP);
    code.push(0x40);

    // if curr == 0 -> break $break (branch depth 1)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_I32_EQZ);
    code.push(OP_BR_IF);
    write_u32(&mut code, 1);

    // chunk_sz = i32.load curr (offset 0)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_I32_LOAD);
    write_u32(&mut code, 2);
    write_u32(&mut code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 4);

    // if chunk_sz >= alloc_size
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_I32_GE_U);
    code.push(OP_IF);
    code.push(0x40);

    // next = i32.load curr (offset 4)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_I32_LOAD);
    write_u32(&mut code, 2);
    write_u32(&mut code, 4);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 5);

    // if prev == 0
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 2);
    code.push(OP_I32_EQZ);
    code.push(OP_IF);
    code.push(0x40);
    // global_free_list = next
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 5);
    code.push(OP_GLOBAL_SET);
    write_u32(&mut code, global_free_list);
    code.push(OP_ELSE);
    // i32.store (prev + 4), next
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 2);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 5);
    code.push(OP_I32_STORE);
    write_u32(&mut code, 2);
    write_u32(&mut code, 4);
    code.push(OP_END);

    // return curr + 8
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_RETURN);

    code.push(OP_END); // end if chunk_sz >= alloc_size

    // prev = curr
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 2);

    // curr = i32.load curr (offset 4)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 3);
    code.push(OP_I32_LOAD);
    write_u32(&mut code, 2);
    write_u32(&mut code, 4);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 3);

    // br $loop (depth 0)
    code.push(OP_BR);
    write_u32(&mut code, 0);

    code.push(OP_END); // end loop
    code.push(OP_END); // end block

    // 3. Fallback: Bump allocate from global_heap_base
    // ptr = global.get global_heap_base
    code.push(OP_GLOBAL_GET);
    write_u32(&mut code, global_heap_base);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 6);

    // new_heap_base = ptr + alloc_size + 8
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 6);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 7);

    // if (new_heap_base < cur_heap_base) -> unreachable trap (address space exhausted)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 7);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 6);
    code.push(OP_I32_LT_U);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);

    // cur_bytes = memory.size << 16
    code.push(OP_MEMORY_SIZE);
    code.push(0x00);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 16);
    code.push(OP_I32_SHL);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 8);

    // if (new_heap_base > cur_bytes) grow memory
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 7);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 8);
    code.push(OP_I32_GT_U);
    code.push(OP_IF);
    code.push(0x40);

    // needed_pages = (new_heap_base - cur_bytes + 65535) >> 16
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 7);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 8);
    code.push(OP_I32_SUB);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 65535);
    code.push(OP_I32_ADD);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 16);
    code.push(OP_I32_SHR_U);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 9);

    // memory.grow(needed_pages)
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 9);
    code.push(OP_MEMORY_GROW);
    code.push(0x00);
    // if res == -1 -> unreachable trap
    code.push(OP_I32_CONST);
    write_i32(&mut code, -1);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);

    code.push(OP_END); // end if

    // i32.store ptr (offset 0), alloc_size
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 6);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_I32_STORE);
    write_u32(&mut code, 2);
    write_u32(&mut code, 0);

    // i32.store ptr (offset 4), 0
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 6);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 0);
    code.push(OP_I32_STORE);
    write_u32(&mut code, 2);
    write_u32(&mut code, 4);

    // global_heap_base = new_heap_base
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 7);
    code.push(OP_GLOBAL_SET);
    write_u32(&mut code, global_heap_base);

    // return ptr + 8
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 6);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_RETURN);
    code.push(OP_END);

    (vec![(9, TYPE_I32)], code)
}

/// Emits `void free(i32 ptr)`, prepending the block to the free list.
pub fn emit_free(global_free_list: u32) -> FuncBody {
    // Locals:
    // 0: ptr (param)
    // 1: chunk (i32)
    // 2: next (i32)
    let mut code = Vec::new();

    // if (ptr == 0) return;
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 0);
    code.push(OP_I32_EQZ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_RETURN);
    code.push(OP_END);

    // chunk = ptr - 8
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 0);
    code.push(OP_I32_CONST);
    write_i32(&mut code, 8);
    code.push(OP_I32_SUB);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 1);

    // next = global.get global_free_list
    code.push(OP_GLOBAL_GET);
    write_u32(&mut code, global_free_list);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, 2);

    // i32.store chunk (offset 4), next
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 2);
    code.push(OP_I32_STORE);
    write_u32(&mut code, 2);
    write_u32(&mut code, 4);

    // global_free_list = chunk
    code.push(OP_LOCAL_GET);
    write_u32(&mut code, 1);
    code.push(OP_GLOBAL_SET);
    write_u32(&mut code, global_free_list);

    code.push(OP_RETURN);
    code.push(OP_END);

    (vec![(2, TYPE_I32)], code)
}
