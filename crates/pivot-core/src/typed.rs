//! Shared typed-expression contract. No worksheet names, locale or display formatting.
use pivot_types::{DecimalError, PivotDecimal, PivotValue as V};

pub const MAX_TEXT_BYTES: usize = 4096;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Boolean(bool),
    String(String),
    If,
    And,
    Or,
    Not,
    Len,
    Mid,
    Concat,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
impl Op {
    pub fn code(&self) -> u8 {
        match self {
            Self::Boolean(_) => 0,
            Self::String(_) => 1,
            Self::If => 2,
            Self::And => 3,
            Self::Or => 4,
            Self::Not => 5,
            Self::Len => 6,
            Self::Mid => 7,
            Self::Concat => 8,
            Self::Equal => 9,
            Self::NotEqual => 10,
            Self::Less => 11,
            Self::LessEqual => 12,
            Self::Greater => 13,
            Self::GreaterEqual => 14,
        }
    }
    pub fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            2 => Self::If,
            3 => Self::And,
            4 => Self::Or,
            5 => Self::Not,
            6 => Self::Len,
            7 => Self::Mid,
            8 => Self::Concat,
            9 => Self::Equal,
            10 => Self::NotEqual,
            11 => Self::Less,
            12 => Self::LessEqual,
            13 => Self::Greater,
            14 => Self::GreaterEqual,
            _ => return None,
        })
    }
    pub fn named(name: &str) -> Option<Self> {
        Some(match name.to_ascii_uppercase().as_str() {
            "IF" => Self::If,
            "AND" => Self::And,
            "OR" => Self::Or,
            "NOT" => Self::Not,
            "LEN" => Self::Len,
            "MID" => Self::Mid,
            _ => return None,
        })
    }
    pub fn valid_arity(&self, n: usize) -> bool {
        match self {
            Self::Boolean(_) => n == 0,
            Self::String(s) => n == 0 && s.len() <= MAX_TEXT_BYTES,
            Self::If | Self::Mid => n == 3,
            Self::And | Self::Or => n > 0 && n <= 4096,
            Self::Not | Self::Len => n == 1,
            _ => n == 2,
        }
    }
    pub fn heap_bytes(&self) -> usize {
        if let Self::String(s) = self {
            s.capacity()
        } else {
            0
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    TypeMismatch,
    InvalidArgument,
    TextTooLong,
    Arithmetic(DecimalError),
}
pub fn number(value: &V) -> Result<PivotDecimal, Error> {
    match value {
        V::Decimal(v) => Ok(*v),
        V::Blank => Ok(PivotDecimal::ZERO),
        _ => Err(Error::TypeMismatch),
    }
}
pub fn condition(value: &V) -> Result<bool, Error> {
    match value {
        V::Boolean(v) => Ok(*v),
        V::Blank => Ok(false),
        _ => Err(Error::TypeMismatch),
    }
}
pub(crate) fn validate_argument(op: &Op, index: usize, value: &V) -> Result<(), Error> {
    match op {
        Op::Concat | Op::Len => {
            text(value)?;
        }
        Op::Not => {
            condition(value)?;
        }
        Op::Mid if index == 0 => {
            text(value)?;
        }
        Op::Mid => {
            let V::Decimal(n) = value else {
                return Err(Error::TypeMismatch);
            };
            if *n
                < if index == 1 {
                    1.into()
                } else {
                    PivotDecimal::ZERO
                }
                || n.round(0).map_err(Error::Arithmetic)? != *n
            {
                return Err(Error::InvalidArgument);
            }
        }
        _ => (),
    }
    Ok(())
}
fn text(value: &V) -> Result<&str, Error> {
    match value {
        V::String(s) => Ok(s),
        V::Blank => Ok(""),
        _ => Err(Error::TypeMismatch),
    }
}
pub fn apply(op: &Op, args: &[V]) -> Result<V, Error> {
    if !op.valid_arity(args.len()) {
        return Err(Error::InvalidArgument);
    }
    Ok(match op {
        Op::Boolean(b) => V::Boolean(*b),
        Op::String(s) => V::String(s.clone()),
        Op::If => args[if condition(&args[0])? { 1 } else { 2 }].clone(),
        Op::And => V::Boolean(args.iter().try_fold(true, |v, a| Ok(v && condition(a)?))?),
        Op::Or => V::Boolean(args.iter().try_fold(false, |v, a| Ok(v || condition(a)?))?),
        Op::Not => V::Boolean(!condition(&args[0])?),
        Op::Len => V::Decimal((text(&args[0])?.chars().count() as i64).into()),
        Op::Concat => {
            let a = text(&args[0])?;
            let b = text(&args[1])?;
            if a.len().saturating_add(b.len()) > MAX_TEXT_BYTES {
                return Err(Error::TextTooLong);
            }
            let mut joined = String::with_capacity(a.len() + b.len());
            joined.push_str(a);
            joined.push_str(b);
            V::String(joined)
        }
        Op::Mid => {
            let source = text(&args[0])?;
            let start = match &args[1] {
                V::Decimal(d) => *d,
                _ => return Err(Error::TypeMismatch),
            };
            let count = match &args[2] {
                V::Decimal(d) => *d,
                _ => return Err(Error::TypeMismatch),
            };
            if start < 1.into()
                || count < PivotDecimal::ZERO
                || start.round(0).map_err(Error::Arithmetic)? != start
                || count.round(0).map_err(Error::Arithmetic)? != count
            {
                return Err(Error::InvalidArgument);
            }
            let length = source.chars().count();
            if start > (length as i64).into() {
                V::String(String::new())
            } else {
                let start = start
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .map_err(|_| Error::InvalidArgument)?
                    - 1;
                let count = if count >= (length as i64).into() {
                    length
                } else {
                    count
                        .normalize()
                        .to_string()
                        .split('.')
                        .next()
                        .unwrap()
                        .parse::<usize>()
                        .map_err(|_| Error::InvalidArgument)?
                };
                V::String(source.chars().skip(start).take(count).collect())
            }
        }
        _ => {
            use std::cmp::Ordering::*;
            let order = match (&args[0], &args[1]) {
                (V::Decimal(a), V::Decimal(b)) => a.cmp(b),
                (V::Boolean(a), V::Boolean(b)) => a.cmp(b),
                (V::String(a), V::String(b)) => a.chars().cmp(b.chars()),
                (V::Blank, V::Blank) => Equal,
                (V::Blank, V::Decimal(b)) => PivotDecimal::ZERO.cmp(b),
                (V::Decimal(a), V::Blank) => a.cmp(&PivotDecimal::ZERO),
                _ => return Err(Error::TypeMismatch),
            };
            V::Boolean(match op {
                Op::Equal => order == Equal,
                Op::NotEqual => order != Equal,
                Op::Less => order == Less,
                Op::LessEqual => order != Greater,
                Op::Greater => order == Greater,
                Op::GreaterEqual => order != Less,
                _ => unreachable!(),
            })
        }
    })
}
