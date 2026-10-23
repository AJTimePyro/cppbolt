//! Handling of one preprocessor line.

use super::lines::{directive_column, split_directive};
use super::*;
use crate::diagnostics::CompileError;

impl<'a> Preprocessor<'a> {
    /// Preprocesses one logical line and appends the produced lines.
    pub(super) fn process_line(
        &mut self,
        line: &str,
        line_number: usize,
        state: &mut LineState<'_>,
    ) -> Result<(), CompileError> {
        let file = state.file;
        let trimmed = line.trim();

        if !trimmed.starts_with('#') {
            let active = state
                .if_stack
                .last()
                .map(|cond| cond.active)
                .unwrap_or(true);
            state.output_lines.push(SourceLine {
                text: if active {
                    self.expand_line(line)
                } else {
                    String::new()
                },
                file: file.to_string(),
                line: line_number,
            });
            return Ok(());
        }

        let directive_line = trimmed[1..].trim_start();
        let (directive, argument) = split_directive(directive_line);

        if self
            .handle_conditional(directive, argument, &mut state.if_stack, line_number)
            .map_err(|error| {
                let col = error.col;
                error.at(file, line_number, col)
            })?
        {
            state
                .output_lines
                .push(SourceLine::empty(file, line_number));
            return Ok(());
        }

        if !state
            .if_stack
            .last()
            .map(|cond| cond.active)
            .unwrap_or(true)
        {
            state
                .output_lines
                .push(SourceLine::empty(file, line_number));
            return Ok(());
        }

        self.handle_directive(directive, argument, line, line_number, state)
    }

    /// Handles one directive.
    fn handle_directive(
        &mut self,
        directive: &str,
        argument: &str,
        line: &str,
        line_number: usize,
        state: &mut LineState<'_>,
    ) -> Result<(), CompileError> {
        let file = state.file;
        match directive {
            "include" => {
                let expanded = self.expand_line(argument);
                let included = self.handle_include(&expanded, state.current_file, line_number)?;
                state.output_lines.extend(included);
            }
            "define" => {
                self.parse_define(argument, line_number)?;
                state
                    .output_lines
                    .push(SourceLine::empty(file, line_number));
            }
            "pragma" => {
                self.handle_pragma(argument, state.current_file);
                state
                    .output_lines
                    .push(SourceLine::empty(file, line_number));
            }
            "error" => {
                let col = directive_column(line);
                let message = if argument.is_empty() {
                    "#error".to_string()
                } else {
                    format!("#error {argument}")
                };
                return Err(CompileError::new(message, line_number, col).at(
                    file,
                    line_number,
                    col,
                ));
            }
            "warning" => {
                let col = directive_column(line);
                let message = if argument.is_empty() {
                    "#warning".to_string()
                } else {
                    format!("#warning {argument}")
                };
                eprintln!("{file}:{line_number}:{col}: warning: {message}");
                state
                    .output_lines
                    .push(SourceLine::empty(file, line_number));
            }
            _ => state
                .output_lines
                .push(SourceLine::empty(file, line_number)),
        }
        Ok(())
    }
}
