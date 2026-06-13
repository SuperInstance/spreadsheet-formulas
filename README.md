# Spreadsheet Formulas

**Spreadsheet Formulas** is a Rust formula engine for parsing and evaluating spreadsheet expressions — supporting arithmetic, cell references, ranges, built-in functions, and custom domain-specific functions like `=EVOLVE(A1:A10, 100)`.

## Why It Matters

Spreadsheets are the world's most widely used programming language — over a billion people use Excel or Google Sheets daily. Adding formula support to any application (dashboard, reporting tool, agent UI) requires a tokenizer, parser, AST evaluator, and cell reference resolver. This crate provides the full pipeline: `tokenizer.rs` breaks text into tokens, `parser.rs` builds an AST, `evaluator.rs` computes results, `cellref.rs` resolves references, and `builtins.rs` provides standard functions (SUM, AVG, MAX, MIN, EVOLVE).

## How It Works

### Tokenization Pipeline

```
Text → Tokenizer → Tokens → Parser → AST → Evaluator → Value
```

The tokenizer recognizes:
- Numbers: `42`, `3.14`, `1e10`
- Strings: `"hello"`
- Cell references: `A1`, `B12`, `$C$3`
- Ranges: `A1:B10`
- Operators: `+`, `-`, `*`, `/`, `^`, `=`, `>`, `<`, `>=`, `<=`, `<>`
- Functions: `SUM(`, `AVG(`, `EVOLVE(`

Tokenization: **O(N)** where N = formula length.

### Parser (Recursive Descent)

The parser uses recursive descent with precedence climbing:

```
expr   → term (('+' | '-') term)*
term   → factor (('*' | '/') factor)*
factor → number | string | cellref | range | func_call | '(' expr ')'
func   → IDENT '(' arg_list ')'
```

Operator precedence: `^` (power) > `*`/`/` > `+`/`-` > comparison operators. Parser complexity: **O(N)**.

### AST Evaluation

The `Expr` AST type:

```rust
enum Expr {
    Number(f64),
    String(String),
    CellRef(CellRef),
    Range(Range),
    BinOp(Op, Box<Expr>, Box<Expr>),
    FuncCall(String, Vec<Expr>),
}
```

Evaluation walks the AST recursively. `CellRef` lookup: **O(1)** (HashMap). Range expansion: **O(W×H)** for a W-wide, H-tall range. `FuncCall`: **O(args)** plus function-specific cost.

### Built-in Functions

| Function | Signature | Complexity |
|----------|-----------|------------|
| `SUM(range)` | Σ of numeric values | O(N) |
| `AVG(range)` | Arithmetic mean | O(N) |
| `MIN/MAX(range)` | Extremum | O(N) |
| `COUNT(range)` | Non-empty cell count | O(N) |
| `EVOLVE(range, target)` | Custom: fitness-weighted evolution | O(N) |

### Dependency Resolution

Cell references create dependency graphs. The evaluator detects and rejects circular references in **O(V + E)** topological sort before evaluation.

## Quick Start

```rust
use spreadsheet_formulas::{evaluate, tokenize, parse};

// Evaluate a simple formula
let tokens = tokenize("=SUM(A1:A3) + 5");
let ast = parse(&tokens)?;
let result = evaluate(&ast, &cell_context)?;
println!("Result: {}", result); // 42 if A1=10, A2=15, A3=12
```

## API

| Module | Key Types |
|--------|-----------|
| `tokenizer` | `Token`, `tokenize(text) -> Vec<Token>` |
| `parser` | `parse(tokens) -> Result<Expr>` |
| `ast` | `Expr` (Number, String, CellRef, Range, BinOp, FuncCall) |
| `evaluator` | `evaluate(expr, ctx) -> Result<Value>` |
| `cellref` | `CellRef` (column, row, absolute flags) |
| `range` | `Range` (start, end CellRefs) |
| `builtins` | `sum()`, `avg()`, `evolve()`, etc. |

## Architecture Notes

Spreadsheet Formulas provides the computation layer for agent dashboards in SuperInstance. In γ + η = C, formula evaluation drives γ (growth — computing derived metrics from raw sensor data) while dependency cycle detection provides η (avoidance — preventing infinite computation loops). The `EVOLVE` function integrates with the ternary evolution benchmarks from `ternary-benchmark`.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for dashboard architecture.

## References

1. Sestoft, P. (2014). *Spreadsheet Implementation Technology: Basics and Extensions*. MIT Press.
2. Erwig, M. et al. (2006). "Ensuring Spreadsheet Integrity with Formula Theory." *Software Engineering and Formal Methods*.
3. "Excel Specification and Limits." Microsoft Support Documentation, 2024.

## License

MIT
