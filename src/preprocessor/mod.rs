//! Textual preprocessing: conditionals, object/function macros, includes.

mod expr;
mod include;
mod macros;

use crate::diagnostics::CompileError;
use crate::source::{IncludeProvider, NoIncludes};
use macros::MacroDef;
use std::collections::{HashMap, HashSet};

static EMBEDDED_ONLY: NoIncludes = NoIncludes;

/// The source location of one preprocessed output line.
#[derive(Clone, Debug)]
pub struct Origin {
    pub file: String,
    pub line: usize,
}

struct SourceLine {
    text: String,
    file: String,
    line: usize,
}

/// State for one `#if`/`#ifdef`/`#ifndef` nesting level.
struct Conditional {
    parent_active: bool,
    taken: bool,
    active: bool,
    seen_else: bool,
}

impl SourceLine {
    fn empty(file: &str, line: usize) -> Self {
        Self {
            text: String::new(),
            file: file.to_string(),
            line,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreprocessedSource {
    pub text: String,
    origins: Vec<Origin>,
}

impl PreprocessedSource {
    pub fn origin(&self, line: usize) -> Option<&Origin> {
        self.origins.get(line.checked_sub(1)?)
    }
}

impl PreprocessedSource {
    pub fn map_error(&self, error: CompileError) -> CompileError {
        let col = error.col;
        match self.origin(error.line) {
            Some(origin) => error.at(origin.file.clone(), origin.line, col),
            None => error,
        }
    }
}

pub struct Preprocessor<'a> {
    macros: HashMap<String, MacroDef>,
    pragma_once_headers: HashSet<String>,
    includes: &'a dyn IncludeProvider,
    include_depth: usize,
    include_stack: Vec<String>,
    depth: crate::depth::DepthCounter,
}

impl Preprocessor<'static> {
    pub fn new() -> Self {
        Self::new_with_includes(&EMBEDDED_ONLY)
    }
}

impl<'a> Preprocessor<'a> {
    pub fn new_with_includes(includes: &'a dyn IncludeProvider) -> Self {
        let mut macros = HashMap::new();
        macros.insert(
            "__cplusplus".to_string(),
            MacroDef::Object("201703L".to_string()),
        );
        macros.insert(
            "__CPP_WASM__".to_string(),
            MacroDef::Object("1".to_string()),
        );
        Self {
            macros,
            pragma_once_headers: HashSet::new(),
            includes,
            include_depth: 0,
            include_stack: Vec::new(),
            depth: crate::depth::DepthCounter::new(),
        }
    }

    pub fn process(&mut self, input: &str) -> Result<String, CompileError> {
        Ok(self.process_named(input, "<input>")?.text)
    }

    pub fn process_named(
        &mut self,
        input: &str,
        root_file: &str,
    ) -> Result<PreprocessedSource, CompileError> {
        self.include_stack.push(root_file.to_string());
        let result = self.process_lines(input, Some(root_file));
        self.include_stack.pop();
        let lines = result?;
        let text = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let origins = lines
            .into_iter()
            .map(|line| Origin {
                file: line.file,
                line: line.line,
            })
            .collect();
        Ok(PreprocessedSource { text, origins })
    }

    fn process_lines(
        &mut self,
        input: &str,
        current_file: Option<&str>,
    ) -> Result<Vec<SourceLine>, CompileError> {
        let file = current_file.unwrap_or("<input>");
        let mut output_lines = Vec::new();
        let mut if_stack: Vec<Conditional> = Vec::new();
        let (logical_lines, physical_lines) = splice_continuations(input);

        for (line_number, line) in logical_lines {
            let trimmed = line.trim();

            if !trimmed.starts_with('#') {
                let active = if_stack.last().map(|cond| cond.active).unwrap_or(true);
                output_lines.push(SourceLine {
                    text: if active {
                        self.expand_line(&line)
                    } else {
                        String::new()
                    },
                    file: file.to_string(),
                    line: line_number,
                });
                continue;
            }

            let directive_line = trimmed[1..].trim_start();
            let (directive, argument) = split_directive(directive_line);

            if self
                .handle_conditional(directive, argument, &mut if_stack, line_number)
                .map_err(|error| {
                    let col = error.col;
                    error.at(file, line_number, col)
                })?
            {
                output_lines.push(SourceLine::empty(file, line_number));
                continue;
            }

            if !if_stack.last().map(|cond| cond.active).unwrap_or(true) {
                output_lines.push(SourceLine::empty(file, line_number));
                continue;
            }

            match directive {
                "include" => {
                    let expanded = self.expand_line(argument);
                    output_lines.extend(self.handle_include(
                        &expanded,
                        current_file,
                        line_number,
                    )?);
                }
                "define" => {
                    self.handle_define(argument, line_number)?;
                    output_lines.push(SourceLine::empty(file, line_number));
                }
                "pragma" => {
                    self.handle_pragma(argument, current_file);
                    output_lines.push(SourceLine::empty(file, line_number));
                }
                "error" => {
                    let col = directive_column(&line);
                    let message = if argument.is_empty() {
                        "#error".to_string()
                    } else {
                        format!("#error {argument}")
                    };
                    return Err(
                        CompileError::new(message, line_number, col).at(file, line_number, col)
                    );
                }
                "warning" => {
                    let col = directive_column(&line);
                    let message = if argument.is_empty() {
                        "#warning".to_string()
                    } else {
                        format!("#warning {argument}")
                    };
                    eprintln!("{file}:{line_number}:{col}: warning: {message}");
                    output_lines.push(SourceLine::empty(file, line_number));
                }
                _ => output_lines.push(SourceLine::empty(file, line_number)),
            }
        }

        if !if_stack.is_empty() {
            return Err(CompileError::new(
                "Unterminated #if/#ifdef/#ifndef block at EOF",
                physical_lines.max(1),
                1,
            )
            .at(file, physical_lines.max(1), 1));
        }

        Ok(output_lines)
    }

