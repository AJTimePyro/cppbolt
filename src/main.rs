#![forbid(unsafe_code)]

use cppbolt::{IncludeProvider, ResolvedInclude, compile_with_provider};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;

/// Resolves includes against the including file, `-I` directories, and finally
/// the process working directory.
#[derive(Clone)]
struct FilesystemIncludes {
    include_dirs: Vec<PathBuf>,
}

impl IncludeProvider for FilesystemIncludes {
    fn read_include(&self, name: &str) -> Option<String> {
        fs::read_to_string(name).ok()
    }

    fn resolve_include(
        &self,
        name: &str,
        including_file: Option<&str>,
        quoted: bool,
    ) -> Option<ResolvedInclude> {
        let mut candidates = Vec::new();
        if quoted
            && let Some(including) = including_file
            && !including.starts_with('<')
            && let Some(parent) = Path::new(including).parent()
            && !parent.as_os_str().is_empty()
        {
            candidates.push(parent.join(name));
        }
        for directory in &self.include_dirs {
            candidates.push(directory.join(name));
        }
        candidates.push(PathBuf::from(name));
        for path in candidates {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(ResolvedInclude {
                    name: clean_path(&path).to_string_lossy().into_owned(),
                    content,
                });
            }
        }
        None
    }
}

/// Removes `.` components so the same file reached as `./x.h` and `x.h` has a
/// single identity for `#pragma once` and cycle detection.
fn clean_path(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();
    for component in path.components() {
        if component != Component::CurDir {
            cleaned.push(component.as_os_str());
        }
    }
    cleaned
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cppbolt-cli [--dump-record-layouts] [-o <out.wasm>] <input.cpp>");
        process::exit(1);
    }
    let options = parse_args(&args);

    let source = match fs::read_to_string(&options.input_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Error reading '{}': {}", options.input_path, error);
            process::exit(1);
        }
    };

    if options.dump_layouts {
        match cppbolt::dump_record_layouts_with_provider(
            &source,
            &options.input_path,
            &options.includes,
        ) {
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

    match compile_source(&source, &options) {
        Ok(wasm_bytes) => {
            if let Err(error) = fs::write(&options.output_file, wasm_bytes) {
                eprintln!("Error writing '{}': {}", options.output_file, error);
                process::exit(1);
            }
        }
        Err(message) => {
            eprintln!("{}", message);
            process::exit(1);
        }
    }
}

struct CliOptions {
    output_file: String,
    input_path: String,
    dump_layouts: bool,
    max_memory_mb: u32,
    includes: FilesystemIncludes,
}

fn parse_args(args: &[String]) -> CliOptions {
    let mut output_file = "a.out.wasm".to_string();
    let mut input_file = None;
    let mut dump_layouts = false;
    let mut max_memory_mb = 256u32;
    let mut include_dirs = Vec::new();

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
            "-I" if index + 1 < args.len() => {
                include_dirs.push(PathBuf::from(&args[index + 1]));
                index += 2;
            }
            "-I" => {
                eprintln!("Error: -I requires a directory");
                process::exit(1);
            }
            option if option.starts_with("-I") => {
                include_dirs.push(PathBuf::from(&option[2..]));
                index += 1;
            }
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

    CliOptions {
        output_file,
        input_path,
        dump_layouts,
        max_memory_mb,
        includes: FilesystemIncludes { include_dirs },
    }
}

fn compile_source(source: &str, options: &CliOptions) -> Result<Vec<u8>, String> {
    // Run the compiler on a thread with a large stack so the shared depth
    // limit is governed by the wasm-hosted build's measured threshold rather
    // than by the platform's default native stack.
    let source = source.to_string();
    let input_path = options.input_path.clone();
    let includes = options.includes.clone();
    let max_memory_mb = options.max_memory_mb;
    match std::thread::Builder::new()
        .name("cppbolt-compile".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            compile_with_provider(&source, &input_path, max_memory_mb, &includes)
                .map_err(|error| error.to_string())
        }) {
        Ok(handle) => match handle.join() {
            Ok(result) => result,
            Err(_) => Err("compiler thread panicked".to_string()),
        },
        Err(error) => Err(format!("cannot start compiler thread: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_quoted_include_relative_to_the_including_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let includes = FilesystemIncludes {
            include_dirs: Vec::new(),
        };
        let source = "#include \"nested/value.h\"\n\nint main() { return value(); }\n";
        let wasm =
            compile_with_provider(source, "tests/include_paths/relative.cpp", 256, &includes)?;
        assert!(!wasm.is_empty());
        Ok(())
    }

    #[test]
    fn searches_include_directories_for_angle_includes() -> Result<(), Box<dyn std::error::Error>> {
        let includes = FilesystemIncludes {
            include_dirs: vec![PathBuf::from("tests/include_paths/include")],
        };
        let source = "#include <math_util.h>\n\nint main() { return add(2, 3) - 5; }\n";
        let wasm = compile_with_provider(source, "tests/include_paths/direct.cpp", 256, &includes)?;
        assert!(!wasm.is_empty());
        Ok(())
    }

    #[test]
    fn cleans_duplicate_relative_components() {
        let cleaned = clean_path(Path::new("./tests/./include_paths/nested/value.h"));
        assert_eq!(cleaned, PathBuf::from("tests/include_paths/nested/value.h"));
    }
}
