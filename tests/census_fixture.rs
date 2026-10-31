//! The census scanner against a source file with every construct it has to
//! survive: an impl block, a nested closure, match arms, a macro, a string
//! with braces, a char literal with a quote, and a block comment.
//!
//! The expected counts are the ones the independent scanner of the audit
//! reports for the same fixture, so the two scanners can be compared.

use cppbolt::census::census;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The source file of the fixture.
const FIXTURE: &str = r##"use std::collections::HashMap;

struct Holder {
    values: HashMap<String, u32>,
}

impl Holder {
    fn insert(&mut self, key: &str, value: u32) {
        self.values.insert(key.to_string(), value);
    }

    fn total(&self) -> u32 {
        self.values.values().sum()
    }
}

fn classify(value: u32) -> u32 {
    match value {
        0 => {
            let helper = |x: u32| {
                let inner = |y: u32| y + 1;
                inner(x) * 2
            };
            helper(3)
        }
        1..=9 => 1,
        _ => {
            let text = "a { brace } in a string";
            let quote = '"';
            let _ = (text, quote);
            0
        }
    }
}

macro_rules! twice {
    ($value:expr) => {
        $value * 2
    };
}

fn twice_value(value: u32) -> u32 {
    twice!(value)
}

/* a block comment with a } brace */
fn after_comment() -> u32 {
    7
}
"##;

/// A counter that gives every test its own directory.
static DIRECTORIES: AtomicUsize = AtomicUsize::new(0);

/// Writes the fixture into a directory and returns the census of it.
///
/// Every test uses its own directory, so the tests can run in parallel.
fn fixture_census() -> Result<cppbolt::census::Census, String> {
    let unique = DIRECTORIES.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("cppbolt-census-{}-{unique}", std::process::id()));
    let source = root.join("src");
    std::fs::create_dir_all(&source).map_err(|error| error.to_string())?;
    std::fs::write(source.join("fixture.rs"), FIXTURE).map_err(|error| error.to_string())?;
    let result = census(&root).map_err(|error| error.to_string());
    let _ = std::fs::remove_dir_all(&root);
    result
}

/// The functions of the fixture, in the order the scanner finds them.
fn fixture_functions() -> Result<Vec<cppbolt::census::FunctionRecord>, String> {
    Ok(fixture_census()?.functions)
}

#[test]
fn the_fixture_has_the_expected_functions() -> Result<(), String> {
    let names: Vec<String> = fixture_functions()?
        .into_iter()
        .map(|function| function.name)
        .collect();
    assert_eq!(
        names,
        vec![
            "insert",
            "total",
            "classify",
            "twice_value",
            "after_comment",
        ]
    );
    Ok(())
}

#[test]
fn the_fixture_counts_the_parameters_of_every_function() -> Result<(), String> {
    let functions = fixture_functions()?;
    let counts: Vec<(usize, usize)> = functions
        .iter()
        .map(|function| (function.parameters, function.parameters_without_receiver))
        .collect();
    // `insert` takes the receiver, the key, and the value.
    assert_eq!(counts[0], (3, 2));
    // `total` takes the receiver only.
    assert_eq!(counts[1], (1, 0));
    assert_eq!(counts[2], (1, 1));
    assert_eq!(counts[3], (1, 1));
    assert_eq!(counts[4], (0, 0));
    Ok(())
}

#[test]
fn the_fixture_counts_the_lines_of_every_function() -> Result<(), String> {
    let lengths: Vec<usize> = fixture_functions()?
        .iter()
        .map(|function| function.length)
        .collect();
    assert_eq!(lengths, vec![3, 3, 18, 3, 3]);
    Ok(())
}

#[test]
fn a_brace_in_a_string_does_not_end_a_function() -> Result<(), String> {
    let functions = fixture_functions()?;
    let classify = functions
        .iter()
        .find(|function| function.name == "classify")
        .ok_or_else(|| "the fixture has no function classify".to_string())?;
    // The string with a brace and the char literal are inside the body of
    // `classify`, so the body ends at its own closing brace and not at the
    // brace of the string.
    let closing = FIXTURE
        .lines()
        .enumerate()
        .filter(|(_, line)| *line == "}")
        .map(|(index, _)| index + 1)
        .last()
        .ok_or_else(|| "the fixture has no closing brace".to_string())?;
    assert!(classify.end_line < closing);
    assert_eq!(classify.length, 18);
    Ok(())
}

#[test]
fn a_block_comment_does_not_end_a_function() -> Result<(), String> {
    let functions = fixture_functions()?;
    let after = functions
        .iter()
        .find(|function| function.name == "after_comment")
        .ok_or_else(|| "the fixture has no function after_comment".to_string())?;
    assert_eq!(after.length, 3);
    Ok(())
}

#[test]
fn the_census_counts_the_files_and_the_functions() -> Result<(), String> {
    let result = fixture_census()?;
    assert_eq!(result.functions.len(), 5);
    assert_eq!(
        result.file_lines.get("src/fixture.rs"),
        Some(&FIXTURE.lines().count())
    );
    assert_eq!(result.over_length(0), 5);
    assert_eq!(result.over_parameters(2), 1);
    assert_eq!(result.over_parameters_without_receiver(2), 0);
    Ok(())
}

#[test]
fn the_census_of_the_repository_has_no_long_function() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let result = census(&root).map_err(|error| error.to_string())?;
    let long: Vec<String> = result
        .functions
        .iter()
        .filter(|function| function.length > 80)
        .map(|function| {
            format!(
                "{}:{} {}",
                function.file, function.start_line, function.name
            )
        })
        .collect();
    assert!(
        long.is_empty(),
        "functions over 80 lines:\n{}",
        long.join("\n")
    );
    let wide: Vec<String> = result
        .functions
        .iter()
        .filter(|function| function.parameters_without_receiver > 5)
        .map(|function| {
            format!(
                "{}:{} {}",
                function.file, function.start_line, function.name
            )
        })
        .collect();
    assert!(
        wide.is_empty(),
        "functions with more than 5 parameters:\n{}",
        wide.join("\n")
    );
    let files: Vec<String> = result
        .file_lines
        .iter()
        .filter(|(_, lines)| **lines > 500)
        .map(|(file, lines)| format!("{file} has {lines} lines"))
        .collect();
    assert!(
        files.is_empty(),
        "files over 500 lines:\n{}",
        files.join("\n")
    );
    Ok(())
}
