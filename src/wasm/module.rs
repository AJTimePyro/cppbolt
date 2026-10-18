//! The in-memory Wasm module and its binary encoding pass.

use super::leb128::write_i32;
use super::leb128::write_u32;
use super::sections::{write_section, write_string};

/// A function body: local declarations followed by encoded instructions.
pub type FuncBody = (Vec<(u32, u8)>, Vec<u8>);

pub struct WasmModule {
    pub types: Vec<(Vec<u8>, Vec<u8>)>,
    pub imports: Vec<(String, String, u32)>,
    pub functions: Vec<u32>,
    pub table: Option<(u32, Option<u32>)>,
    pub exports: Vec<(String, u8, u32)>,
    pub globals: Vec<(u8, bool, i32)>,
    pub elements: Vec<(u32, Vec<u32>)>,
    pub codes: Vec<FuncBody>,
    pub data: Vec<(u32, Vec<u8>)>,
    pub memory_pages: (u32, Option<u32>),
}

impl Default for WasmModule {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmModule {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            table: None,
            exports: Vec::new(),
            globals: Vec::new(),
            elements: Vec::new(),
            codes: Vec::new(),
            data: Vec::new(),
            memory_pages: (2, Some(256)),
        }
    }

    pub fn add_type(&mut self, params: Vec<u8>, results: Vec<u8>) -> u32 {
        for (index, (existing_params, existing_results)) in self.types.iter().enumerate() {
            if existing_params == &params && existing_results == &results {
                return index as u32;
            }
        }
        let index = self.types.len() as u32;
        self.types.push((params, results));
        index
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\x00asm");
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        self.encode_type_section(&mut out);
        self.encode_import_section(&mut out);
        self.encode_function_section(&mut out);
        self.encode_table_section(&mut out);
        self.encode_memory_section(&mut out);
        self.encode_global_section(&mut out);
        self.encode_export_section(&mut out);
        self.encode_element_section(&mut out);
        self.encode_code_section(&mut out);
        self.encode_data_section(&mut out);
        out
    }

    fn encode_type_section(&self, out: &mut Vec<u8>) {
        if self.types.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.types.len() as u32);
        for (params, results) in &self.types {
            section.push(0x60);
            write_u32(&mut section, params.len() as u32);
            section.extend_from_slice(params);
            write_u32(&mut section, results.len() as u32);
            section.extend_from_slice(results);
        }
        write_section(out, 1, &section);
    }

    fn encode_import_section(&self, out: &mut Vec<u8>) {
        if self.imports.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.imports.len() as u32);
        for (module, field, type_index) in &self.imports {
            write_string(&mut section, module);
            write_string(&mut section, field);
            section.push(0x00);
            write_u32(&mut section, *type_index);
        }
        write_section(out, 2, &section);
    }

    fn encode_function_section(&self, out: &mut Vec<u8>) {
        if self.functions.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.functions.len() as u32);
        for type_index in &self.functions {
            write_u32(&mut section, *type_index);
        }
        write_section(out, 3, &section);
    }

    fn encode_table_section(&self, out: &mut Vec<u8>) {
        let Some((min, max)) = self.table else {
            return;
        };
        let mut section = Vec::new();
        write_u32(&mut section, 1);
        section.push(0x70);
        if let Some(maximum) = max {
            section.push(0x01);
            write_u32(&mut section, min);
            write_u32(&mut section, maximum);
        } else {
            section.push(0x00);
            write_u32(&mut section, min);
        }
        write_section(out, 4, &section);
    }

    fn encode_memory_section(&self, out: &mut Vec<u8>) {
        let mut section = Vec::new();
        write_u32(&mut section, 1);
        if let Some(max) = self.memory_pages.1 {
            section.push(0x01);
            write_u32(&mut section, self.memory_pages.0);
            write_u32(&mut section, max);
        } else {
            section.push(0x00);
            write_u32(&mut section, self.memory_pages.0);
        }
        write_section(out, 5, &section);
    }

    fn encode_global_section(&self, out: &mut Vec<u8>) {
        if self.globals.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.globals.len() as u32);
        for (value_type, is_mutable, initial) in &self.globals {
            section.push(*value_type);
            section.push(u8::from(*is_mutable));
            section.push(0x41);
            write_i32(&mut section, *initial);
            section.push(0x0b);
        }
        write_section(out, 6, &section);
    }

    fn encode_export_section(&self, out: &mut Vec<u8>) {
        if self.exports.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.exports.len() as u32);
        for (name, kind, index) in &self.exports {
            write_string(&mut section, name);
            section.push(*kind);
            write_u32(&mut section, *index);
        }
        write_section(out, 7, &section);
    }

    fn encode_element_section(&self, out: &mut Vec<u8>) {
        if self.elements.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.elements.len() as u32);
        for (offset, functions) in &self.elements {
            section.push(0x00);
            section.push(0x41);
            write_i32(&mut section, *offset as i32);
            section.push(0x0b);
            write_u32(&mut section, functions.len() as u32);
            for function in functions {
                write_u32(&mut section, *function);
            }
        }
        write_section(out, 9, &section);
    }

    fn encode_code_section(&self, out: &mut Vec<u8>) {
        if self.codes.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.codes.len() as u32);
        for (locals, bytecode) in &self.codes {
            let mut body = Vec::new();
            write_u32(&mut body, locals.len() as u32);
            for (count, value_type) in locals {
                write_u32(&mut body, *count);
                body.push(*value_type);
            }
            body.extend_from_slice(bytecode);
            write_u32(&mut section, body.len() as u32);
            section.extend_from_slice(&body);
        }
        write_section(out, 10, &section);
    }

    fn encode_data_section(&self, out: &mut Vec<u8>) {
        if self.data.is_empty() {
            return;
        }
        let mut section = Vec::new();
        write_u32(&mut section, self.data.len() as u32);
        for (offset, bytes) in &self.data {
            section.push(0x00);
            section.push(0x41);
            write_i32(&mut section, *offset as i32);
            section.push(0x0b);
            write_u32(&mut section, bytes.len() as u32);
            section.extend_from_slice(bytes);
        }
        write_section(out, 11, &section);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_an_empty_module_header() {
        let bytes = WasmModule::new().encode();
        assert_eq!(&bytes[..8], b"\x00asm\x01\x00\x00\x00");
    }

    #[test]
    fn interns_repeated_function_types() {
        let mut module = WasmModule::new();
        let first = module.add_type(vec![0x7f], vec![0x7f]);
        let second = module.add_type(vec![0x7f], vec![0x7f]);
        assert_eq!(first, second);
    }
}
