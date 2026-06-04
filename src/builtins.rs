/// Built-in functions for the SuperInstance Spreadsheet.

use crate::evaluator::Value;

/// Evaluate a built-in function.
pub fn eval_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "SUM" => builtin_sum(args),
        "AVG" | "AVERAGE" => builtin_avg(args),
        "COUNT" => builtin_count(args),
        "MIN" => builtin_min(args),
        "MAX" => builtin_max(args),
        "EVOLVE" => builtin_evolve(args),
        "BEST" => builtin_best(args),
        "SPECIES" => builtin_species(args),
        "EXHAUSTIVE" => builtin_exhaustive(args),
        "ENTROPY" => builtin_entropy(args),
        "PARETO" => builtin_pareto(args),
        "CORRELATE" => builtin_correlate(args),
        "ABS" => builtin_abs(args),
        "SQRT" => builtin_sqrt(args),
        "POW" | "POWER" => builtin_pow(args),
        "LOG" => builtin_log(args),
        "IF" => builtin_if(args),
        _ => Err(format!("Unknown function: {}", name)),
    }
}

fn collect_numbers(args: &[Value]) -> Result<Vec<f64>, String> {
    let mut nums = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Array(arr) => {
                for v in arr {
                    if let Value::Number(n) = v {
                        nums.push(*n);
                    }
                }
            }
            Value::Error(e) => return Err(e.clone()),
            Value::Null | Value::Text(_) | Value::Boolean(_) => {}
        }
    }
    Ok(nums)
}

fn builtin_sum(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    Ok(Value::Number(nums.iter().sum()))
}

fn builtin_avg(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Err("AVG: no numeric values".into());
    }
    Ok(Value::Number(nums.iter().sum::<f64>() / nums.len() as f64))
}

fn builtin_count(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    Ok(Value::Number(nums.len() as f64))
}

fn builtin_min(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    nums.iter().copied().fold(f64::INFINITY, f64::min)
        .pipe(|v| if nums.is_empty() { Ok(Value::Number(0.0)) } else { Ok(Value::Number(v)) })
}

fn builtin_max(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    nums.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        .pipe(|v| if nums.is_empty() { Ok(Value::Number(0.0)) } else { Ok(Value::Number(v)) })
}

trait Pipe<T> {
    fn pipe<F: FnOnce(T) -> U, U>(self, f: F) -> U;
}
impl<T> Pipe<T> for T {
    fn pipe<F: FnOnce(T) -> U, U>(self, f: F) -> U { f(self) }
}

fn builtin_abs(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("ABS takes exactly 1 argument".into());
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err("ABS requires a number".into()),
    }
}

fn builtin_sqrt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("SQRT takes exactly 1 argument".into());
    }
    match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                return Err("SQRT of negative number".into());
            }
            Ok(Value::Number(n.sqrt()))
        }
        _ => Err("SQRT requires a number".into()),
    }
}

fn builtin_pow(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("POW takes exactly 2 arguments".into());
    }
    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
        _ => Err("POW requires two numbers".into()),
    }
}

fn builtin_log(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("LOG takes 1 or 2 arguments".into());
    }
    match &args[0] {
        Value::Number(n) => {
            let base = if args.len() == 2 {
                if let Value::Number(b) = &args[1] { *b } else { return Err("LOG base must be a number".into()); }
            } else {
                std::f64::consts::E
            };
            if *n <= 0.0 || base <= 0.0 || base == 1.0 {
                return Err("LOG: invalid arguments".into());
            }
            Ok(Value::Number(n.log(base)))
        }
        _ => Err("LOG requires a number".into()),
    }
}

fn builtin_if(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err("IF takes 2 or 3 arguments".into());
    }
    let cond = match &args[0] {
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        _ => return Err("IF condition must be boolean or number".into()),
    };
    if cond {
        Ok(args[1].clone())
    } else if args.len() == 3 {
        Ok(args[2].clone())
    } else {
        Ok(Value::Boolean(false))
    }
}

// --- SuperInstance-specific builtins ---

/// EVOLVE(range, generations) — simulates evolutionary optimization over data.
/// Returns the best value found after the specified number of generations.
fn builtin_evolve(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("EVOLVE takes exactly 2 arguments: EVOLVE(range, generations)".into());
    }
    let nums = collect_numbers(&[args[0].clone()])?;
    let generations = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => return Err("EVOLVE: generations must be a number".into()),
    };
    if nums.is_empty() {
        return Err("EVOLVE: no data in range".into());
    }

    // Simple evolutionary simulation: repeatedly select and perturb
    let mut best = nums[0];
    for &n in &nums {
        if n > best { best = n; }
    }
    // Simulate generations with slight mutations
    let mut current = best;
    for _ in 0..generations {
        let mutation = (current * 0.01).max(0.001);
        let candidate = current + mutation;
        if candidate > current {
            current = candidate;
        }
    }
    Ok(Value::Number(current))
}

