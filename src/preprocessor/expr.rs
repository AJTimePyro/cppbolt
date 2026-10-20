//! Integer expression evaluation for `#if` and `#elif`.
//!
//! The evaluator uses an explicit shunting-yard stack so deeply nested
//! parentheses consume heap memory, not Rust stack frames. Parenthesis depth
//! is still charged against the shared compiler depth limit.

use super::Preprocessor;
use crate::depth::{DepthCounter, DepthGuard};
use crate::diagnostics::CompileError;
use std::collections::HashMap;

impl Preprocessor<'_> {
    pub(super) fn evaluate_condition(
        &self,
        argument: &str,
        line_number: usize,
    ) -> Result<bool, CompileError> {
        let replaced = replace_defined(argument, &self.macros);
        let expanded = self.expand_line(&replaced);
        let value = IfExpr::new(&expanded, &self.depth, line_number).evaluate()?;
        Ok(value != 0)
    }
}

/// Rewrites `defined(NAME)` and `defined NAME` before macro expansion, as the
/// standard requires.
fn replace_defined(argument: &str, macros: &HashMap<String, super::macros::MacroDef>) -> String {
    let chars: Vec<char> = argument.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if !is_ident_start(chars[index]) {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && is_ident_continue(chars[index]) {
            index += 1;
        }
        let ident: String = chars[start..index].iter().collect();
        if ident != "defined" {
            out.push_str(&ident);
            continue;
        }
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        let parenthesized = chars.get(index) == Some(&'(');
        if parenthesized {
            index += 1;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
        }
        let name_start = index;
        while index < chars.len() && is_ident_continue(chars[index]) {
            index += 1;
        }
        let name: String = chars[name_start..index].iter().collect();
        if parenthesized {
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            if chars.get(index) == Some(&')') {
                index += 1;
            }
        }
        out.push(if macros.contains_key(&name) { '1' } else { '0' });
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    LParen,
    Question,
    Colon,
    Binary(&'static str),
    Unary(&'static str),
}

impl Op {
    fn precedence(self) -> u8 {
        match self {
            Op::LParen | Op::Question | Op::Colon => 0,
            Op::Binary("||") => 1,
            Op::Binary("&&") => 2,
            Op::Binary("|") => 3,
            Op::Binary("^") => 4,
            Op::Binary("&") => 5,
            Op::Binary("==" | "!=") => 6,
            Op::Binary("<" | "<=" | ">" | ">=") => 7,
            Op::Binary("<<" | ">>") => 8,
            Op::Binary("+" | "-") => 9,
            Op::Binary("*" | "/" | "%") => 10,
            Op::Binary(_) => 0,
            Op::Unary(_) => 11,
        }
    }
}

struct IfExpr<'a> {
    chars: Vec<char>,
    pos: usize,
    depth: &'a DepthCounter,
    line: usize,
}

impl<'a> IfExpr<'a> {
    fn new(text: &str, depth: &'a DepthCounter, line: usize) -> Self {
        Self {
            chars: text.chars().collect(),
            pos: 0,
            depth,
            line,
        }
    }

    fn evaluate(mut self) -> Result<i64, CompileError> {
        let tokens = self.tokenize()?;
        let mut values: Vec<i64> = Vec::new();
        let mut operators: Vec<Op> = Vec::new();
        let mut paren_guards: Vec<DepthGuard> = Vec::new();
        let mut expect_operand = true;

        for token in tokens {
            match token {
                Token::Number(value) => {
                    values.push(value);
                    expect_operand = false;
                }
                Token::LParen => {
                    paren_guards.push(self.depth.enter(self.line, 1)?);
                    operators.push(Op::LParen);
                    expect_operand = true;
                }
                Token::RParen => {
                    while let Some(op) = operators.last().copied() {
                        match op {
                            Op::LParen => break,
                            Op::Question | Op::Colon => {
                                return Err(self.error("expected ':' before ')' in #if expression"));
                            }
                            _ => {
                                operators.pop();
                                reduce(&mut values, op).map_err(|message| self.error(message))?;
                            }
                        }
                    }
                    if operators.pop() != Some(Op::LParen) {
                        return Err(self.error("unmatched ')' in #if expression"));
                    }
                    if paren_guards.pop().is_none() {
                        return Err(self.error("unmatched ')' in #if expression"));
                    }
                    expect_operand = false;
                }
                Token::Question => {
                    // The condition is complete; reduce operators that bind
                    // more tightly than the conditional operator.
                    while let Some(op) = operators.last().copied() {
                        if matches!(op, Op::LParen | Op::Question | Op::Colon) {
                            break;
                        }
                        operators.pop();
                        reduce(&mut values, op).map_err(|message| self.error(message))?;
                    }
                    operators.push(Op::Question);
                    expect_operand = true;
                }
                Token::Colon => {
                    while let Some(op) = operators.last().copied() {
                        match op {
                            Op::Question | Op::LParen => break,
                            _ => {
                                operators.pop();
                                reduce(&mut values, op).map_err(|message| self.error(message))?;
                            }
                        }
                    }
                    if operators.pop() != Some(Op::Question) {
                        return Err(self.error("expected '?' before ':' in #if expression"));
                    }
                    operators.push(Op::Colon);
                    expect_operand = true;
                }
                Token::Operator(text) => {
                    let op = if matches!(text, "!" | "~")
                        || (expect_operand && matches!(text, "+" | "-"))
                    {
                        Op::Unary(text)
                    } else {
                        Op::Binary(text)
                    };
                    if matches!(op, Op::Binary(_)) {
                        while let Some(top) = operators.last().copied() {
                            if matches!(top, Op::LParen | Op::Question | Op::Colon) {
                                break;
                            }
                            if top.precedence() < op.precedence() {
                                break;
                            }
                            operators.pop();
                            reduce(&mut values, top).map_err(|message| self.error(message))?;
                        }
                    }
                    operators.push(op);
                    expect_operand = true;
                }
            }
        }

        while let Some(op) = operators.pop() {
            match op {
                Op::LParen => return Err(self.error("unmatched '(' in #if expression")),
                Op::Question => return Err(self.error("expected ':' in #if expression")),
                _ => reduce(&mut values, op).map_err(|message| self.error(message))?,
            }
        }
        if !paren_guards.is_empty() {
            return Err(self.error("unmatched '(' in #if expression"));
        }
        if values.len() != 1 {
            return Err(self.error("invalid #if expression"));
        }
        values
            .pop()
            .ok_or_else(|| self.error("invalid #if expression"))
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_whitespace() {
                self.pos += 1;
                continue;
            }
            if c == '(' {
                self.pos += 1;
                tokens.push(Token::LParen);
                continue;
            }
            if c == ')' {
                self.pos += 1;
                tokens.push(Token::RParen);
                continue;
            }
            if c == '?' {
                self.pos += 1;
                tokens.push(Token::Question);
                continue;
            }
            if c == ':' {
                self.pos += 1;
                tokens.push(Token::Colon);
                continue;
            }
            if c == '\'' {
                tokens.push(Token::Number(self.parse_char_literal()?));
                continue;
            }
            if c.is_ascii_digit() {
                tokens.push(Token::Number(self.parse_number()?));
                continue;
            }
            if is_ident_start(c) {
                let start = self.pos;
                while self.pos < self.chars.len() && is_ident_continue(self.chars[self.pos]) {
                    self.pos += 1;
                }
                let ident: String = self.chars[start..self.pos].iter().collect();
                tokens.push(Token::Number(match ident.as_str() {
                    "true" => 1,
                    _ => 0,
                }));
                continue;
            }
            let rest: String = self.chars[self.pos..].iter().collect();
            let operator = ["||", "&&", "==", "!=", "<=", ">=", "<<", ">>"]
                .into_iter()
                .find(|op| rest.starts_with(op))
                .or_else(|| {
                    let single = rest.chars().next()?;
                    matches!(
                        single,
                        '|' | '&' | '^' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '!' | '~'
                    )
                    .then_some(match single {
                        '|' => "|",
                        '&' => "&",
                        '^' => "^",
                        '<' => "<",
                        '>' => ">",
                        '+' => "+",
                        '-' => "-",
                        '*' => "*",
                        '/' => "/",
                        '%' => "%",
                        '!' => "!",
                        '~' => "~",
                        _ => unreachable!(),
                    })
                });
            let Some(operator) = operator else {
                return Err(self.error("unexpected character in #if expression"));
            };
            self.pos += operator.chars().count();
            tokens.push(Token::Operator(operator));
        }
        Ok(tokens)
    }

    fn parse_number(&mut self) -> Result<i64, CompileError> {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.pos += 1;
        }
        let token: String = self.chars[start..self.pos].iter().collect();
        let mut digits = token.as_str();
        while let Some(last) = digits.chars().last() {
            if matches!(last, 'u' | 'U' | 'l' | 'L') {
                digits = &digits[..digits.len() - 1];
            } else {
                break;
            }
        }
        let (radix, body) = if let Some(rest) = digits
            .strip_prefix("0x")
            .or_else(|| digits.strip_prefix("0X"))
        {
            (16, rest)
        } else if let Some(rest) = digits
            .strip_prefix("0b")
            .or_else(|| digits.strip_prefix("0B"))
        {
            (2, rest)
        } else if digits.len() > 1 && digits.starts_with('0') {
            (8, &digits[1..])
        } else {
            (10, digits)
        };
        i64::from_str_radix(if body.is_empty() { "0" } else { body }, radix)
            .map_err(|_| self.error("invalid integer in #if expression"))
    }

    fn parse_char_literal(&mut self) -> Result<i64, CompileError> {
        self.pos += 1;
        if self.pos >= self.chars.len() {
            return Err(self.error("unterminated character literal in #if expression"));
        }
        let value = if self.chars[self.pos] == '\\' {
            self.pos += 1;
            let escaped = self.chars.get(self.pos).copied().unwrap_or('0');
            match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '0' => '\0',
                other => other,
            }
        } else {
            self.chars[self.pos]
        };
        self.pos += 1;
        if self.chars.get(self.pos) != Some(&'\'') {
            return Err(self.error("unterminated character literal in #if expression"));
        }
        self.pos += 1;
        Ok(value as i64)
    }

    fn error(&self, message: &str) -> CompileError {
        CompileError::new(message, self.line, 1)
    }
}

