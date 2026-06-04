# Future Integration: spreadsheet-formulas

## Current State
A formula engine for the SuperInstance Spreadsheet — parsing and evaluating spreadsheet formulas in pure Rust with zero dependencies and no unsafe code. Supports cell references, ranges, built-in functions (arithmetic, statistical, SuperInstance-specific), and full operator precedence.

## Integration Opportunities

### With room physics
Formulas become the room's physics rules. Each ternary cell's tick behavior is defined by a formula: `=AVG(neighbor.value) * 0.5 + RNG() * sin(phase)` describes a cell that averages its neighbors, adds stochastic exploration, and oscillates over time. The formula engine evaluates these rules every tick, defining the room's dynamics. Different rooms have different formula sets: physics rooms use conservation laws, ecology rooms use Lotka-Volterra, music rooms use harmonic ratios.

### With ternary-cell
The formula engine provides ternary-cell's `predict` phase. A cell's formula IS its prediction function: given neighbor states, what does this cell expect to perceive? The formula engine evaluates this prediction in <1µs per cell, enabling 1M+ cells at 100Hz.

### With superinstance-spreadsheet
The formula engine is the computation backend for the ternary spreadsheet. When a user types `=EVOLVE(A1:A10, 100)` in the browser, the formula engine runs 100 generations of evolution on those cells. SuperInstance-specific functions (EVOLVE, COMPETE, MUTATE, FITNESS) extend the standard formula vocabulary.

## Dormant Ideas Now Unlockable
The zero-dependency, no-unsafe Rust design means the formula engine runs everywhere: Codespace (full compute), Jetson (edge), ESP32 (bare metal). The same formula language works at every tier — the room's physics are portable across hardware.

## Potential in Mature Systems
The formula engine becomes the universal room programming language. Instead of writing Rust code to define room behavior, you write formulas. The formula engine interprets them at the appropriate hardware tier. Room designers never touch code — they write spreadsheet formulas.

## Cross-Pollination Ideas
- **spreadsheet-cells**: Cell model + formula engine = complete room physics
- **evolution-ternary**: EVOLVE, MUTATE, FITNESS functions for evolutionary dynamics
- **conservation-matrix-rs**: CONSERVE, BALANCE functions for conservation-law checking

## Dependencies for Next Steps
- SuperInstance-specific formula functions for ternary operations
- Integration with ternary-cell's tick cycle
- WASM compilation for browser-based formula evaluation
