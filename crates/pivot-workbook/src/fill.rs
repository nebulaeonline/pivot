//! Trusted same-document duplication and bounded four-direction Fill.
use crate::*;
use pivot_core::model::integration::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillDirection {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Default)]
struct Work {
    source: usize,
    tokens: usize,
    references: usize,
}
impl Work {
    fn source(&mut self, n: usize) -> Result<()> {
        self.source += n;
        self.check()
    }
    fn check(&self) -> Result<()> {
        if self.source > 128 * 1024 || self.tokens > 16_384 || self.references > 8192 {
            Err(Error::Limit)
        } else {
            Ok(())
        }
    }
}
fn poll(cancel: &CancellationToken) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::Graph("cancelled".into()))
    } else {
        Ok(())
    }
}
impl Workbook {
    pub fn fill(
        &mut self,
        expected: Expected,
        rectangle: Rectangle,
        direction: FillDirection,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        poll(cancel)?;
        self.axes_at(rectangle.first)?;
        self.axes_at(rectangle.last)?;
        let height = rectangle
            .last
            .row
            .checked_sub(rectangle.first.row)
            .and_then(|x| x.checked_add(1))
            .ok_or(Error::Position)?;
        let width = rectangle
            .last
            .column
            .checked_sub(rectangle.first.column)
            .and_then(|x| x.checked_add(1))
            .ok_or(Error::Position)?;
        if matches!(direction, FillDirection::Up | FillDirection::Down) && height == 1
            || matches!(direction, FillDirection::Left | FillDirection::Right) && width == 1
        {
            poll(cancel)?;
            return Ok(expected);
        }
        if u64::from(height) * u64::from(width) > 256 {
            return Err(Error::Limit);
        }
        let pairs = (rectangle.first.row..=rectangle.last.row).flat_map(|r| {
            (rectangle.first.column..=rectangle.last.column).filter_map(move |c| {
                let at = Position::new(r, c);
                let from = match direction {
                    FillDirection::Down => Position::new(rectangle.first.row, c),
                    FillDirection::Up => Position::new(rectangle.last.row, c),
                    FillDirection::Right => Position::new(r, rectangle.first.column),
                    FillDirection::Left => Position::new(r, rectangle.last.column),
                };
                (from != at).then_some((from, at))
            })
        });
        self.duplicate_pairs(expected, pairs, cancel)
    }
    pub(crate) fn duplicate_pairs(
        &mut self,
        expected: Expected,
        pairs: impl Iterator<Item = (Position, Position)>,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        poll(cancel)?;
        let _scratch = self
            .core
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        let capture = self.capture_local(expected)?;
        poll(cancel)?;
        let mut work = Work::default();
        let mut edits = Vec::new();
        for (from, at) in pairs {
            poll(cancel)?;
            let edit = self.plan_duplicate(&capture, from, at, &mut work)?;
            if !self.duplicate_unchanged(&capture, &edit)? {
                edits.push(edit);
            }
        }
        drop(capture);
        poll(cancel)?;
        if edits.is_empty() {
            return Ok(expected);
        }
        persistent::apply_fill(self, expected, &edits, cancel)
    }
    fn plan_duplicate(
        &self,
        capture: &CalculationCapture,
        from: Position,
        at: Position,
        work: &mut Work,
    ) -> Result<Edit> {
        let content = capture.pinned_content(from)?.map(|v| v.content);
        let (text, bound) = match content {
            None | Some(Content::Blank) => {
                return Ok(Edit::Clear(at));
            }
            Some(Content::Boolean(value)) => {
                return Ok(Edit::SetBoolean {
                    position: at,
                    value,
                });
            }
            Some(Content::Decimal(value)) => {
                return Ok(Edit::SetDecimal {
                    position: at,
                    value,
                });
            }
            Some(Content::LiteralText(text)) => {
                work.source(text.len() + 1)?;
                return Ok(Edit::SetText { position: at, text });
            }
            Some(Content::TextFormula(f)) if f.diagnostic.is_some() => {
                work.source(f.original_source.len())?;
                return Ok(Edit::PreserveRejectedFormula {
                    position: at,
                    source: f.original_source,
                });
            }
            Some(Content::TextFormula(f)) => (f.original_source, f.tokens),
            Some(Content::Formula { source, tokens }) => (source, tokens),
            Some(Content::StructuredFormula(f)) => (f.source().into(), f.tokens().to_vec()),
        };
        let text = sheets::render_source(&text, &bound, self.sheets(), self.limits.source_bytes)?;
        work.source(text.len())?;
        work.tokens += bound.len();
        work.check()?;
        let proof = bound.iter().any(|t| matches!(t, Token::Output(_)));
        let parsed = if proof {
            expression::parse_structured_proof(&text)
        } else {
            expression::parse_spreadsheet(&text)
        };
        let binding = bound.iter().find_map(|t| {
            if let Token::Output(b) = t {
                Some(*b)
            } else {
                None
            }
        });
        let (syntax, _, diagnostic) =
            Self::lower_text_axes(&self.rows, &self.columns, &parsed, binding);
        if diagnostic.is_some() || syntax.len() != bound.len() {
            return Err(Error::InvalidDraft);
        }
        let mut effective = Vec::new();
        for (syntax, bound) in syntax.iter().zip(&bound) {
            match (syntax, bound) {
                (Token::At(_), Token::Cell(target)) => effective.push(
                    capture
                        .pinned
                        .records
                        .get(&target.cell)
                        .and_then(|r| r.position)
                        .ok_or(Error::InvalidTarget)?,
                ),
                (
                    Token::SumAt { kind, .. },
                    Token::Sum {
                        kind: k,
                        first,
                        last,
                        ..
                    },
                ) if kind == k => {
                    effective.push(*first);
                    effective.push(*last);
                }
                _ => (),
            }
        }
        // Transform syntax in source order; postfix visits references left-to-right too.
        let dr = i64::from(at.row) - i64::from(from.row);
        let dc = i64::from(at.column) - i64::from(from.column);
        work.references += effective.len();
        work.check()?;
        let (text, tokens, references) = clipboard::transform_mode(
            &text,
            &effective,
            dr,
            dc,
            self.rows.length,
            self.columns.length,
            proof,
        )?;
        work.source(text.len())?;
        work.tokens += tokens;
        work.references += references;
        work.check()?;
        let parsed = if proof {
            expression::parse_structured_proof(&text)
        } else {
            expression::parse_spreadsheet(&text)
        };
        let (mut translated, _, diagnostic) =
            Self::lower_text_axes(&self.rows, &self.columns, &parsed, binding);
        if diagnostic.is_some() {
            return Err(Error::InvalidDraft);
        }
        let mut preserved = bound.iter().filter_map(|t| {
            if let Token::Output(id) = t {
                Some(*id)
            } else {
                None
            }
        });
        for token in &mut translated {
            if let Token::Output(id) = token {
                *id = preserved.next().ok_or(Error::InvalidTarget)?;
            }
        }
        if preserved.next().is_some() {
            return Err(Error::InvalidTarget);
        }
        Ok(Edit::SetFormula {
            position: at,
            source: text,
            tokens: translated,
        })
    }
    fn duplicate_unchanged(&self, capture: &CalculationCapture, edit: &Edit) -> Result<bool> {
        let at = match edit {
            Edit::Clear(p) => *p,
            Edit::SetDecimal { position, .. }
            | Edit::SetText { position, .. }
            | Edit::PreserveRejectedFormula { position, .. }
            | Edit::SetFormula { position, .. } => *position,
            _ => return Err(Error::InvalidTarget),
        };
        let old = capture.pinned_content(at)?.map(|c| c.content);
        Ok(match (edit, old) {
            (Edit::Clear(_), None | Some(Content::Blank)) => true,
            (Edit::SetDecimal { value, .. }, Some(Content::Decimal(old))) => *value == old,
            (Edit::SetText { text, .. }, Some(Content::LiteralText(old))) => *text == old,
            (Edit::PreserveRejectedFormula { source, .. }, Some(Content::TextFormula(old))) => {
                old.diagnostic.is_some() && old.original_source == *source
            }
            (Edit::SetFormula { source, tokens, .. }, Some(old)) => {
                let (old_source, old_tokens) = match &old {
                    Content::Formula { source, tokens } => (source.as_str(), tokens.as_slice()),
                    Content::TextFormula(f) if f.diagnostic.is_none() => {
                        (f.original_source.as_str(), f.tokens.as_slice())
                    }
                    Content::StructuredFormula(f) => (f.source(), f.tokens()),
                    _ => return Ok(false),
                };
                old_source == source
                    && tokens.len() == old_tokens.len()
                    && tokens.iter().zip(old_tokens).all(|(a, b)| match (a, b) {
                        (
                            Token::QualifierAt { name, span },
                            Token::Qualifier { worksheet, span: s },
                        ) => span == s && self.sheet_by_name(name) == Some(*worksheet),
                        (Token::At(p), Token::Cell(id)) => {
                            capture
                                .pinned
                                .records
                                .get(&id.cell)
                                .and_then(|r| r.position)
                                == Some(*p)
                        }
                        (
                            Token::SumAt {
                                kind,
                                first,
                                last,
                                span,
                            },
                            Token::Sum {
                                kind: k,
                                first: f,
                                last: l,
                                span: s,
                                ..
                            },
                        ) => kind == k && first == f && last == l && span == s,
                        _ => a == b,
                    })
            }
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pivot_core::model::integration::IntegrationRuntime;
    #[test]
    fn cancellation_after_planning_and_order_independence() {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = Workbook::new_in("planning", Limits::default(), &rt).unwrap();
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: "7".into(),
            }],
        )
        .unwrap();
        let before = b.expected();
        let stats = b.stats();
        let cancel = CancellationToken::default();
        let pairs = (2..=4).map(|r| {
            if r == 4 {
                cancel.cancel();
            }
            (Position::new(1, 1), Position::new(r, 1))
        });
        assert!(b.duplicate_pairs(before, pairs, &cancel).is_err());
        assert_eq!(b.expected(), before);
        assert_eq!(b.stats(), stats);
        assert!(b.cell_at(Position::new(2, 1)).unwrap().is_none());
        b.duplicate_pairs(
            before,
            (2..=4)
                .rev()
                .map(|r| (Position::new(1, 1), Position::new(r, 1))),
            &Default::default(),
        )
        .unwrap();
        let expected = b.expected();
        assert_eq!(
            b.fill(
                expected,
                Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(4, 1)
                },
                FillDirection::Down,
                &Default::default()
            )
            .unwrap(),
            expected
        );
        drop(b);
        let m = rt.memory();
        assert!(rt.finish().is_ok());
        assert_eq!(m.stats().charged, 0);
    }
}
