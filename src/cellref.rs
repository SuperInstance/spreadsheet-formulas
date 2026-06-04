/// Cell reference parsing and representation.
/// Supports formats like A1, B12, $A$1, $A1, A$1.

use core::fmt;

/// A cell reference, optionally with absolute column/row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub col: usize,      // 0-indexed column (A=0, B=1, ...)
    pub row: usize,      // 0-indexed row
    pub abs_col: bool,   // $A style
    pub abs_row: bool,   // $1 style
}

impl CellRef {
    /// Parse a cell reference string like "A1", "$B$12", "C$5", "$D3".
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let mut chars = input.chars().peekable();
        let mut abs_col = false;
        let mut abs_row = false;

        // Optional $ for absolute column
        if chars.peek() == Some(&'$') {
            abs_col = true;
            chars.next();
        }

        // Column letter(s)
        let mut col_str = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() {
                col_str.push(chars.next().unwrap().to_ascii_uppercase());
            } else {
                break;
            }
        }
        if col_str.is_empty() {
            return None;
        }

        // Optional $ for absolute row
        if chars.peek() == Some(&'$') {
            abs_row = true;
            chars.next();
        }

        // Row number
        let mut row_str = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                row_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        if row_str.is_empty() {
            return None;
        }

        // Must consume entire input
        if chars.next().is_some() {
            return None;
        }

        let col = col_letters_to_index(&col_str)?;
        let row: usize = row_str.parse().ok()?;
        if row == 0 {
            return None;
        }

        Some(CellRef {
            col,
            row: row - 1, // 0-indexed
            abs_col,
            abs_row,
        })
    }

    /// Convert to A1-style string.
    pub fn to_a1(&self) -> String {
        format!(
            "{}{}{}{}",
            if self.abs_col { "$" } else { "" },
            index_to_col_letters(self.col),
            if self.abs_row { "$" } else { "" },
            self.row + 1,
        )
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a1())
    }
}

/// Convert column letters (A=0, B=1, ..., Z=25, AA=26, ...) to index.
pub fn col_letters_to_index(letters: &str) -> Option<usize> {
    let mut index: usize = 0;
    for c in letters.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        index = index * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    Some(index - 1)
}

/// Convert 0-indexed column to letters.
pub fn index_to_col_letters(mut index: usize) -> String {
    let mut result = String::new();
    loop {
        result.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let c = CellRef::parse("A1").unwrap();
        assert_eq!(c.col, 0);
        assert_eq!(c.row, 0);
        assert!(!c.abs_col);
        assert!(!c.abs_row);
    }

    #[test]
    fn parse_absolute_both() {
        let c = CellRef::parse("$B$12").unwrap();
        assert_eq!(c.col, 1);
        assert_eq!(c.row, 11);
        assert!(c.abs_col);
        assert!(c.abs_row);
    }

    #[test]
    fn parse_absolute_col() {
        let c = CellRef::parse("$C5").unwrap();
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 4);
        assert!(c.abs_col);
        assert!(!c.abs_row);
    }

    #[test]
    fn parse_absolute_row() {
        let c = CellRef::parse("D$7").unwrap();
        assert_eq!(c.col, 3);
        assert_eq!(c.row, 6);
        assert!(!c.abs_col);
        assert!(c.abs_row);
    }

    #[test]
    fn parse_multi_letter_col() {
        let c = CellRef::parse("AA1").unwrap();
        assert_eq!(c.col, 26);
        assert_eq!(c.row, 0);
    }

    #[test]
    fn parse_invalid_no_row() {
        assert!(CellRef::parse("A").is_none());
    }

    #[test]
    fn parse_invalid_empty() {
        assert!(CellRef::parse("").is_none());
    }

    #[test]
    fn roundtrip() {
        for input in &["A1", "$B$12", "C$5", "$D3", "AA100"] {
            assert_eq!(CellRef::parse(input).unwrap().to_a1(), *input);
        }
    }

    #[test]
    fn col_index_roundtrip() {
        for i in 0..702 {
            assert_eq!(col_letters_to_index(&index_to_col_letters(i)), Some(i));
        }
    }
}
