/// Tokenizer — lexes formula strings into tokens.

use crate::cellref::CellRef;
use crate::range::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Ident(String),
    CellRef(CellRef),
    Range(Range),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Amp, // &
}

/// Tokenize a formula string (without the leading =).
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // String literal
        if c == '"' {
            let mut s = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string literal".into());
            }
            i += 1; // skip closing "
            tokens.push(Token::String(s));
            continue;
        }

        // Number
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let mut num_str = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_str.push(chars[i]);
                i += 1;
            }
            let val: f64 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
            tokens.push(Token::Number(val));
            continue;
        }

        // Comparison operators (multi-char)
        if c == '<' || c == '>' || c == '!' || c == '=' {
            if c == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::Lte);
                i += 2;
                continue;
            }
            if c == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::Gte);
                i += 2;
                continue;
            }
            if c == '<' {
                tokens.push(Token::Lt);
                i += 1;
                continue;
            }
            if c == '>' {
                tokens.push(Token::Gt);
                i += 1;
                continue;
            }
            if c == '=' {
                tokens.push(Token::Eq);
                i += 1;
                continue;
            }
            if c == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::Neq);
                i += 2;
                continue;
            }
        }

        // Single char operators
        match c {
            '(' => { tokens.push(Token::LParen); i += 1; continue; }
            ')' => { tokens.push(Token::RParen); i += 1; continue; }
            ',' => { tokens.push(Token::Comma); i += 1; continue; }
            '+' => { tokens.push(Token::Plus); i += 1; continue; }
            '-' => { tokens.push(Token::Minus); i += 1; continue; }
            '*' => { tokens.push(Token::Star); i += 1; continue; }
            '/' => { tokens.push(Token::Slash); i += 1; continue; }
            '^' => { tokens.push(Token::Caret); i += 1; continue; }
            '%' => { tokens.push(Token::Percent); i += 1; continue; }
            '&' => { tokens.push(Token::Amp); i += 1; continue; }
            _ => {}
        }

        // Identifiers (may include $), cell refs, or ranges
        if c.is_ascii_alphabetic() || c == '$' || c == '_' {
            let mut ident = String::new();
            let _start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '$' || chars[i] == '_') {
                ident.push(chars[i]);
                i += 1;
            }

            // Try to parse as range (contains :)
            if i < chars.len() && chars[i] == ':' {
                let mut range_str = ident.clone();
                range_str.push(':');
                i += 1;
                let right_start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '$') {
                    range_str.push(chars[i]);
                    i += 1;
                }
                if let Some(r) = Range::parse(&range_str) {
                    tokens.push(Token::Range(r));
                    continue;
                }
                // Fallback: didn't parse as range
                let _ = right_start;
                return Err(format!("Failed to parse range: {}", range_str));
            }

            // Try cell ref
            if let Some(cell) = CellRef::parse(&ident) {
                tokens.push(Token::CellRef(cell));
                continue;
            }

            // It's an identifier
            tokens.push(Token::Ident(ident.to_uppercase()));
            continue;
        }

        return Err(format!("Unexpected character: '{}'", c));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_number() {
        let tokens = tokenize("42.5").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.5));
    }

    #[test]
    fn tokenize_string() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens[0], Token::String("hello world".into()));
    }

    #[test]
    fn tokenize_function_call() {
        let tokens = tokenize("SUM(A1:A10)").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Ident("SUM".into()));
        assert_eq!(tokens[1], Token::LParen);
        assert!(matches!(&tokens[2], Token::Range(_)));
        assert_eq!(tokens[3], Token::RParen);
    }

    #[test]
    fn tokenize_operators() {
        let tokens = tokenize("1+2*3").unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Number(1.0));
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::Number(2.0));
        assert_eq!(tokens[3], Token::Star);
        assert_eq!(tokens[4], Token::Number(3.0));
    }

    #[test]
    fn tokenize_comparison() {
        let tokens = tokenize("A1<=B1").unwrap();
        assert!(matches!(&tokens[1], Token::Lte));
    }

    #[test]
    fn tokenize_cell_ref() {
        let tokens = tokenize("$A$1").unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::CellRef(c) = &tokens[0] {
            assert_eq!(c.col, 0);
            assert_eq!(c.row, 0);
        } else {
            panic!("Expected CellRef");
        }
    }
}