#[derive(Clone, Copy, Debug)]
enum Token {
    Number(i64),
    Operator(&'static str),
    LParen,
    RParen,
    Question,
    Colon,
}

fn reduce(values: &mut Vec<i64>, op: Op) -> Result<(), &'static str> {
    match op {
        Op::Unary("!") => {
            let value = values.pop().ok_or("missing operand")?;
            values.push(i64::from(value == 0));
        }
        Op::Unary("~") => {
            let value = values.pop().ok_or("missing operand")?;
            values.push(!value);
        }
        Op::Unary("-") => {
            let value = values.pop().ok_or("missing operand")?;
            values.push(value.wrapping_neg());
        }
        Op::Unary("+") => {}
        Op::Binary("||") => {
            let right = values.pop().ok_or("missing operand")?;
            let left = values.pop().ok_or("missing operand")?;
            values.push(i64::from(left != 0 || right != 0));
        }
        Op::Binary("&&") => {
            let right = values.pop().ok_or("missing operand")?;
            let left = values.pop().ok_or("missing operand")?;
            values.push(i64::from(left != 0 && right != 0));
        }
        Op::Binary("|") => binary(values, |a, b| a | b)?,
        Op::Binary("^") => binary(values, |a, b| a ^ b)?,
        Op::Binary("&") => binary(values, |a, b| a & b)?,
        Op::Binary("==") => binary(values, |a, b| i64::from(a == b))?,
        Op::Binary("!=") => binary(values, |a, b| i64::from(a != b))?,
        Op::Binary("<") => binary(values, |a, b| i64::from(a < b))?,
        Op::Binary("<=") => binary(values, |a, b| i64::from(a <= b))?,
        Op::Binary(">") => binary(values, |a, b| i64::from(a > b))?,
        Op::Binary(">=") => binary(values, |a, b| i64::from(a >= b))?,
        Op::Binary("<<") => binary(values, |a, b| a.wrapping_shl(b as u32))?,
        Op::Binary(">>") => binary(values, |a, b| a.wrapping_shr(b as u32))?,
        Op::Binary("+") => binary(values, i64::wrapping_add)?,
        Op::Binary("-") => binary(values, i64::wrapping_sub)?,
        Op::Binary("*") => binary(values, i64::wrapping_mul)?,
        Op::Binary("/") => {
            let right = values.pop().ok_or("missing operand")?;
            let left = values.pop().ok_or("missing operand")?;
            if right == 0 {
                return Err("division by zero in #if expression");
            }
            values.push(left.wrapping_div(right));
        }
        Op::Binary("%") => {
            let right = values.pop().ok_or("missing operand")?;
            let left = values.pop().ok_or("missing operand")?;
            if right == 0 {
                return Err("division by zero in #if expression");
            }
            values.push(left.wrapping_rem(right));
        }
        Op::Colon => {
            let else_value = values.pop().ok_or("missing operand")?;
            let then_value = values.pop().ok_or("missing operand")?;
            let condition = values.pop().ok_or("missing operand")?;
            values.push(if condition != 0 {
                then_value
            } else {
                else_value
            });
        }
        Op::LParen | Op::Question | Op::Binary(_) | Op::Unary(_) => {
            return Err("invalid #if expression");
        }
    }
    Ok(())
}