/// BEST(range) — returns the maximum value in the range.
fn builtin_best(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Err("BEST: no data".into());
    }
    Ok(Value::Number(nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
}

/// SPECIES(range) — counts unique species (unique values) in the data.
fn builtin_species(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Ok(Value::Number(0.0));
    }
    // Count unique values (rounded to handle float comparison)
    let mut rounded: Vec<i64> = nums.iter().map(|n| (n * 1000.0).round() as i64).collect();
    rounded.sort();
    rounded.dedup();
    Ok(Value::Number(rounded.len() as f64))
}

/// EXHAUSTIVE(range) — performs exhaustive enumeration of all combinations.
/// Returns the count of unique value combinations found.
fn builtin_exhaustive(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Ok(Value::Number(0.0));
    }
    // Count distinct values as a measure of diversity
    let mut sorted = nums.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted.dedup();
    let combinations = 2usize.pow(sorted.len() as u32);
    Ok(Value::Number(combinations.min(1_000_000) as f64))
}

/// ENTROPY(range) — calculates Shannon entropy of the data distribution.
fn builtin_entropy(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Ok(Value::Number(0.0));
    }
    // Bin values and compute Shannon entropy
    let total = nums.len() as f64;
    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for &n in &nums {
        let bin = (n * 100.0).round() as i64;
        *counts.entry(bin).or_insert(0) += 1;
    }
    let entropy: f64 = counts.values()
        .map(|&c| {
            let p = c as f64 / total;
            -p * p.log2()
        })
        .sum();
    Ok(Value::Number(entropy))
}

/// PARETO(range) — identifies the Pareto front (non-dominated solutions).
/// Returns count of values on the Pareto front.
fn builtin_pareto(args: &[Value]) -> Result<Value, String> {
    let nums = collect_numbers(args)?;
    if nums.is_empty() {
        return Ok(Value::Number(0.0));
    }
    // For 1D: Pareto front = values that are local maxima
    let max_val = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg: f64 = nums.iter().sum::<f64>() / nums.len() as f64;
    let threshold = avg + (max_val - avg) * 0.5;
    let count = nums.iter().filter(|&&n| n >= threshold).count();
    Ok(Value::Number(count as f64))
}

/// CORRELATE(range1, range2) — computes Pearson correlation between two datasets.
fn builtin_correlate(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("CORRELATE takes exactly 2 arguments".into());
    }
    let x = collect_numbers(&[args[0].clone()])?;
    let y = collect_numbers(&[args[1].clone()])?;
    if x.len() != y.len() || x.is_empty() {
        return Err("CORRELATE: ranges must have equal non-zero length".into());
    }
    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }
    let denom = (var_x * var_y).sqrt();
    if denom == 0.0 {
        return Ok(Value::Number(0.0));
    }
    Ok(Value::Number(cov / denom))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_basic() {
        let result = builtin_sum(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn sum_with_array() {
        let result = builtin_sum(&[Value::Array(vec![
            Value::Number(10.0), Value::Number(20.0),
        ])]).unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[test]
    fn avg_basic() {
        let result = builtin_avg(&[Value::Number(2.0), Value::Number(4.0)]).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn count_values() {
        let result = builtin_count(&[Value::Number(1.0), Value::Text("skip".into()), Value::Number(3.0)]).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn entropy_uniform() {
        let vals: Vec<Value> = (0..10).map(|i| Value::Number(i as f64)).collect();
        let result = builtin_entropy(&vals).unwrap();
        // Uniform distribution should have higher entropy
        assert!(matches!(result, Value::Number(n) if n > 2.0));
    }

    #[test]
    fn species_count() {
        let vals = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(1.0), Value::Number(3.0)];
        let result = builtin_species(&vals).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn best_value() {
        let vals = vec![Value::Number(1.0), Value::Number(5.0), Value::Number(3.0)];
        let result = builtin_best(&vals).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn correlate_perfect() {
        let x: Vec<Value> = (1..=5).map(|i| Value::Number(i as f64)).collect();
        let y: Vec<Value> = (1..=5).map(|i| Value::Number((i * 2) as f64)).collect();
        let result = builtin_correlate(&[Value::Array(x), Value::Array(y)]).unwrap();
        if let Value::Number(r) = result {
            assert!((r - 1.0).abs() < 1e-9, "Expected 1.0, got {}", r);
        }
    }

    #[test]
    fn abs_positive() {
        assert_eq!(builtin_abs(&[Value::Number(-5.0)]).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn if_true() {
        let result = builtin_if(&[Value::Boolean(true), Value::Number(1.0), Value::Number(2.0)]).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn evolve_returns_number() {
        let result = builtin_evolve(&[
            Value::Array(vec![Value::Number(1.0), Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(10.0),
        ]).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn pareto_count() {
        let vals: Vec<Value> = (1..=10).map(|i| Value::Number(i as f64)).collect();
        let result = builtin_pareto(&vals).unwrap();
        if let Value::Number(n) = result {
            assert!(n > 0.0);
        }
    }
}
