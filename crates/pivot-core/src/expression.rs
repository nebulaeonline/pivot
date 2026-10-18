//! Initial expression grammar: decimals, ASCII names, unary +/-, binary
//! + - * /, and parentheses. No assignments, calls, or cell addressing.
use pivot_types::PivotDecimal;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Decimal(PivotDecimal),
    Name(String),
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
    if source.len() > 4096 {
        return Err(ParseError {
            offset: 4096,
            message: "expression too long",
        });
    }
    let mut p = Parser { source, pos: 0 };
    let result = p.expr(0, 0)?;
    p.space();
    if p.pos != source.len() {
        return Err(p.error("unexpected trailing input"));
    }
    Ok(result)
}
struct Parser<'a> {
    source: &'a str,
    pos: usize,
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
    fn expr(&mut self, min: u8, depth: usize) -> Result<Expr, ParseError> {
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
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                let start = self.pos;
                while self
                    .peek()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    self.pos += 1;
                }
                Expr::Name(self.source[start..self.pos].to_owned())
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
