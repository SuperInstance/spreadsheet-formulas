# spreadsheet-formulas

Formula engine for the **SuperInstance Spreadsheet** — parsing and evaluating spreadsheet formulas in pure Rust.

## Features

- **Formula parsing** with full operator precedence
- **Cell references**: `A1`, `$B$12`, `C$5`, `$D3`
- **Ranges**: `A1:A10`, `B:B`, `$A$1:$C$5`
- **Built-in functions**: arithmetic, statistical, and SuperInstance-specific
- **Zero dependencies** — pure Rust, no unsafe code

## Quick Start

```rust
use spreadsheet_formulas::{Parser, evaluate, DataContext, Value};

let expr = Parser::parse_formula("=SUM(A1:A10)").unwrap();

let mut ctx = DataContext::new();
for i in 0..10 {
    ctx.insert((i, 0), Value::Number((i + 1) as f64));
}

let result = evaluate(&expr, &ctx).unwrap();
```

## Formula Reference

### Operators

| Operator | Description | Precedence |
|----------|-------------|------------|
| `^` | Exponentiation | Highest |
| `*`, `/` | Multiplication, Division | |
| `+`, `-` | Addition, Subtraction | |
| `&` | String concatenation | |
| `=`, `!=`, `<`, `>`, `<=`, `>=` | Comparison | Lowest |
| `%` | Percent (postfix) | After unary |
| `-` | Unary negation | |

### Cell References

- `A1` — relative column, relative row
- `$A$1` — absolute column, absolute row
- `$A1` — absolute column, relative row
- `A$1` — relative column, absolute row

### Ranges

- `A1:B5` — cell range
- `B:B` — full column
- `B:D` — multi-column range

### Built-in Functions

#### Standard Functions

| Function | Description |
|----------|-------------|
| `SUM(range)` | Sum of values |
| `AVG(range)` / `AVERAGE(range)` | Arithmetic mean |
| `COUNT(range)` | Count numeric values |
| `MIN(range)` | Minimum value |
| `MAX(range)` | Maximum value |
| `ABS(n)` | Absolute value |
| `SQRT(n)` | Square root |
| `POW(base, exp)` | Power |
| `LOG(n)` / `LOG(n, base)` | Logarithm |
| `IF(cond, then, else)` | Conditional |

#### SuperInstance Functions

| Function | Description |
|----------|-------------|
| `EVOLVE(range, generations)` | Evolutionary optimization — simulates generations of improvement over the data |
| `BEST(range)` | Returns the maximum value in the range |
| `SPECIES(range)` | Counts unique species (unique values) in the data |
| `EXHAUSTIVE(range)` | Counts unique value combinations (up to 1M) |
| `ENTROPY(range)` | Calculates Shannon entropy of the data distribution |
| `PARETO(range)` | Identifies values on the Pareto front |
| `CORRELATE(range1, range2)` | Pearson correlation coefficient between two datasets |

### Examples

```
=SUM(A1:A10)
=AVG(B1:B20)
=EVOLVE(A1:A10, 100)
=BEST(B:B)
=ENTROPY(C1:C20)
=SPECIES(D1:D50)
=CORRELATE(A1:A10, B1:B10)
=PARETO(E1:E100)
=EXHAUSTIVE(F1:F10)
=IF(A1>5, "high", "low")
=2^10
=50%
```

## Architecture

- `tokenizer` — Lexes formula strings into tokens
- `parser` — Recursive descent parser producing an AST
- `ast` — Expression types (numbers, operators, function calls, etc.)
- `cellref` — Cell reference parsing (A1, $B$12)
- `range` — Range parsing (A1:A10, B:B)
- `evaluator` — Evaluates AST against a data context
- `builtins` — All built-in function implementations

## License

MIT

---

## 🚢 Fleet Integration

This repo is part of the SuperInstance spreadsheet ecosystem — a complement to the
220+ repo MIDI fleet. Every spreadsheet cell uses the same ternary {-1, 0, +1}
encoding as every fleet repo.

**Key insight:** The fleet IS the spreadsheet. Our I2I bottle protocol (message passing)
and the spreadsheet cell formula system (functional composition) are dual architectures
for the same multi-agent coordination problem.

### Direct Connections

| Spreadsheet Concept | Fleet Counterpart | What They Share |
|-------------------|-------------------|-----------------|
| Cell value | Agent state | Ternary {-1,0,+1} |
| Cell formula | I2I bottle | Communication pattern |
| Grid topology | fleet-bridge | Routing infrastructure |
| Evolutionary sort | fleet-orchestra | Agent coordination |
| MIDI cell | All MIDI repos | Note generation |

### Related Repos
- [superinstance-spreadsheet](https://github.com/SuperInstance/superinstance-spreadsheet) — Browser UI
- [fleet-ternary-music](https://github.com/SuperInstance/fleet-ternary-music) — Core math
- [fleet-orchestra](https://github.com/SuperInstance/fleet-orchestra) — Agent orchestration
- [fleet-arm-compat](https://github.com/SuperInstance/fleet-arm-compat) — ARM verification
