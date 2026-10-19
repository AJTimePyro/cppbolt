#![forbid(unsafe_code)]

use cppwasm::{IncludeProvider, compile_with_provider};
use std::env;
use std::fs;
use std::process;

/// Resolves quoted includes against the process working directory, matching
/// the native tool's historical `#include "path"` behavior.
struct FilesystemIncludes;

impl IncludeProvider for FilesystemIncludes {
    fn read_include(&self, name: &str) -> Option<String> {
        fs::read_to_string(name).ok()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cppwasm-cli [--dump-record-layouts] [-o <out.wasm>] <input.cpp>");
        process::exit(1);
    }

    let mut output_file = "a.out.wasm".to_string();
    let mut input_file = None;
    let mut dump_layouts = false;
    let mut max_memory_mb = 256u32;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--dump-record-layouts" => {
                dump_layouts = true;
                index += 1;
            }
            "-o" if index + 1 < args.len() => {
                output_file = args[index + 1].clone();
                index += 2;
            }
            "--max-memory" | "--max-memory-mb" if index + 1 < args.len() => {
                let Some(value) = args[index + 1].parse().ok() else {
                    eprintln!("Error: invalid memory limit '{}'", args[index + 1]);
                    process::exit(1);
                };
                max_memory_mb = value;
                index += 2;
            }
            "-I" if index + 1 < args.len() => index += 2,
            option if option.starts_with("-I") => index += 1,
            option if !option.starts_with('-') => {
                input_file = Some(option.to_string());
                index += 1;
            }
            _ => index += 1,
        }
    }

    let Some(input_path) = input_file else {
        eprintln!("Error: No input file specified");
        process::exit(1);
    };

    let source = match fs::read_to_string(&input_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Error reading '{}': {}", input_path, error);
            process::exit(1);
        }
    };

    let includes = FilesystemIncludes;
    if dump_layouts {
        match cppwasm::dump_record_layouts_with_provider(&source, &input_path, &includes) {
            Ok(layouts) => {
                print!("{}", layouts);
                process::exit(0);
            }
            Err(error) => {
                eprintln!("{}", error);
                process::exit(1);
            }
        }
    }

    match compile_with_provider(&source, &input_path, max_memory_mb, &includes) {
        Ok(wasm_bytes) => {
            if let Err(error) = fs::write(&output_file, wasm_bytes) {
                eprintln!("Error writing '{}': {}", output_file, error);
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}
