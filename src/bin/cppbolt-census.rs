//! Prints the census of the Rust sources: the size of every function, the
//! number of its parameters, and the size of every file.
//!
//! Usage: `cppbolt-census [root] [--tsv <path>]`
//!
//! The `--tsv` option writes one record per function, so the census can be
//! compared with another scanner record by record.

use cppbolt::census::{Census, FunctionRecord, census};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PARAMETER_LIMIT: usize = 5;
const FILE_LINE_LIMIT: usize = 500;

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut tsv: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--tsv" => match args.next() {
                Some(path) => tsv = Some(PathBuf::from(path)),
                None => {
                    eprintln!("--tsv needs a path");
                    return ExitCode::FAILURE;
                }
            },
            _ => root = PathBuf::from(argument),
        }
    }
    let result = match census(&root) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("cannot read {}: {error}", root.display());
            return ExitCode::FAILURE;
        }
    };

    if let Some(path) = tsv
        && let Err(error) = write_tsv(&path, &result)
    {
        eprintln!("cannot write {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    print_counts(&result);
    print_longest(&result);
    print_parameters(&result);
    print_files(&result);
    ExitCode::SUCCESS
}

/// Prints the number of files, the number of functions, and the number of
/// functions over each length limit.
fn print_counts(result: &Census) {
    println!("rust files scanned: {}", result.file_lines.len());
    println!("functions found:    {}", result.functions.len());
    println!();
    for limit in [60, 80, 100] {
        println!("functions > {limit} lines: {}", result.over_length(limit));
    }
    println!();
}

/// Prints the fifteen longest functions.
fn print_longest(result: &Census) {
    println!("15 LONGEST FUNCTIONS");
    println!("length | file:line | fn | params");
    let longest: Vec<&FunctionRecord> = result.longest(0);
    for function in longest.iter().take(15) {
        println!(
            "{:6} | {}:{} | {} | {}",
            function.length, function.file, function.start_line, function.name, function.parameters
        );
    }
    println!();
}

/// Prints the functions with more than five parameters.
fn print_parameters(result: &Census) {
    println!(
        "functions with more than {PARAMETER_LIMIT} parameters: {}",
        result.over_parameters(PARAMETER_LIMIT)
    );
    println!(
        "functions with more than {PARAMETER_LIMIT} parameters (excluding self): {}",
        result.over_parameters_without_receiver(PARAMETER_LIMIT)
    );
    for function in result
        .functions
        .iter()
        .filter(|function| function.parameters > PARAMETER_LIMIT)
    {
        println!(
            "  {:2} params | {}:{} | {}",
            function.parameters, function.file, function.start_line, function.name
        );
    }
    println!();
}

/// Prints the files with more than five hundred lines.
fn print_files(result: &Census) {
    println!("FILES OVER 500 LINES");
    let mut files: Vec<(&String, &usize)> = result.file_lines.iter().collect();
    files.sort_by(|left, right| right.1.cmp(left.1));
    for (file, lines) in &files {
        if **lines > FILE_LINE_LIMIT {
            println!("  {lines:5}  {file}");
        }
    }
    println!(
        "files over {FILE_LINE_LIMIT} lines: {} of {}",
        result.files_over(FILE_LINE_LIMIT),
        result.file_lines.len()
    );
}

/// Writes one record per function, in the format of the audit scanner.
fn write_tsv(path: &Path, result: &Census) -> std::io::Result<()> {
    let mut text =
        String::from("file\tstart_line\tend_line\tlength\tparams\tparams_no_self\tname\n");
    let mut functions: Vec<&FunctionRecord> = result.functions.iter().collect();
    functions
        .sort_by(|left, right| (&left.file, left.start_line).cmp(&(&right.file, right.start_line)));
    for function in functions {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            function.file,
            function.start_line,
            function.end_line,
            function.length,
            function.parameters,
            function.parameters_without_receiver,
            function.name
        ));
    }
    std::fs::write(path, text)
}
