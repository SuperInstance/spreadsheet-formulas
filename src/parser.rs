/// Formula parser — parses tokens into an AST.
/// Recursive descent parser with operator precedence.

use crate::ast::{BinOp, Expr, UnaryOp};
use crate::tokenizer::{Token, tokenize};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let t = self.advance().ok_or("Unexpected end of input")?;
        if &t != expected {
            return Err(format!("Expected {:?}, got {:?}", expected, t));
        }
        Ok(())
    }

    /// Parse a formula string into an AST.
    pub fn parse_formula(input: &str) -> Result<Expr, String> {
        let trimmed = input.trim();
        // Strip leading =
        let formula = if trimmed.starts_with('=') { &trimmed[1..] } else { trimmed };
        let tokens = tokenize(formula)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_comparison()?;
        if parser.pos < parser.tokens.len() {
            return Err(format!("Unexpected token at position {}", parser.pos));
        }
        Ok(expr)
    }

    // Precedence levels (lowest to highest):
    // comparison: = != < > <= >=
    // concat: &
    // addition: + -
    // multiplication: * /
    // power: ^
    // unary: -
    // percent: %
    // primary: number, string, cell, range, function, parens

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Some(Token::Eq) => BinOp::Eq,
                Some(Token::Neq) => BinOp::Neq,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::Lte) => BinOp::Lte,
                Some(Token::Gte) => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        loop {
            if matches!(self.peek(), Some(Token::Amp)) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::BinaryOp {
                    op: BinOp::Concat,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let right = self.parse_unary()?; // right-associative would recurse parse_power
            left = Expr::BinaryOp {
                op: BinOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            });
        }
        self.parse_percent()
    }

    fn parse_percent(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        while matches!(self.peek(), Some(Token::Percent)) {
            self.advance();
            expr = Expr::Percent(Box::new(expr));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.advance().ok_or("Unexpected end of input")?;
        match token {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::String(s) => Ok(Expr::String(s)),
            Token::CellRef(c) => Ok(Expr::CellRef(c)),
            Token::Range(r) => Ok(Expr::Range(r)),
            Token::LParen => {
                let expr = self.parse_comparison()?;
                self.expect(&Token::RParen).map_err(|e| format!("{} (expected closing paren)", e))?;
                Ok(expr)
            }
            Token::Ident(name) => {
                // Function call
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(Token::RParen)) {
                        args.push(self.parse_comparison()?);
                        while matches!(self.peek(), Some(Token::Comma)) {
                            self.advance();
                            args.push(self.parse_comparison()?);
                        }
                    }
                    self.expect(&Token::RParen).map_err(|e| format!("{} (in function {})", e, name))?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    // Boolean literals
                    if name == "TRUE" {
                        Ok(Expr::Boolean(true))
                    } else if name == "FALSE" {
                        Ok(Expr::Boolean(false))
                    } else {
                        Err(format!("Unknown identifier: {}", name))
                    }
                }
            }
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let expr = Parser::parse_formula("42").unwrap();
        assert_eq!(expr, Expr::Number(42.0));
    }

    #[test]
    fn parse_string() {
        let expr = Parser::parse_formula("\"hello\"").unwrap();
        assert_eq!(expr, Expr::String("hello".into()));
    }

    #[test]
    fn parse_binary_op() {
        let expr = Parser::parse_formula("1+2").unwrap();
        if let Expr::BinaryOp { op, .. } = &expr {
            assert_eq!(*op, BinOp::Add);
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn parse_precedence() {
        // 1+2*3 should be 1+(2*3)
        let expr = Parser::parse_formula("1+2*3").unwrap();
        if let Expr::BinaryOp { op, left, right } = &expr {
            assert_eq!(*op, BinOp::Add);
            assert!(matches!(**left, Expr::Number(1.0)));
            assert!(matches!(**right, Expr::BinaryOp { op: BinOp::Mul, .. }));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn parse_function_call() {
        let expr = Parser::parse_formula("SUM(A1:A10)").unwrap();
        if let Expr::FunctionCall { name, args } = &expr {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected FunctionCall");
        }
    }

    #[test]
    fn parse_nested_function() {
        let expr = Parser::parse_formula("EVOLVE(A1:A10, 100)").unwrap();
        if let Expr::FunctionCall { name, args } = &expr {
            assert_eq!(name, "EVOLVE");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected FunctionCall");
        }
    }

    #[test]
    fn parse_cell_ref() {
        let expr = Parser::parse_formula("A1").unwrap();
        assert!(matches!(expr, Expr::CellRef(_)));
    }

    #[test]
    fn parse_range() {
        let expr = Parser::parse_formula("A1:B5").unwrap();
        assert!(matches!(expr, Expr::Range(_)));
    }

    #[test]
    fn parse_unary_negate() {
        let expr = Parser::parse_formula("-5").unwrap();
        assert!(matches!(expr, Expr::UnaryOp { op: UnaryOp::Negate, .. }));
    }

    #[test]
    fn parse_percent() {
        let expr = Parser::parse_formula("50%").unwrap();
        assert!(matches!(expr, Expr::Percent(_)));
    }

    #[test]
    fn parse_with_leading_equals() {
        let expr = Parser::parse_formula("=SUM(A1:A10)").unwrap();
        assert!(matches!(expr, Expr::FunctionCall { .. }));
    }

    #[test]
    fn parse_boolean() {
        assert_eq!(Parser::parse_formula("TRUE").unwrap(), Expr::Boolean(true));
        assert_eq!(Parser::parse_formula("FALSE").unwrap(), Expr::Boolean(false));
    }

    #[test]
    fn parse_comparison() {
        let expr = Parser::parse_formula("A1>5").unwrap();
        if let Expr::BinaryOp { op, .. } = &expr {
            assert_eq!(*op, BinOp::Gt);
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn parse_power() {
        let expr = Parser::parse_formula("2^3").unwrap();
        if let Expr::BinaryOp { op, .. } = &expr {
            assert_eq!(*op, BinOp::Pow);
        } else {
            panic!("Expected BinaryOp");
        }
    }
}
