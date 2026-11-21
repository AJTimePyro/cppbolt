//! Registration of built-in function signatures.

use super::*;

impl Sema {
    pub(super) fn register_builtins(&mut self) {
        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: "printf".to_string(),
            mangled_name: "printf".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Pointer(Box::new(Type::Char))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });
        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: "std::printf".to_string(),
            mangled_name: "std::printf".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Pointer(Box::new(Type::Char))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: "malloc".to_string(),
            mangled_name: "malloc".to_string(),
            return_type: Type::Pointer(Box::new(Type::Void)),
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: "free".to_string(),
            mangled_name: "free".to_string(),
            return_type: Type::Void,
            param_types: vec![Type::Pointer(Box::new(Type::Void))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: "__builtin_wasm_memory_size".to_string(),
            mangled_name: "__builtin_wasm_memory_size".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });
    }
}
