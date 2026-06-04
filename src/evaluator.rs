/// Evaluator — evaluates parsed formulas against a data context.

use crate::ast::{BinOp, Expr, UnaryOp};
use crate::builtins::eval_builtin;
use std::collections::HashMap;

/// A cell value in the spreadsheet.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
    Error(String),
}

/// Data context: maps (row, col) to cell values.
pub type DataContext = HashMap<(usize, usize), Value>;

/// Evaluate an expression against a data context.
pub fn evaluate(expr: &Expr, ctx: &DataContext) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::String(s) => Ok(Value::Text(s.clone())),
        Expr::Boolean(b) => Ok(Value::Boolean(*b)),
        Expr::CellRef(cell) => {
            let key = (cell.row, cell.col);
            Ok(ctx.get(&key).cloned().unwrap_or(Value::Number(0.0)))
        }
        Expr::Range(range) => {
            let values: Vec<Value> = range
                .iter_cells()
                .filter_map(|(r, c)| ctx.get(&(r, c)).cloned())
                .collect();
            Ok(Value::Array(values))
        }
        Expr::BinaryOp { op, left, right } => {
            let lval = evaluate(left, ctx)?;
            let rval = evaluate(right, ctx)?;
            eval_binary(*op, lval, rval)
        }
        Expr::UnaryOp { op: UnaryOp::Negate, operand } => {
            let val = evaluate(operand, ctx)?;
            match val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err("Cannot negate non-number".into()),
            }
        }
        Expr::Percent(expr) => {
            let val = evaluate(expr, ctx)?;
            match val {
                Value::Number(n) => Ok(Value::Number(n / 100.0)),
                _ => Err("Cannot apply % to non-number".into()),
            }
        }
        Expr::FunctionCall { name, args } => {
            let evaluated: Result<Vec<Value>, String> = args
                .iter()
                .map(|a| evaluate(a, ctx))
                .collect();
            eval_builtin(name, &evaluated?)
        }
    }
}

fn to_number(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        Value::Text(s) => s.parse::<f64>().map_err(|_| format!("Cannot convert '{}' to number", s)),
        Value::Error(e) => Err(e.clone()),
        Value::Array(_) => Err("Cannot convert array to number".into()),
    }
}

fn eval_binary(op: BinOp, left: Value, right: Value) -> Result<Value, String> {
    match op {
        BinOp::Add => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Number(l + r))
        }
        BinOp::Sub => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Number(l - r))
        }
        BinOp::Mul => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Number(l * r))
        }
        BinOp::Div => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            if r == 0.0 {
                return Err("Division by zero".into());
            }
            Ok(Value::Number(l / r))
        }
        BinOp::Pow => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Number(l.powf(r)))
        }
        BinOp::Concat => {
            let ls = value_to_string(&left);
            let rs = value_to_string(&right);
            Ok(Value::Text(format!("{}{}", ls, rs)))
        }
        BinOp::Eq => Ok(Value::Boolean(values_eq(&left, &right))),
        BinOp::Neq => Ok(Value::Boolean(!values_eq(&left, &right))),
        BinOp::Lt => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Boolean(l < r))
        }
        BinOp::Gt => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Boolean(l > r))
        }
        BinOp::Lte => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Boolean(l <= r))
        }
        BinOp::Gte => {
            let l = to_number(&left)?;
            let r = to_number(&right)?;
            Ok(Value::Boolean(l >= r))
        }
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::Text(x), Value::Text(y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Number(n) => format!("{}", n),
        Value::Text(s) => s.clone(),
        Value::Boolean(b) => if *b { "TRUE".into() } else { "FALSE".into() },
        Value::Null => String::new(),
        Value::Error(e) => format!("#ERR: {}", e),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval_str(formula: &str, ctx: &DataContext) -> Result<Value, String> {
        let expr = Parser::parse_formula(formula)?;
        evaluate(&expr, ctx)
    }

    #[test]
    fn eval_arithmetic() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("1+2*3", &ctx).unwrap(), Value::Number(7.0));
    }

    #[test]
    fn eval_cell_ref() {
        let mut ctx = DataContext::new();
        ctx.insert((0, 0), Value::Number(42.0)); // A1
        assert_eq!(eval_str("A1", &ctx).unwrap(), Value::Number(42.0));
    }

    #[test]
    fn eval_range_sum() {
        let mut ctx = DataContext::new();
        for i in 0..5 {
            ctx.insert((i, 0), Value::Number((i + 1) as f64)); // A1:A5
        }
        assert_eq!(eval_str("SUM(A1:A5)", &ctx).unwrap(), Value::Number(15.0));
    }

    #[test]
    fn eval_division_by_zero() {
        let ctx = DataContext::new();
        assert!(eval_str("1/0", &ctx).is_err());
    }

    #[test]
    fn eval_concat() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("\"hello\"&\" world\"", &ctx).unwrap(), Value::Text("hello world".into()));
    }

    #[test]
    fn eval_negate() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("-(3+2)", &ctx).unwrap(), Value::Number(-5.0));
    }

    #[test]
    fn eval_comparison() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("5>3", &ctx).unwrap(), Value::Boolean(true));
        assert_eq!(eval_str("2=2", &ctx).unwrap(), Value::Boolean(true));
    }

    #[test]
    fn eval_percent() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("50%", &ctx).unwrap(), Value::Number(0.5));
    }

    #[test]
    fn eval_evolve() {
        let mut ctx = DataContext::new();
        for i in 0..10 {
            ctx.insert((i, 0), Value::Number(i as f64));
        }
        let result = eval_str("EVOLVE(A1:A10, 100)", &ctx).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn eval_entropy() {
        let mut ctx = DataContext::new();
        for i in 0..20 {
            ctx.insert((i, 2), Value::Number((i % 4) as f64));
        }
        let result = eval_str("ENTROPY(C1:C20)", &ctx).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn eval_power() {
        let ctx = DataContext::new();
        assert_eq!(eval_str("2^10", &ctx).unwrap(), Value::Number(1024.0));
    }
}