    fn handle_conditional(
        &mut self,
        directive: &str,
        argument: &str,
        if_stack: &mut Vec<Conditional>,
        line_number: usize,
    ) -> Result<bool, CompileError> {
        match directive {
            "if" | "ifdef" | "ifndef" => {
                let parent_active = if_stack.last().map(|cond| cond.active).unwrap_or(true);
                let condition = match directive {
                    "if" => {
                        if parent_active {
                            self.evaluate_condition(argument, line_number)?
                        } else {
                            false
                        }
                    }
                    "ifdef" => self.macros.contains_key(argument.trim()),
                    _ => !self.macros.contains_key(argument.trim()),
                };
                if_stack.push(Conditional {
                    parent_active,
                    taken: condition,
                    active: parent_active && condition,
                    seen_else: false,
                });
                Ok(true)
            }
            "elif" => {
                let Some(previous) = if_stack.last() else {
                    return Err(CompileError::new("#elif without #if", line_number, 1));
                };
                if previous.seen_else {
                    return Err(CompileError::new("#elif after #else", line_number, 1));
                }
                let parent_active = previous.parent_active;
                let already_taken = previous.taken;
                let condition = if parent_active && !already_taken {
                    self.evaluate_condition(argument, line_number)?
                } else {
                    false
                };
                let Some(current) = if_stack.last_mut() else {
                    return Err(CompileError::new("#elif without #if", line_number, 1));
                };
                current.active = parent_active && !already_taken && condition;
                current.taken = already_taken || condition;
                Ok(true)
            }
            "else" => {
                let Some(current) = if_stack.last_mut() else {
                    return Err(CompileError::new("#else without #if", line_number, 1));
                };
                if current.seen_else {
                    return Err(CompileError::new("#else after #else", line_number, 1));
                }
                current.active = current.parent_active && !current.taken;
                current.taken = true;
                current.seen_else = true;
                Ok(true)
            }
            "endif" => {
                if if_stack.pop().is_none() {
                    return Err(CompileError::new("#endif without #if", line_number, 1));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_include(
        &mut self,
        header_spec: &str,
        current_file: Option<&str>,
        line_number: usize,
    ) -> Result<Vec<SourceLine>, CompileError> {
        let file = current_file.unwrap_or("<input>");
        let clean = header_spec.trim_matches(|c| c == '<' || c == '>' || c == '"');
        let Some(resolved) = include::resolve(self.includes, header_spec, current_file) else {
            return Err(CompileError::new(
                format!("cannot find include '{}'", clean),
                line_number,
                1,
            )
            .at(file, line_number, 1));
        };
        if self.pragma_once_headers.contains(&resolved.name) {
            return Ok(vec![SourceLine::empty(file, line_number)]);
        }
        if self.include_stack.iter().any(|name| name == &resolved.name) {
            return Err(CompileError::new(
                format!("include cycle detected: '{}'", resolved.name),
                line_number,
                1,
            )
            .at(file, line_number, 1));
        }
        if self.include_depth >= 128 {
            return Err(
                CompileError::new("include nesting is too deep", line_number, 1).at(
                    file,
                    line_number,
                    1,
                ),
            );
        }
        self.include_depth += 1;
        self.include_stack.push(resolved.name.clone());
        let result = self.process_lines(&resolved.content, Some(&resolved.name));
        self.include_stack.pop();
        self.include_depth -= 1;
        result
    }

    fn handle_define(&mut self, rest: &str, line_number: usize) -> Result<(), CompileError> {
        self.parse_define(rest, line_number)
    }

    fn handle_pragma(&mut self, argument: &str, current_file: Option<&str>) {
        let is_once = argument.split_whitespace().next() == Some("once");
        if !is_once {
            return;
        }
        if let Some(file) = current_file {
            self.pragma_once_headers.insert(file.to_string());
        }
    }
}

impl Default for Preprocessor<'static> {
    fn default() -> Self {
        Self::new()
    }
}

/// Joins physical lines ending in a backslash before any directive, macro, or
/// literal processing. Returns each logical line with its first physical line
/// number, plus the physical line count for EOF diagnostics.
fn splice_continuations(input: &str) -> (Vec<(usize, String)>, usize) {
    let physical: Vec<&str> = input.lines().collect();
    let mut logical = Vec::new();
    let mut index = 0;
    while index < physical.len() {
        let start_line = index + 1;
        let mut text = String::new();
        while index < physical.len() {
            let line = physical[index];
            index += 1;
            if let Some(prefix) = line.strip_suffix('\\') {
                text.push_str(prefix);
            } else {
                text.push_str(line);
                break;
            }
        }
        logical.push((start_line, text));
    }
    (logical, physical.len())
}

/// Returns the 1-based column of the leading `#` on a directive line.
fn directive_column(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count() + 1
}

fn split_directive(directive_line: &str) -> (&str, &str) {
    let trimmed = directive_line.trim();
    match trimmed.split_once(char::is_whitespace) {
        Some((directive, rest)) => (directive, rest.trim_start()),
        None => (trimmed, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::Preprocessor;
    use crate::source::IncludeProvider;
    use std::cell::Cell;

    #[test]
    fn splices_backslash_newlines_before_processing() {
        let (lines, physical) = super::splice_continuations("a\\\nb\nc\\\nd\\\ne\n");
        assert_eq!(lines, vec![(1, "ab".to_string()), (3, "cde".to_string())]);
        assert_eq!(physical, 5);
    }

    #[test]
    fn origins_survive_include_expansion() -> Result<(), Box<dyn std::error::Error>> {
        let mut preprocessor = Preprocessor::new();
        let source = preprocessor.process_named("#include <cstdio>\nint value;", "main.cpp")?;

        assert!(!source.text.contains("cpp_wasm_origin"));
        let last_line = source.text.lines().count();
        let origin = source
            .origin(last_line)
            .ok_or_else(|| std::io::Error::other("missing origin"))?;
        assert_eq!(origin.file, "main.cpp");
        assert_eq!(origin.line, 2);
        Ok(())
    }

    struct CycleIncludes;

    impl IncludeProvider for CycleIncludes {
        fn read_include(&self, name: &str) -> Option<String> {
            match name {
                "cycle_a" => Some("#include <cycle_b>\n".to_string()),
                "cycle_b" => Some("#include <cycle_a>\n".to_string()),
                _ => None,
            }
        }
    }

    #[test]
    fn detects_include_cycles() -> Result<(), Box<dyn std::error::Error>> {
        let includes = CycleIncludes;
        let mut preprocessor = Preprocessor::new_with_includes(&includes);
        let error = match preprocessor.process_named("#include <cycle_a>\n", "main.cpp") {
            Ok(_) => return Err("expected an include-cycle diagnostic".into()),
            Err(error) => error,
        };
        assert!(error.message.contains("include cycle"));
        assert_eq!(error.file, "cycle_b");
        Ok(())
    }

    struct EndlessIncludes {
        next: Cell<usize>,
    }

    impl IncludeProvider for EndlessIncludes {
        fn read_include(&self, _name: &str) -> Option<String> {
            let level = self.next.get();
            self.next.set(level + 1);
            Some(format!("#include <level_{level}>\n"))
        }
    }

    #[test]
    fn evaluates_if_elif_and_defined() -> Result<(), Box<dyn std::error::Error>> {
        let mut preprocessor = Preprocessor::new();
        let source = preprocessor.process(
            "#define A 1\n#define B A\n#if B == 1\nint x = 1;\n#elif 1\nint x = 2;\n#else\nint x = 3;\n#endif\n#if defined(A) && !defined(MISSING)\nint y = 1;\n#endif\n",
        )?;
        assert!(source.contains("int x = 1;"));
        assert!(!source.contains("int x = 2;"));
        assert!(!source.contains("int x = 3;"));
        assert!(source.contains("int y = 1;"));
        Ok(())
    }

    #[test]
    fn expands_macros_in_include_directives() -> Result<(), Box<dyn std::error::Error>> {
        let mut preprocessor = Preprocessor::new();
        let source = preprocessor.process("#define HEADER <cstdio>\n#include HEADER\n")?;
        assert!(source.contains("printf"));
        Ok(())
    }

    #[test]
    fn rejects_deep_if_parentheses_without_overflow() -> Result<(), Box<dyn std::error::Error>> {
        let mut preprocessor = Preprocessor::new();
        let source = format!("#if {}1{}\n#endif\n", "(".repeat(5000), ")".repeat(5000));
        let error = match preprocessor.process(&source) {
            Ok(_) => return Err("expected a nesting-depth diagnostic".into()),
            Err(error) => error,
        };
        assert!(error.message.contains("Nesting depth"));
        Ok(())
    }

    #[test]
    fn rejects_deep_include_nesting() -> Result<(), Box<dyn std::error::Error>> {
        let includes = EndlessIncludes { next: Cell::new(0) };
        let mut preprocessor = Preprocessor::new_with_includes(&includes);
        let error = match preprocessor.process_named("#include <root>\n", "main.cpp") {
            Ok(_) => return Err("expected an include-depth diagnostic".into()),
            Err(error) => error,
        };
        assert!(error.message.contains("too deep"));
        Ok(())
    }
}
