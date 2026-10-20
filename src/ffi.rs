// The exported C entry points take pointers with the validity and ownership
// rules documented at each function; callers outside Rust cannot express
// `unsafe`, so the checks stay inside these wrappers.
#![allow(unsafe_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::compile_with_options_named;
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE_RESULTS: AtomicUsize = AtomicUsize::new(0);

static mut RESULT_LEN: usize = 0;
static mut ERROR_LEN: usize = 0;
static mut ERROR_BYTES: [u8; 4096] = [0; 4096];

/// Allocates an uninitialised input buffer of `size` bytes.
///
/// The caller owns the returned block and must release it with
/// [`dealloc_buffer`] using the same size, or hand it to `compile_wasm`.
#[unsafe(no_mangle)]
pub extern "C" fn alloc_buffer(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

/// Releases a buffer previously returned by [`alloc_buffer`].
///
/// The `size` must be the value passed to `alloc_buffer`; input buffers are
/// allocated with that exact capacity, so the deallocation layout matches.
#[unsafe(no_mangle)]
pub extern "C" fn dealloc_buffer(pointer: *mut u8, size: usize) {
    if !pointer.is_null() && size > 0 {
        // SAFETY: the pointer and capacity are returned by alloc_buffer and ownership is transferred back once.
        unsafe {
            drop(Vec::from_raw_parts(pointer, size, size));
        }
    }
}

/// Releases a successful result buffer returned by `compile_wasm`.
///
/// Results are boxed slices, so `length` must be `get_result_len()` from the
/// same call. Error text lives in a static buffer and is never freed.
#[unsafe(no_mangle)]
pub extern "C" fn free_result(pointer: *mut u8, length: usize) {
    if pointer.is_null() || length == 0 {
        return;
    }
    // SAFETY: the pointer and length are exactly the boxed slice returned by
    // compile_wasm and ownership is transferred back once.
    unsafe {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            pointer, length,
        )));
    }
    LIVE_RESULTS.fetch_sub(1, Ordering::SeqCst);
}

fn set_error(message: &str) {
    // SAFETY: the fixed buffer is private to this ABI and the copy is bounded.
    unsafe {
        RESULT_LEN = 0;
        let bytes = message.as_bytes();
        let length = bytes.len().min(4096);
        ERROR_BYTES[..length].copy_from_slice(&bytes[..length]);
        ERROR_LEN = length;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn compile_wasm(source_pointer: *const u8, source_length: usize) -> *mut u8 {
    let source = if source_length == 0 {
        ""
    } else {
        if source_pointer.is_null() {
            set_error("source pointer is null");
            return std::ptr::null_mut();
        }
        // SAFETY: the ABI caller supplies a readable buffer of the declared length.
        let source_bytes = unsafe { std::slice::from_raw_parts(source_pointer, source_length) };
        match std::str::from_utf8(source_bytes) {
            Ok(source) => source,
            Err(_) => {
                set_error("source is not valid UTF-8");
                return std::ptr::null_mut();
            }
        }
    };

    match compile_with_options_named(source, "<input>", 256) {
        Ok(bytes) => {
            let mut boxed = bytes.into_boxed_slice();
            let pointer = boxed.as_mut_ptr();
            let length = boxed.len();
            std::mem::forget(boxed);
            // SAFETY: the ABI is single-threaded in the supported browser/Node hosts.
            unsafe {
                RESULT_LEN = length;
                ERROR_LEN = 0;
            }
            LIVE_RESULTS.fetch_add(1, Ordering::SeqCst);
            pointer
        }
        Err(error) => {
            set_error(&error.to_string());
            std::ptr::null_mut()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_buffers_are_freed_exactly_once() {
        let source = b"int main() { return 0; }";
        let input = alloc_buffer(source.len());
        // SAFETY: the test owns the input buffer for the duration of the call.
        unsafe {
            std::ptr::copy_nonoverlapping(source.as_ptr(), input, source.len());
        }
        let result = compile_wasm(input, source.len());
        assert!(!result.is_null(), "expected a compiled module");
        assert_eq!(LIVE_RESULTS.load(Ordering::SeqCst), 1);
        let length = get_result_len();
        assert!(length > 8);
        free_result(result, length);
        assert_eq!(LIVE_RESULTS.load(Ordering::SeqCst), 0);
        dealloc_buffer(input, source.len());
    }
}
