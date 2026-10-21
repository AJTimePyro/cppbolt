//! Physical-line handling for the preprocessor.

/// Joins physical lines ending in a backslash before any directive, macro, or
/// literal processing. Returns each logical line with its first physical line
/// number, plus the physical line count for EOF diagnostics.
pub(super) fn splice_continuations(input: &str) -> (Vec<(usize, String)>, usize) {
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
pub(super) fn directive_column(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count() + 1
}

pub(super) fn split_directive(directive_line: &str) -> (&str, &str) {
    let trimmed = directive_line.trim();
    match trimmed.split_once(char::is_whitespace) {
        Some((directive, rest)) => (directive, rest.trim_start()),
        None => (trimmed, ""),
    }
}
