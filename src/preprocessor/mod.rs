//! Textual preprocessing: conditionals, object/function macros, includes.

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
        let lines = self.process_lines(input, Some(root_file))?;
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
        let mut if_stack: Vec<bool> = Vec::new();

        for (line_index, line) in input.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed = line.trim();

            if !trimmed.starts_with('#') {
                let active = if_stack.last().copied().unwrap_or(true);
                output_lines.push(SourceLine {
                    text: if active {
                        self.expand_line(line)
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

            if self.handle_conditional(directive, argument, &mut if_stack, line_number)? {
                output_lines.push(SourceLine::empty(file, line_number));
                continue;
            }

            if !if_stack.last().copied().unwrap_or(true) {
                output_lines.push(SourceLine::empty(file, line_number));
                continue;
            }

            match directive {
                "include" => {
                    output_lines.extend(self.handle_include(
                        argument,
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
                _ => output_lines.push(SourceLine::empty(file, line_number)),
            }
        }

        if !if_stack.is_empty() {
            return Err(CompileError::new(
                "Unterminated #ifdef / #ifndef block at EOF",
                input.lines().count(),
                1,
            ));
        }

        Ok(output_lines)
    }

    fn handle_conditional(
        &mut self,
        directive: &str,
        argument: &str,
        if_stack: &mut Vec<bool>,
        line_number: usize,
    ) -> Result<bool, CompileError> {
        match directive {
            "ifdef" => {
                let parent = if_stack.last().copied().unwrap_or(true);
                if_stack.push(parent && self.macros.contains_key(argument));
                Ok(true)
            }
            "ifndef" => {
                let parent = if_stack.last().copied().unwrap_or(true);
                if_stack.push(parent && !self.macros.contains_key(argument));
                Ok(true)
            }
            "else" => {
                let Some(active) = if_stack.pop() else {
                    return Err(CompileError::new(
                        "#else without #ifdef/#ifndef",
                        line_number,
                        1,
                    ));
                };
                let parent = if_stack.last().copied().unwrap_or(true);
                if_stack.push(parent && !active);
                Ok(true)
            }
            "endif" => {
                if if_stack.pop().is_none() {
                    return Err(CompileError::new(
                        "#endif without #ifdef/#ifndef",
                        line_number,
                        1,
                    ));
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
        let clean = header_spec.trim_matches(|c| c == '<' || c == '>' || c == '"');
        if self.pragma_once_headers.contains(clean) {
            return Ok(vec![SourceLine::empty(
                current_file.unwrap_or("<input>"),
                line_number,
            )]);
        }

        let Some(content) = include::resolve(self.includes, header_spec) else {
            return Ok(vec![SourceLine::empty(
                current_file.unwrap_or("<input>"),
                line_number,
            )]);
        };
        self.process_lines(&content, Some(clean))
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
}
