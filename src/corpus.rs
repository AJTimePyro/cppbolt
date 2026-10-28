//! The corpus of the compiler: the feature groups, the test files, and the
//! comparison of a program output with the recorded expectation.
//!
//! The corpus runner, the golden baseline, and the native oracle share this
//! module, so a test that is discovered by one gate is discovered by all of
//! them.

use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The feature groups of the corpus.
pub const FEATURES: &[&str] = &[
    "c_basics",
    "c_basics_variant",
    "classes_oop",
    "classes_oop_variant",
    "memory_model",
    "memory_model_variant",
    "runtime_trap",
    "templates_operators",
    "mini_stl",
    "mini_stl_variant",
    "independent",
];

/// Lists the C++ sources of a directory, recursively traversing subdirectories.
/// Results are sorted deterministically.
pub fn cpp_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_cpp_files(directory, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_cpp_files(directory: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_cpp_files(&path, out)?;
        } else if path.extension() == Some(OsStr::new("cpp")) {
            out.push(path);
        }
    }
    Ok(())
}

/// The stdin file of a test, if the test has one.
pub fn input_path(feature: &str, name: &str) -> Option<PathBuf> {
    let path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.input"));
    path.exists().then_some(path)
}

/// Runs a native program with the recorded input.
pub fn run_with_input(
    binary: &Path,
    input: Option<&Path>,
) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("timeout");
    command.arg("5s").arg(binary);
    match input {
        Some(path) => {
            let text = fs::read(path)?;
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&text)?;
            }
            child.wait_with_output()
        }
        None => command.output(),
    }
}

/// Compares the output of a program with the recorded expectation.
///
/// The expected file holds the stdout and the stderr of the program, in that
/// order.
pub fn compare_output(
    output: &std::process::Output,
    feature: &str,
    name: &str,
) -> Result<(), String> {
    if !output.status.success() {
        return Err(format!("runtime exit {}", output.status));
    }
    let expected_path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.expected"));
    let expected = fs::read(&expected_path)
        .map_err(|error| format!("cannot read {}: {error}", expected_path.display()))?;
    let mut actual = output.stdout.clone();
    actual.extend_from_slice(&output.stderr);
    if actual == expected {
        Ok(())
    } else {
        Err("output mismatch".to_string())
    }
}

/// Extracts the relative test identifier from a test source path given the feature directory.
/// Returns a normalized path with forward slashes and no `.cpp` extension.
pub fn test_name(feature_dir: &Path, cpp_path: &Path) -> Option<String> {
    let rel = cpp_path.strip_prefix(feature_dir).ok()?;
    let without_ext = rel.with_extension("");
    Some(without_ext.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cpp_files_recursively_finds_and_sorts_sources() -> Result<(), Box<dyn std::error::Error>> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("{error}"))?
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("cppwasm_corpus_test_{nonce}"));
        let sub = temp.join("sub");
        fs::create_dir_all(&sub)?;

        fs::write(temp.join("root.cpp"), "")?;
        fs::write(temp.join("ignored.txt"), "")?;
        fs::write(sub.join("nested.cpp"), "")?;

        let files = cpp_files(&temp)?;
        let names: Vec<String> = files.iter().filter_map(|p| test_name(&temp, p)).collect();

        let _ = fs::remove_dir_all(&temp);

        assert_eq!(names, vec!["root".to_string(), "sub/nested".to_string()]);
        Ok(())
    }
}
