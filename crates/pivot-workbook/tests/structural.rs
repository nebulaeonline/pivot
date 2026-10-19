#![cfg(feature = "scaling-experiment")]
use pivot_workbook::{structural::*, *};
fn formula(p: Position, s: &str) -> Edit {
    Edit::SetFormulaText {
        position: p,
        source: s.into(),
    }
}
#[test]
fn workbook_roots_commit_atomically_and_exact_input_metadata_is_captured() {
    let a = Position::new(1, 1);
    let b = Position::new(1, 2);
    let c = Position::new(1, 3);
    let mut w = Workbook::new("Atomic", Limits::default()).unwrap();
    w.apply(
        w.expected(),
        &[
            Edit::SetDecimal {
                position: a,
                value: DecimalInput::from_decimal(2.into()),
            },
            Edit::SetDecimal {
                position: b,
                value: DecimalInput::from_decimal(7.into()),
            },
            formula(c, "=A1+1"),
        ],
    )
    .unwrap();
    let mut research = StructuralWorkbook::new(pivot_workbook::scaling::stage(&w)).unwrap();
    let s = research.capture().unwrap();
    let exact = DecimalInput::from_parts(0, 0, 0, true, 28).unwrap();
    let edits = [
        Edit::SetDecimal {
            position: a,
            value: exact,
        },
        formula(c, "=B1*2"),
    ];
    research.apply(research.version(), &edits).unwrap();
    w.apply(w.expected(), &edits).unwrap();
    let t = research.capture().unwrap();
    assert_eq!(s.value(c).unwrap(), 3.into());
    assert_eq!(t.value(c).unwrap(), 14.into());
    assert_eq!(t.metadata(a).unwrap(), Metadata::Decimal(exact));
    assert_eq!(
        w.cell_at(a).unwrap().unwrap().content,
        Content::Decimal(exact)
    );
    assert!(
        matches!(t.metadata(c).unwrap(), Metadata::Formula { source, references, .. } if source == "=B1*2" && references[0].target == w.cell_at(b).unwrap().unwrap().reference)
    );
    let version = research.version();
    let before = research.metadata(c).unwrap();
    assert!(
        research
            .apply(
                version,
                &[formula(c, "=A1+9"), formula(Position::new(2, 1), "=A1")]
            )
            .is_err()
    );
    assert_eq!(research.version(), version);
    assert_eq!(research.metadata(c).unwrap(), before);
    assert!(research.apply(s.version, &[formula(c, "=1")]).is_err());
    research.apply(version, &[formula(c, "=A1+")]).unwrap();
    assert!(research.capture().is_err());
    assert!(matches!(
        research.metadata(c).unwrap(),
        Metadata::Formula {
            diagnostic: Some(_),
            ..
        }
    ));
    research
        .apply(research.version(), &[formula(c, "=B1+3")])
        .unwrap();
    assert_eq!(research.capture().unwrap().value(c).unwrap(), 10.into());
    assert_eq!(t.value(c).unwrap(), 14.into());
}
