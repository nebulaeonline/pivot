//! Initial expression grammar: decimals, ASCII names, unary +/-, binary
//! + - * /, and parentheses. An explicit spreadsheet mode adds scalar A1 references.
use pivot_types::PivotDecimal;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Typed {
        op: crate::typed::Op,
        arguments: Vec<Expr>,
    },
    AggregateCall {
        kind: AggregateKind,
        arguments: Vec<AggregateArgument>,
    },
    #[doc(hidden)]
    CapturedOutput {
        producer: String,
        output: String,
        span: std::ops::Range<usize>,
    },
    Decimal(PivotDecimal),
    Name(String),
    SumRange {
        kind: AggregateKind,
        first: (u32, u32),
        last: (u32, u32),
        span: std::ops::Range<usize>,
        first_reference: ReferenceSyntax,
        last_reference: ReferenceSyntax,
    },
    CellReference {
        qualifier: Option<SheetSyntax>,
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
/// Distinguish bare rectangles from nested single-rectangle function results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateArgument {
    Range(Expr),
    Scalar(Expr),
}
impl AggregateArgument {
    pub fn expression(&self) -> &Expr {
        match self {
            Self::Range(e) | Self::Scalar(e) => e,
        }
    }
}
/// Closed range reducers; SUM retains its established strict policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggregateKind {
    Sum,
    Min,
    Max,
    Average,
    Count,
}
/// Postfix call boundaries. Range consumes the preceding selection operand;
/// Value consumes a scalar expression. No names or worksheet coordinates here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateControl {
    Start(AggregateKind),
    Range,
    Value,
    End,
}
impl AggregateControl {
    pub fn code(self) -> u8 {
        match self {
            Self::Start(k) => k as u8,
            Self::Range => 5,
            Self::Value => 6,
            Self::End => 7,
        }
    }
    pub fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0 => Self::Start(AggregateKind::Sum),
            1 => Self::Start(AggregateKind::Min),
            2 => Self::Start(AggregateKind::Max),
            3 => Self::Start(AggregateKind::Average),
            4 => Self::Start(AggregateKind::Count),
            5 => Self::Range,
            6 => Self::Value,
            7 => Self::End,
            _ => return None,
        })
    }
}
impl AggregateKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "SUM" => Some(Self::Sum),
            "MIN" => Some(Self::Min),
            "MAX" => Some(Self::Max),
            "AVERAGE" => Some(Self::Average),
            "COUNT" => Some(Self::Count),
            _ => None,
        }
    }
}
/// Copy-axis flags and exact UTF-8 source span for one spreadsheet address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSyntax {
    pub qualifier: Option<SheetSyntax>,
    pub row: u32,
    pub column: u32,
    pub absolute_row: bool,
    pub absolute_column: bool,
    pub span: std::ops::Range<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetSyntax {
    pub name: String,
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
    parse_flags_counted(source, spreadsheet, false)
}
#[doc(hidden)]
pub fn parse_structured_proof(source: &str) -> Result<Expr, ParseError> {
    parse_flags_counted(source, true, true).0
}
fn parse_flags_counted(
    source: &str,
    spreadsheet: bool,
    structured: bool,
) -> (Result<Expr, ParseError>, usize) {
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
        structured,
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
    structured: bool,
    source: &'a str,
    pos: usize,
    spreadsheet: bool,
    work: usize,
}
impl Parser<'_> {
    fn proof_label(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        if !self
            .peek()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        {
            return Err(self.error("invalid proof label"));
        }
        self.pos += 1;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.pos += 1;
        }
        if self.pos - start > 63 {
            return Err(self.error("proof label too long"));
        }
        Ok(self.source[start..self.pos].into())
    }
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
        let qualifier = self.sheet_qualifier()?;
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
            qualifier,
            row,
            column,
            absolute_row,
            absolute_column,
            span: start..self.pos,
        })
    }
    fn sheet_qualifier(&mut self) -> Result<Option<SheetSyntax>, ParseError> {
        let start = self.pos;
        let name = if self.peek() == Some(b'\'') {
            self.pos += 1;
            let mut value = String::new();
            loop {
                match self.source[self.pos..].chars().next() {
                    None => return Err(self.error("unterminated worksheet qualifier")),
                    Some('\'') => {
                        self.pos += 1;
                        if self.peek() == Some(b'\'') {
                            value.push('\'');
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    Some(c) => {
                        value.push(c);
                        self.pos += c.len_utf8();
                    }
                }
            }
            value
        } else {
            if !self
                .peek()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
            {
                return Ok(None);
            }
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                self.pos += 1;
            }
            self.source[start..self.pos].to_owned()
        };
        let end = self.pos;
        self.space();
        if self.peek() != Some(b'!') {
            self.pos = start;
            return Ok(None);
        }
        self.pos += 1;
        Ok(Some(SheetSyntax {
            name,
            span: start..end,
        }))
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
                    operand: Box::new(self.expr(9, depth + 1)?),
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
            Some(b'@') if self.structured => {
                let start = self.pos;
                self.pos += 1;
                if self.peek() != Some(b'{') {
                    return Err(self.error("expected structured opening brace"));
                }
                self.pos += 1;
                let producer = self.proof_label()?;
                if self.peek() != Some(b'.') {
                    return Err(self.error("expected structured output separator"));
                }
                self.pos += 1;
                let output = self.proof_label()?;
                if self.peek() != Some(b'}') {
                    return Err(self.error("expected structured closing brace"));
                }
                self.pos += 1;
                Expr::CapturedOutput {
                    producer,
                    output,
                    span: start..self.pos,
                }
            }
            Some(b'#')
                if self.spreadsheet
                    && self
                        .source
                        .as_bytes()
                        .get(self.pos..self.pos + 5)
                        .is_some_and(|s| s.eq_ignore_ascii_case(b"#REF!")) =>
            {
                self.pos += 5;
                Expr::Typed {
                    op: crate::typed::Op::ReferenceDeleted(String::new()),
                    arguments: Vec::new(),
                }
            }
            Some(b'"') if self.spreadsheet => {
                self.pos += 1;
                let mut value = String::new();
                loop {
                    match self.source[self.pos..].chars().next() {
                        None => return Err(self.error("unterminated string")),
                        Some('"') => {
                            self.pos += 1;
                            if self.peek() == Some(b'"') {
                                value.push('"');
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                        Some('\r' | '\n') => return Err(self.error("multiline formula string")),
                        Some(c) => {
                            value.push(c);
                            self.pos += c.len_utf8();
                        }
                    }
                }
                Expr::Typed {
                    op: crate::typed::Op::String(value),
                    arguments: Vec::new(),
                }
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
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') | Some(b'$' | b'\'')
                if self.spreadsheet || !matches!(self.peek(), Some(b'$' | b'\'')) =>
            {
                let start = self.pos;
                while self
                    .peek()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    self.pos += 1;
                }
                let name = &self.source[start..self.pos];
                let qualified = self.source[self.pos..]
                    .trim_start_matches(|c: char| c.is_ascii_whitespace())
                    .starts_with('!');
                if self.spreadsheet
                    && !qualified
                    && (name.eq_ignore_ascii_case("TRUE") || name.eq_ignore_ascii_case("FALSE"))
                {
                    Expr::Typed {
                        op: crate::typed::Op::Boolean(name.eq_ignore_ascii_case("TRUE")),
                        arguments: Vec::new(),
                    }
                } else if let Some(op) = self
                    .spreadsheet
                    .then(|| crate::typed::Op::named(name))
                    .flatten()
                    .filter(|_| !qualified)
                {
                    self.space();
                    if self.peek() != Some(b'(') {
                        return Err(self.error("expected function arguments"));
                    }
                    self.pos += 1;
                    let mut arguments = Vec::new();
                    loop {
                        arguments.push(self.expr(0, depth + 1)?);
                        self.space();
                        match self.peek() {
                            Some(b',') => self.pos += 1,
                            Some(b')') => {
                                self.pos += 1;
                                break;
                            }
                            _ => return Err(self.error("expected comma or closing parenthesis")),
                        }
                    }
                    if !op.valid_arity(arguments.len()) {
                        return Err(self.error("invalid function arity"));
                    }
                    Expr::Typed { op, arguments }
                } else if let Some(kind) = self
                    .spreadsheet
                    .then_some(())
                    .filter(|_| {
                        !self.source[self.pos..]
                            .trim_start_matches(|c: char| c.is_ascii_whitespace())
                            .starts_with('!')
                    })
                    .is_some()
                    .then(|| AggregateKind::from_name(&self.source[start..self.pos]))
                    .flatten()
                {
                    self.space();
                    if self.peek() != Some(b'(') {
                        return Err(self.error("expected SUM rectangle"));
                    }
                    self.pos += 1;
                    let mut arguments = Vec::new();
                    loop {
                        self.space();
                        let at = self.pos;
                        let first = self.range_cell().ok();
                        self.space();
                        let argument = if let Some(first) =
                            first.filter(|_| self.peek() == Some(b':'))
                        {
                            self.pos += 1;
                            let last = self.range_cell()?;
                            if last.qualifier.is_some() {
                                return Err(self.error("range endpoint qualifier is not supported"));
                            }
                            AggregateArgument::Range(Expr::SumRange {
                                kind,
                                first: (first.row, first.column),
                                last: (last.row, last.column),
                                span: at..self.pos,
                                first_reference: first,
                                last_reference: last,
                            })
                        } else {
                            self.pos = at;
                            AggregateArgument::Scalar(self.expr(0, depth + 1)?)
                        };
                        arguments.push(argument);
                        self.space();
                        match self.peek() {
                            Some(b',') => {
                                self.pos += 1;
                            }
                            Some(b')') => {
                                self.pos += 1;
                                break;
                            }
                            _ => return Err(self.error("expected comma or closing parenthesis")),
                        }
                    }
                    if arguments.len() == 1 && matches!(arguments[0], AggregateArgument::Range(_)) {
                        let AggregateArgument::Range(mut range) = arguments.pop().unwrap() else {
                            unreachable!()
                        };
                        if let Expr::SumRange { span, .. } = &mut range {
                            *span = start..self.pos;
                        }
                        range
                    } else {
                        Expr::AggregateCall { kind, arguments }
                    }
                } else if self.spreadsheet {
                    self.pos = start;
                    let reference = self.range_cell()?;
                    Expr::CellReference {
                        qualifier: reference.qualifier,
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
        let mut compared = false;
        loop {
            self.space();
            if self.spreadsheet {
                let rest = &self.source[self.pos..];
                let typed = [
                    ("<>", crate::typed::Op::NotEqual),
                    ("<=", crate::typed::Op::LessEqual),
                    (">=", crate::typed::Op::GreaterEqual),
                    ("=", crate::typed::Op::Equal),
                    ("<", crate::typed::Op::Less),
                    (">", crate::typed::Op::Greater),
                    ("&", crate::typed::Op::Concat),
                ]
                .into_iter()
                .find(|(s, _)| rest.starts_with(s));
                if let Some((symbol, op)) = typed {
                    let power = if symbol == "&" { 2 } else { 0 };
                    if power < min {
                        break;
                    }
                    if power == 0 && compared {
                        return Err(self.error("chained comparison"));
                    }
                    compared |= power == 0;
                    self.pos += symbol.len();
                    self.work += 1;
                    let right = self.expr(power + 1, depth + 1)?;
                    left = Expr::Typed {
                        op,
                        arguments: vec![left, right],
                    };
                    continue;
                }
            }
            let (op, power) = match self.peek() {
                Some(b'+') => (BinaryOp::Add, 4),
                Some(b'-') => (BinaryOp::Subtract, 4),
                Some(b'*') => (BinaryOp::Multiply, 6),
                Some(b'/') => (BinaryOp::Divide, 6),
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
    fn aggregate_lists_are_bounded_and_reject_missing_or_nonrectangular_arguments() {
        for source in [
            "=SUM(1,2)",
            "=AVERAGE(A1:A3,2*3,MAX(1,2))",
            "=COUNT(A1)",
            "=MIN(A1:A2,B1:B3)",
        ] {
            assert!(
                matches!(parse_spreadsheet(source), Ok(Expr::AggregateCall { .. })),
                "{source}"
            );
            assert!(parse(source).is_err());
        }
        for source in [
            "=SUM()",
            "=COUNT(,1)",
            "=SUM(1,)",
            "=MIN(A1:A2,,3)",
            "=SUM(A1:A2+3)",
            "=SUM(A1:B2:C3)",
            "=SUM(1;2)",
            "=AVERAGE(1,2",
        ] {
            assert!(parse_spreadsheet(source).is_err(), "{source}");
        }
        let nested = format!("{}1{}", "SUM(".repeat(65), ")".repeat(65));
        assert!(parse_spreadsheet(&nested).is_err());
        assert!(parse_spreadsheet(&format!("SUM({})", "1,".repeat(3000))).is_err());
    }
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
                ..
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