fn binary(values: &mut Vec<i64>, operation: fn(i64, i64) -> i64) -> Result<(), &'static str> {
    let right = values.pop().ok_or("missing operand")?;
    let left = values.pop().ok_or("missing operand")?;
    values.push(operation(left, right));
    Ok(())
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(text: &str) -> Result<i64, CompileError> {
        let depth = DepthCounter::new();
        IfExpr::new(text, &depth, 1).evaluate()
    }

    #[test]
    fn evaluates_precedence_and_conditional() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(eval("1 + 2 * 3")?, 7);
        assert_eq!(eval("(1 + 2) * 3")?, 9);
        assert_eq!(eval("0 || 2 && 3")?, 1);
        assert_eq!(eval("1 << 4")?, 16);
        assert_eq!(eval("5 > 3 ? 7 : 9")?, 7);
        assert_eq!(eval("0 ? 7 : 3 ? 8 : 9")?, 8);
        assert_eq!(eval("!0 + -1")?, 0);
        Ok(())
    }

    #[test]
    fn rejects_division_by_zero() -> Result<(), Box<dyn std::error::Error>> {
        let depth = DepthCounter::new();
        let error = match IfExpr::new("1 / 0", &depth, 1).evaluate() {
            Ok(_) => return Err("expected a division-by-zero diagnostic".into()),
            Err(error) => error,
        };
        assert!(error.message.contains("division by zero"));
        Ok(())
    }
}
