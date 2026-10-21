//! Integer expression evaluation for `#if` and `#elif`.
//!
//! The evaluator uses an explicit shunting-yard stack so deeply nested
//! parentheses consume heap memory, not Rust stack frames. Parenthesis depth
//! is still charged against the shared compiler depth limit.

use super::Preprocessor;
use super::expr_defined::replace_defined;
use super::expr_defined::{is_ident_continue, is_ident_start};
use crate::depth::{DepthCounter, DepthGuard};
use crate::diagnostics::CompileError;

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

struct EvalState {
    values: Vec<i64>,
    operators: Vec<Op>,
    paren_guards: Vec<DepthGuard>,
    expect_operand: bool,
}

impl EvalState {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            operators: Vec::new(),
            paren_guards: Vec::new(),
            expect_operand: true,
        }
    }
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
        let mut state = EvalState::new();
        for token in tokens {
            self.apply_token(token, &mut state)?;
        }

        while let Some(op) = state.operators.pop() {
            match op {
                Op::LParen => return Err(self.error("unmatched '(' in #if expression")),
                Op::Question => return Err(self.error("expected ':' in #if expression")),
                _ => reduce(&mut state.values, op).map_err(|message| self.error(message))?,
            }
        }
        if !state.paren_guards.is_empty() {
            return Err(self.error("unmatched '(' in #if expression"));
        }
        if state.values.len() != 1 {
            return Err(self.error("invalid #if expression"));
        }
        state
            .values
            .pop()
            .ok_or_else(|| self.error("invalid #if expression"))
    }

    fn apply_token(&self, token: Token, state: &mut EvalState) -> Result<(), CompileError> {
        match token {
            Token::Number(value) => {
                state.values.push(value);
                state.expect_operand = false;
            }
            Token::LParen => {
                state.paren_guards.push(self.depth.enter(self.line, 1)?);
                state.operators.push(Op::LParen);
                state.expect_operand = true;
            }
            Token::RParen => self.close_paren(state)?,
            Token::Question => self.push_question(state)?,
            Token::Colon => self.push_colon(state)?,
            Token::Operator(text) => self.push_operator(text, state)?,
        }
        Ok(())
    }

    fn close_paren(&self, state: &mut EvalState) -> Result<(), CompileError> {
        while let Some(op) = state.operators.last().copied() {
            match op {
                Op::LParen => break,
                Op::Question => {
                    return Err(self.error("expected ':' before ')' in #if expression"));
                }
                _ => {
                    state.operators.pop();
                    reduce(&mut state.values, op).map_err(|message| self.error(message))?;
                }
            }
        }
        if state.operators.pop() != Some(Op::LParen) {
            return Err(self.error("unmatched ')' in #if expression"));
        }
        if state.paren_guards.pop().is_none() {
            return Err(self.error("unmatched ')' in #if expression"));
        }
        state.expect_operand = false;
        Ok(())
    }

    fn push_question(&self, state: &mut EvalState) -> Result<(), CompileError> {
        // The condition is complete; reduce operators that bind more tightly
        // than the conditional operator.
        while let Some(op) = state.operators.last().copied() {
            if matches!(op, Op::LParen | Op::Question | Op::Colon) {
                break;
            }
            state.operators.pop();
            reduce(&mut state.values, op).map_err(|message| self.error(message))?;
        }
        state.operators.push(Op::Question);
        state.expect_operand = true;
        Ok(())
    }

    fn push_colon(&self, state: &mut EvalState) -> Result<(), CompileError> {
        while let Some(op) = state.operators.last().copied() {
            match op {
                Op::Question | Op::LParen => break,
                _ => {
                    state.operators.pop();
                    reduce(&mut state.values, op).map_err(|message| self.error(message))?;
                }
            }
        }
        if state.operators.pop() != Some(Op::Question) {
            return Err(self.error("expected '?' before ':' in #if expression"));
        }
        state.operators.push(Op::Colon);
        state.expect_operand = true;
        Ok(())
    }

    fn push_operator(&self, text: &'static str, state: &mut EvalState) -> Result<(), CompileError> {
        let op = if matches!(text, "!" | "~") || (state.expect_operand && matches!(text, "+" | "-"))
        {
            Op::Unary(text)
        } else {
            Op::Binary(text)
        };
        if matches!(op, Op::Binary(_)) {
            while let Some(top) = state.operators.last().copied() {
                if matches!(top, Op::LParen | Op::Question | Op::Colon) {
                    break;
                }
                if top.precedence() < op.precedence() {
                    break;
                }
                state.operators.pop();
                reduce(&mut state.values, top).map_err(|message| self.error(message))?;
            }
        }
        state.operators.push(op);
        state.expect_operand = true;
        Ok(())
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
            let Some(operator) = self.scan_operator() else {
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

    /// Reads the next operator of a `#if` expression.
    fn scan_operator(&self) -> Option<&'static str> {
        let rest: String = self.chars[self.pos..].iter().collect();
        ["||", "&&", "==", "!=", "<=", ">=", "<<", ">>"]
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
            })
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
        assert_eq!(eval("(1 ? 0 : 1)")?, 0);
        assert_eq!(eval("(0 ? 2 : 3) + 4")?, 7);
        assert_eq!(eval("((1 ? 2 : 3) ? 4 : 5)")?, 4);
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
