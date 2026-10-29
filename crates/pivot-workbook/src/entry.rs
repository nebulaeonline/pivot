//! Rust-authoritative single-line worksheet entry admission.
use super::*;
/// Literal String values may contain CSV line breaks and tabs. Editors retain
/// their existing single-line admission; this validates stored value content.
pub fn validate_value_text(text: &str, limit: usize) -> Result<()> {
    if text.len() > limit {
        return Err(Error::Limit);
    }
    if text.chars().any(|ch| {
        (ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
            || matches!(ch, '\u{2028}' | '\u{2029}')
    }) {
        return Err(Error::InvalidTarget);
    }
    Ok(())
}
pub(crate) fn validate_text(text: &str, limit: usize) -> Result<()> {
    if text.len() > limit {
        return Err(Error::Limit);
    }
    if text
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(Error::InvalidTarget);
    }
    Ok(())
}
pub fn classify(position: Position, source: &str, limit: usize) -> Result<Edit> {
    if source.is_empty() {
        return Ok(Edit::Clear(position));
    }
    if let Some(text) = source.strip_prefix('\'') {
        validate_text(text, limit)?;
        return Ok(Edit::SetText {
            position,
            text: text.into(),
        });
    }
    if source.len() > limit {
        return Err(Error::Limit);
    }
    let trimmed = source.trim();
    if trimmed.starts_with('=') {
        return Ok(Edit::SetFormulaText {
            position,
            source: source.into(),
        });
    }
    if trimmed.eq_ignore_ascii_case("TRUE") || trimmed.eq_ignore_ascii_case("FALSE") {
        return Ok(Edit::SetBoolean {
            position,
            value: trimmed.eq_ignore_ascii_case("TRUE"),
        });
    }
    let body = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
    let numeric = body.bytes().any(|b| b.is_ascii_digit())
        && body.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        && body.bytes().filter(|&b| b == b'.').count() <= 1;
    if numeric {
        let scale = body.split_once('.').map_or(0, |(_, f)| f.len());
        if scale > 28 {
            return Err(Error::Limit);
        }
        let value: PivotDecimal = trimmed.parse().map_err(Error::Numeric)?;
        let (lo, mid, hi, negative, actual_scale) = value.parts();
        if actual_scale as usize != scale {
            return Err(Error::Limit);
        }
        let value = DecimalInput::from_parts(
            lo,
            mid,
            hi,
            negative || trimmed.starts_with('-'),
            actual_scale,
        )?;
        return Ok(Edit::SetDecimal { position, value });
    }
    validate_text(source, limit)?;
    Ok(Edit::SetText {
        position,
        text: source.into(),
    })
}
