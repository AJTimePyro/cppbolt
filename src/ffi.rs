// The exported C entry points take pointers with the validity and ownership
// rules documented at each function; callers outside Rust cannot express
// `unsafe`, so the checks stay inside these wrappers.
#![allow(unsafe_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::compile_with_options_named;

static mut RESULT_LEN: usize = 0;
static mut ERROR_LEN: usize = 0;
static mut ERROR_BYTES: [u8; 4096] = [0; 4096];

#[unsafe(no_mangle)]
pub extern "C" fn alloc_buffer(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc_buffer(pointer: *mut u8, size: usize) {
    if !pointer.is_null() && size > 0 {
        // SAFETY: the pointer and capacity are returned by alloc_buffer and ownership is transferred back once.
        unsafe {
            drop(Vec::from_raw_parts(pointer, size, size));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn compile_wasm(source_pointer: *const u8, source_length: usize) -> *const u8 {
    // SAFETY: the ABI caller supplies a readable UTF-8 buffer of the declared length.
    let source_bytes = unsafe { std::slice::from_raw_parts(source_pointer, source_length) };
    let source = match std::str::from_utf8(source_bytes) {
        Ok(source) => source,
        Err(_) => return std::ptr::null(),
    };

    match compile_with_options_named(source, "<input>", 256) {
        Ok(mut bytes) => {
            let pointer = bytes.as_mut_ptr();
            let length = bytes.len();
            std::mem::forget(bytes);
            // SAFETY: the ABI is single-threaded in the supported browser/Node hosts.
            unsafe {
                RESULT_LEN = length;
                ERROR_LEN = 0;
            }
            pointer
        }
        Err(error) => {
            let message = error.to_string();
            // SAFETY: the fixed buffer is private to this ABI and the copy is bounded.
            unsafe {
                RESULT_LEN = 0;
                let bytes = message.as_bytes();
                let length = bytes.len().min(4096);
                ERROR_BYTES[..length].copy_from_slice(&bytes[..length]);
                ERROR_LEN = length;
            }
            std::ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_result_len() -> usize {
    // SAFETY: see compile_wasm's single-threaded ABI contract.
    unsafe { RESULT_LEN }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_error_ptr() -> *const u8 {
    // SAFETY: the pointer refers to the private static diagnostic buffer.
    std::ptr::addr_of!(ERROR_BYTES).cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_error_len() -> usize {
    // SAFETY: see compile_wasm's single-threaded ABI contract.
    unsafe { ERROR_LEN }
}
