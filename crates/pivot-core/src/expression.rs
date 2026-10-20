//! Initial expression grammar: decimals, ASCII names, unary +/-, binary
//! + - * /, and parentheses. An explicit spreadsheet mode adds scalar A1 references.
use pivot_types::PivotDecimal;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Decimal(PivotDecimal),
    Name(String),
    SumRange {
        first: (u32, u32),
        last: (u32, u32),
        span: std::ops::Range<usize>,
        first_reference: ReferenceSyntax,
        last_reference: ReferenceSyntax,
    },
    CellReference {
        row: u32,
        column: u32,
        absolute_row: bool,
        absolute_column: bool,
        span: std::ops::Range<usize>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}
/// Copy-axis flags and exact UTF-8 source span for one spreadsheet address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSyntax {
    pub row: u32,
    pub column: u32,
    pub absolute_row: bool,
    pub absolute_column: bool,
    pub span: std::ops::Range<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Position is a UTF-8 byte offset into the original source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: &'static str,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}
impl std::error::Error for ParseError {}

/// Parse a complete expression, bounded to 4096 bytes and 64 recursive levels.
/// The byte limit additionally bounds AST size from flat operator chains.
pub fn parse(source: &str) -> Result<Expr, ParseError> {
    parse_mode(source, false)
}

