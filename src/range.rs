/// Range parsing and representation.
/// Supports A1:A10, B:B, $A$1:$C$5, etc.

use crate::cellref::CellRef;

/// A cell range, either two explicit cell references or a full column range.
#[derive(Debug, Clone, PartialEq)]
pub enum Range {
    /// Two explicit cell references: A1:B5
    CellRange { start: CellRef, end: CellRef },
    /// Full column range: B:B or B:D
    ColRange { start_col: usize, end_col: usize },
}

impl Range {
    /// Parse a range string.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let left = parts[0].trim();
        let right = parts[1].trim();

        // Try full column range: both sides are just letters (no digits)
        let _left_letters: String = left.chars().filter(|c| c.is_ascii_alphabetic()).collect();
        let _left_digits: String = left.chars().filter(|c| c.is_ascii_digit()).collect();
        let _right_letters: String = right.chars().filter(|c| c.is_ascii_alphabetic()).collect();
        let _right_digits: String = right.chars().filter(|c| c.is_ascii_digit()).collect();

        // Strip $ for column-only check
        let left_clean: String = left.chars().filter(|c| *c != '$').collect();
        let right_clean: String = right.chars().filter(|c| *c != '$').collect();

        // Column range: B:B or B:D (no digits after stripping $)
        let left_has_no_digits = left_clean.chars().all(|c| c.is_ascii_alphabetic());
        let right_has_no_digits = right_clean.chars().all(|c| c.is_ascii_alphabetic());

        if left_has_no_digits && right_has_no_digits {
            let start_col = crate::cellref::col_letters_to_index(&left_clean)?;
            let end_col = crate::cellref::col_letters_to_index(&right_clean)?;
            return Some(Range::ColRange { start_col, end_col });
        }

        // Cell range: A1:B5
        let start = CellRef::parse(left)?;
        let end = CellRef::parse(right)?;
        Some(Range::CellRange { start, end })
    }

    /// Iterate over all (row, col) pairs in this range.
    pub fn iter_cells(&self) -> Box<dyn Iterator<Item = (usize, usize)> + '_> {
        match self {
            Range::CellRange { start, end } => {
                let min_row = start.row.min(end.row);
                let max_row = start.row.max(end.row);
                let min_col = start.col.min(end.col);
                let max_col = start.col.max(end.col);
                Box::new(
                    (min_row..=max_row)
                        .flat_map(move |r| (min_col..=max_col).map(move |c| (r, c))),
                )
            }
            Range::ColRange { start_col, end_col } => {
                // Column ranges are unbounded; iterate col pairs only
                let min = *start_col.min(end_col);
                let max = *start_col.max(end_col);
                Box::new((min..=max).map(|c| (0usize, c)))
            }
        }
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::CellRange { start, end } => write!(f, "{}:{}", start, end),
            Range::ColRange { start_col, end_col } => write!(
                f,
                "{}:{}",
                crate::cellref::index_to_col_letters(*start_col),
                crate::cellref::index_to_col_letters(*end_col)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cell_range() {
        let r = Range::parse("A1:B5").unwrap();
        assert!(matches!(r, Range::CellRange { .. }));
    }

    #[test]
    fn parse_col_range() {
        let r = Range::parse("B:B").unwrap();
        assert!(matches!(r, Range::ColRange { start_col: 1, end_col: 1 }));
    }

    #[test]
    fn parse_multi_col_range() {
        let r = Range::parse("B:D").unwrap();
        assert!(matches!(r, Range::ColRange { start_col: 1, end_col: 3 }));
    }

    #[test]
    fn parse_absolute_range() {
        let r = Range::parse("$A$1:$C$5").unwrap();
        if let Range::CellRange { start, end } = r {
            assert_eq!(start.col, 0);
            assert_eq!(start.row, 0);
            assert!(start.abs_col && start.abs_row);
            assert_eq!(end.col, 2);
            assert_eq!(end.row, 4);
        } else {
            panic!("Expected CellRange");
        }
    }

    #[test]
    fn iter_cells_range() {
        let r = Range::parse("A1:B2").unwrap();
        let cells: Vec<_> = r.iter_cells().collect();
        assert_eq!(cells, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
    }

    #[test]
    fn parse_invalid_no_colon() {
        assert!(Range::parse("A1").is_none());
    }
}
