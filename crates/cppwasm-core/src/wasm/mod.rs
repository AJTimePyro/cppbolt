//! WebAssembly binary encoding primitives.

pub mod leb128;
pub mod module;
pub mod sections;

pub use leb128::{write_i32, write_i64, write_u32};
pub use module::{FuncBody, WasmModule};
