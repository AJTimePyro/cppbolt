//! Dependency-free C++ subset to WebAssembly compiler.
//!
//! See the [mdBook Documentation](https://ajtimepyro.is-a.dev/cppbolt/book/) for architecture details.

#![deny(unsafe_code)]
#![warn(missing_docs)]
// AST variants are named after their C++ construct (`StructDecl`, `EnumDecl`);
// the redundant suffix is the clearer spelling at every use site.
#![allow(clippy::enum_variant_names)]

pub(crate) mod ast;
pub mod census;
pub(crate) mod codegen;
pub mod corpus;
pub(crate) mod depth;
pub(crate) mod diagnostics;
pub mod ffi;
pub(crate) mod lexer;
pub(crate) mod name;
pub mod oracle;
pub(crate) mod parser;
pub(crate) mod preprocessor;
pub(crate) mod runtime;
pub(crate) mod sema;
pub(crate) mod source;

pub(crate) mod wasm;

pub use diagnostics::CompileError;
pub use name::QualifiedName;
pub use source::{FileId, IncludeProvider, NoIncludes, ResolvedInclude, Span};

/// The nesting limits of the compiler, one per nesting construct.
///
/// The limits are measured on the WebAssembly-hosted build under Node/V8 and
/// apply to the native build as well, so the same input is accepted or
/// rejected on both hosts.
#[derive(Debug, Clone, Copy)]
pub struct NestingLimits {
    /// Nested expressions.
    pub expressions: usize,
    /// Nested parentheses.
    pub parentheses: usize,
    /// Nested call argument lists.
    pub calls: usize,
    /// Nested statements.
    pub statements: usize,
    /// Nested compound blocks.
    pub blocks: usize,
    /// Nested declarations.
    pub declarations: usize,
    /// Nested type expressions.
    pub types: usize,
    /// Nested preprocessor conditions.
    pub preprocessor: usize,
}

/// The nesting limits of the compiler.
pub fn nesting_limits() -> NestingLimits {
    NestingLimits {
        expressions: depth::MAX_EXPRESSION_DEPTH,
        parentheses: depth::MAX_PARENTHESIS_DEPTH,
        calls: depth::MAX_CALL_DEPTH,
        statements: depth::MAX_STATEMENT_DEPTH,
        blocks: depth::MAX_BLOCK_DEPTH,
        declarations: depth::MAX_DECLARATION_DEPTH,
        types: depth::MAX_TYPE_DEPTH,
        preprocessor: depth::MAX_PREPROCESSOR_DEPTH,
    }
}

/// Runs only the preprocessor and returns the expanded source.
pub fn preprocess(source: &str) -> Result<String, CompileError> {
    preprocessor::Preprocessor::new().process(source)
}

/// Compiles with the default 256 MiB memory limit.
pub fn compile(source: &str) -> Result<Vec<u8>, CompileError> {
    compile_with_options(source, 256)
}

/// Compiles with a custom maximum linear-memory size in MiB.
pub fn compile_with_options(source: &str, max_memory_mb: u32) -> Result<Vec<u8>, CompileError> {
    compile_with_options_named(source, "<input>", max_memory_mb)
}

/// Compiles with a diagnostic filename and a custom memory limit.
pub fn compile_with_options_named(
    source: &str,
    filename: &str,
    max_memory_mb: u32,
) -> Result<Vec<u8>, CompileError> {
    compile_with_provider(source, filename, max_memory_mb, &NoIncludes)
}

/// Preprocesses, parses, validates, and analyzes `source`.
fn parse_and_analyze(
    source: &str,
    filename: &str,
    includes: &dyn IncludeProvider,
) -> Result<(preprocessor::PreprocessedSource, ast::Program, sema::Sema), CompileError> {
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
    Ok((preprocessed, program, sema))
}

/// Compiles with a caller-supplied include provider.
///
/// The native CLI passes a filesystem provider; the browser ABI passes none.
/// The returned bytes are a complete WebAssembly module.
pub fn compile_with_provider(
    source: &str,
    filename: &str,
    max_memory_mb: u32,
    includes: &dyn IncludeProvider,
) -> Result<Vec<u8>, CompileError> {
    let (preprocessed, program, sema) = parse_and_analyze(source, filename, includes)?;

    let mut codegen = codegen::Codegen::new(&sema);
    codegen.max_memory_mb = max_memory_mb;
    codegen
        .compile_program(&program)
        .map_err(|error| preprocessed.map_error(error))
}

/// Renders the compiler's record layout report for `source`.
pub fn dump_record_layouts(source: &str) -> Result<String, CompileError> {
    dump_record_layouts_with_provider(source, "<input>", &NoIncludes)
}

/// Renders the record layout report using a caller-supplied include provider.
pub fn dump_record_layouts_with_provider(
    source: &str,
    filename: &str,
    includes: &dyn IncludeProvider,
) -> Result<String, CompileError> {
    let (_preprocessed, _program, sema) = parse_and_analyze(source, filename, includes)?;
    Ok(sema.dump_record_layouts())
}