/// Parse same-sheet scalar spreadsheet syntax; spans refer to the original text.
pub fn parse_spreadsheet(source: &str) -> Result<Expr, ParseError> {
    parse_mode(source, true)
}
/// Bounded parser work, including attempts made before a syntax error.
/// Parenthesized subexpressions count conservatively as additional work.
pub fn parse_spreadsheet_counted(source: &str) -> (Result<Expr, ParseError>, usize) {
    parse_mode_counted(source, true)
}
fn parse_mode(source: &str, spreadsheet: bool) -> Result<Expr, ParseError> {
    parse_mode_counted(source, spreadsheet).0
}
fn parse_mode_counted(source: &str, spreadsheet: bool) -> (Result<Expr, ParseError>, usize) {
    if source.len() > 4096 {
        return (
            Err(ParseError {
                offset: 4096,
                message: "expression too long",
            }),
            0,
        );
    }
    let mut p = Parser {
        source,
        pos: 0,
        spreadsheet,
        work: 0,
    };
    let result = (|| {
        if spreadsheet {
            p.space();
            if p.peek() == Some(b'=') {
                p.pos += 1;
            }
        }
        let result = p.expr(0, 0)?;
        p.space();
        if p.pos != source.len() {
            return Err(p.error("unexpected trailing input"));
        }
        Ok(result)
    })();
    (result, p.work)
}
struct Parser<'a> {
    source: &'a str,
    pos: usize,
    spreadsheet: bool,
    work: usize,
}
impl Parser<'_> {
    fn error(&self, message: &'static str) -> ParseError {
        ParseError {
            offset: self.pos,
            message,
        }
    }
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }
    fn space(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }
    fn range_cell(&mut self) -> Result<ReferenceSyntax, ParseError> {
        self.space();
        let start = self.pos;
        let absolute_column = self.peek() == Some(b'$');
        if absolute_column {
            self.pos += 1;
        }
        let letters = self.pos;
        let mut column = 0u32;
        while let Some(b) = self.peek().filter(u8::is_ascii_alphabetic) {
            column = column
                .checked_mul(26)
                .and_then(|c| c.checked_add((b.to_ascii_uppercase() - b'A' + 1).into()))
                .ok_or(ParseError {
                    offset: start,
                    message: "invalid cell reference",
                })?;
            self.pos += 1;
        }
        if self.pos == letters {
            return Err(ParseError {
                offset: start,
                message: "invalid cell reference",
            });
        }
        let absolute_row = self.peek() == Some(b'$');
        if absolute_row {
            self.pos += 1;
        }
        let digits = self.pos;
        if self.peek() == Some(b'0') {
            return Err(ParseError {
                offset: start,
                message: "invalid cell reference",
            });
        }
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        let row = self.source[digits..self.pos]
            .parse::<u32>()
            .map_err(|_| ParseError {
                offset: start,
                message: "invalid cell reference",
            })?;
        if row == 0 {
            return Err(ParseError {
                offset: start,
                message: "invalid cell reference",
            });
        }
        if self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(ParseError {
                offset: start,
                message: "invalid cell reference",
            });
        }
        Ok(ReferenceSyntax {
            row,
            column,
            absolute_row,
            absolute_column,
            span: start..self.pos,
        })
    }
    fn expr(&mut self, min: u8, depth: usize) -> Result<Expr, ParseError> {
        self.work += 1;
        if depth >= 64 {
            return Err(self.error("expression nesting too deep"));
        }
        self.space();
        let mut left = match self.peek() {
            Some(b'+' | b'-') => {
                let op = if self.peek() == Some(b'+') {
                    UnaryOp::Plus
                } else {
                    UnaryOp::Minus
                };
                self.pos += 1;
                Expr::Unary {
                    op,
                    operand: Box::new(self.expr(5, depth + 1)?),
                }
            }
            Some(b'(') => {
                self.pos += 1;
                let inner = self.expr(0, depth + 1)?;
                self.space();
                if self.peek() != Some(b')') {
                    return Err(self.error("expected closing parenthesis"));
                }
                self.pos += 1;
                inner
            }
            Some(b'0'..=b'9') => {
                let start = self.pos;
                while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                    self.pos += 1;
                }
                if self.peek() == Some(b'.') {
                    self.pos += 1;
                    while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                        self.pos += 1;
                    }
                }
                let value = self.source[start..self.pos]
                    .parse()
                    .map_err(|_| ParseError {
                        offset: start,
                        message: "invalid or unrepresentable decimal",
                    })?;
                Expr::Decimal(value)
            }
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') | Some(b'$')
                if self.spreadsheet || self.peek() != Some(b'$') =>
            {
                let start = self.pos;
                while self
                    .peek()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    self.pos += 1;
                }
                if self.spreadsheet && self.source[start..self.pos].eq_ignore_ascii_case("SUM") {
                    self.space();
                    if self.peek() != Some(b'(') {
                        return Err(self.error("expected SUM rectangle"));
                    }
                    self.pos += 1;
                    let first = self.range_cell()?;
                    self.space();
                    if self.peek() != Some(b':') {
                        return Err(self.error("expected range colon"));
                    }
                    self.pos += 1;
                    let last = self.range_cell()?;
                    self.space();
                    if self.peek() != Some(b')') {
                        return Err(self.error("expected closing parenthesis"));
                    }
                    self.pos += 1;
                    Expr::SumRange {
                        first: (first.row, first.column),
                        last: (last.row, last.column),
                        span: start..self.pos,
                        first_reference: first,
                        last_reference: last,
                    }
                } else if self.spreadsheet {
                    self.pos = start;
                    let reference = self.range_cell()?;
                    Expr::CellReference {
                        row: reference.row,
                        column: reference.column,
                        absolute_row: reference.absolute_row,
                        absolute_column: reference.absolute_column,
                        span: reference.span,
                    }
                } else {
                    Expr::Name(self.source[start..self.pos].to_owned())
                }
            }
            _ => return Err(self.error("expected decimal, name, sign, or parenthesis")),
        };
        loop {
            self.space();
            let (op, power) = match self.peek() {
                Some(b'+') => (BinaryOp::Add, 1),
                Some(b'-') => (BinaryOp::Subtract, 1),
                Some(b'*') => (BinaryOp::Multiply, 3),
                Some(b'/') => (BinaryOp::Divide, 3),
                _ => break,
            };
            if power < min {
                break;
            }
            self.pos += 1;
            self.work += 1;
            let right = self.expr(power + 1, depth + 1)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence_associativity_and_names() {
        let Expr::Binary {
            op: BinaryOp::Subtract,
            left,
            right,
        } = parse("Revenue - Costs * 2").unwrap()
        else {
            panic!()
        };
        assert_eq!(*left, Expr::Name("Revenue".into()));
        assert!(matches!(
            *right,
            Expr::Binary {
                op: BinaryOp::Multiply,
                ..
            }
        ));
        let Expr::Binary {
            op: BinaryOp::Divide,
            left,
            ..
        } = parse("8 / 4 / 2").unwrap()
        else {
            panic!()
        };
        assert!(matches!(
            *left,
            Expr::Binary {
                op: BinaryOp::Divide,
                ..
            }
        ));
        let Expr::Binary {
            op: BinaryOp::Multiply,
            left,
            ..
        } = parse("(1 + 2) * 3").unwrap()
        else {
            panic!()
        };
        assert!(matches!(
            *left,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
        assert!(matches!(
            parse(" -+Net / Revenue ").unwrap(),
            Expr::Binary {
                op: BinaryOp::Divide,
                ..
            }
        ));
        assert_eq!(
            parse("1.2300").unwrap(),
            Expr::Decimal("1.2300".parse().unwrap())
        );
        assert_eq!(parse("_input2").unwrap(), Expr::Name("_input2".into()));
    }
    #[test]
    fn malformed_and_unsupported_input() {
        for s in [
            "",
            "()",
            "1 +",
            "(1",
            "1)",
            "1 2",
            "1e3",
            "1.",
            "x = 1",
            "SUM(x)",
            "Sheet!A1",
            "é",
            "1 + 💰",
            "0.00000000000000000000000000001",
        ] {
            assert!(parse(s).is_err(), "{s}");
        }
        assert_eq!(parse("1 + )").unwrap_err().offset, 4);
    }
    #[test]
    fn resource_limits() {
        assert!(parse(&"-".repeat(100)).is_err());
        assert!(parse(&format!("{}1{}", "(".repeat(100), ")".repeat(100))).is_err());
        assert!(parse(&"1".repeat(4097)).is_err());
    }
}

#[cfg(test)]
mod copy_syntax_tests {
    use super::*;
    #[test]
    fn scalar_axes_and_exact_spans() {
        for (text, row_lock, column_lock) in [
            ("A2", false, false),
            ("$A2", false, true),
            ("A$2", true, false),
            ("$A$2", true, true),
        ] {
            let source = format!("=  {text} + 1");
            let Expr::Binary { left, .. } = parse_spreadsheet(&source).unwrap() else {
                panic!()
            };
            let Expr::CellReference {
                row,
                column,
                absolute_row,
                absolute_column,
                span,
            } = *left
            else {
                panic!()
            };
            assert_eq!(
                (row, column, absolute_row, absolute_column),
                (2, 1, row_lock, column_lock)
            );
            assert_eq!(&source[span], text);
            assert!(parse(&source).is_err());
        }
        for invalid in [
            "=$A", "=A$", "=$$A1", "=A$$1", "=$A$0", "=$A$01", "=A1$", "=$1",
        ] {
            assert!(parse_spreadsheet(invalid).is_err(), "{invalid}");
        }
        assert_eq!(parse("A1").unwrap(), Expr::Name("A1".into()));
    }
    #[test]
    fn range_endpoints_keep_orientation_axes_and_spans() {
        let source = "=sum( a$3 : $B$1 )";
        let Expr::SumRange {
            first,
            last,
            first_reference,
            last_reference,
            ..
        } = parse_spreadsheet(source).unwrap()
        else {
            panic!()
        };
        assert_eq!((first, last), ((3, 1), (1, 2)));
        assert_eq!(&source[first_reference.span], "a$3");
        assert_eq!(&source[last_reference.span], "$B$1");
        assert!(first_reference.absolute_row && !first_reference.absolute_column);
        assert!(last_reference.absolute_row && last_reference.absolute_column);
    }
}
