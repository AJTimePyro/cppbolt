//! Dependency-free C++ subset to WebAssembly compiler.

#![deny(unsafe_code)]
// AST variants are named after their C++ construct (`StructDecl`, `EnumDecl`);
// the redundant suffix is the clearer spelling at every use site.
#![allow(clippy::enum_variant_names)]

pub(crate) mod ast;
pub(crate) mod codegen;
pub(crate) mod diagnostics;
pub mod ffi;
pub(crate) mod lexer;
pub(crate) mod name;
pub(crate) mod parser;
pub(crate) mod preprocessor;
pub(crate) mod runtime;
pub(crate) mod sema;
pub(crate) mod source;

pub(crate) mod wasm;

pub use diagnostics::CompileError;
pub use name::QualifiedName;
pub use source::{FileId, IncludeProvider, NoIncludes, SourceMap, Span};

pub fn preprocess(source: &str) -> Result<String, CompileError> {
    preprocessor::Preprocessor::new().process(source)
}

pub fn compile(source: &str) -> Result<Vec<u8>, CompileError> {
    compile_with_options(source, 256)
}

pub fn compile_with_options(source: &str, max_memory_mb: u32) -> Result<Vec<u8>, CompileError> {
    compile_with_options_named(source, "<input>", max_memory_mb)
}

pub fn compile_with_options_named(
    source: &str,
    filename: &str,
    max_memory_mb: u32,
) -> Result<Vec<u8>, CompileError> {
    compile_with_provider(source, filename, max_memory_mb, &NoIncludes)
}

/// Compiles with a caller-supplied include provider.
///
/// The native CLI passes a filesystem provider; the browser ABI passes none.
pub fn compile_with_provider(
    source: &str,
    filename: &str,
    max_memory_mb: u32,
    includes: &dyn IncludeProvider,
) -> Result<Vec<u8>, CompileError> {
    let mut preprocessor = preprocessor::Preprocessor::new_with_includes(includes);
    let preprocessed = preprocessor.process_named(source, filename)?;

    let mut lexer = lexer::Lexer::new(&preprocessed.text);
    let tokens = lexer
        .tokenize()
        .map_err(|error| preprocessed.map_error(error))?;

    let mut parser = parser::Parser::new(tokens);
    parser
        .validate_supported_features()
        .map_err(|error| preprocessed.map_error(error))?;
    let mut program = parser
        .parse_program()
        .map_err(|error| preprocessed.map_error(error))?;

    let mut sema = sema::Sema::new();
    sema.analyze_program(&mut program)
        .map_err(|error| preprocessed.map_error(error))?;

    let mut codegen = codegen::Codegen::new(&sema);
    codegen.max_memory_mb = max_memory_mb;
    codegen
        .compile_program(&program)
        .map_err(|error| preprocessed.map_error(error))
}

pub fn dump_record_layouts(source: &str) -> Result<String, CompileError> {
    dump_record_layouts_with_provider(source, "<input>", &NoIncludes)
}

pub fn dump_record_layouts_with_provider(
    source: &str,
    filename: &str,
    includes: &dyn IncludeProvider,
) -> Result<String, CompileError> {
    let mut preprocessor = preprocessor::Preprocessor::new_with_includes(includes);
    let preprocessed = preprocessor.process_named(source, filename)?;

    let mut lexer = lexer::Lexer::new(&preprocessed.text);
    let tokens = lexer
        .tokenize()
        .map_err(|error| preprocessed.map_error(error))?;

    let mut parser = parser::Parser::new(tokens);
    let mut program = parser
        .parse_program()
        .map_err(|error| preprocessed.map_error(error))?;

    let mut sema = sema::Sema::new();
    sema.analyze_program(&mut program)
        .map_err(|error| preprocessed.map_error(error))?;

    Ok(sema.dump_record_layouts())
}
