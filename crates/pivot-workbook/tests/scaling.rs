#![cfg(feature = "scaling-experiment")]
use pivot_types::PivotDecimal;
use pivot_workbook::{scaling::*, *};

#[test]
fn exact_input_metadata_and_arithmetic_are_preserved_in_overlay_captures() {
    let mut shared = SharedWorkbook::new(build(1000, "independent"));
    for value in [
        DecimalInput::from_parts(0, 0, 0, true, 28).unwrap(),
        DecimalInput::from_parts(12300, 0, 0, false, 4).unwrap(),
        DecimalInput::from_parts(1, 0, 0, true, 28).unwrap(),
    ] {
        let p = position(0);
        shared
            .apply(
                shared.expected(),
                &[Edit::SetDecimal { position: p, value }],
            )
            .unwrap();
        let cell = shared.cell_at(p).unwrap().unwrap();
        assert_eq!(cell.content, Content::Decimal(value));
        assert_eq!(
            shared
                .capture()
                .unwrap()
                .value(cell.reference)
                .unwrap()
                .parts(),
            value.value().parts()
        );
    }
}

#[test]
fn calculation_on_another_thread_is_isolated_from_new_inputs_and_topology() {
    let mut shared = SharedWorkbook::new(build(1000, "independent"));
    let output = shared.cell_at(position(999)).unwrap().unwrap().reference;
    let old = shared.capture().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..100 {
            assert_eq!(old.value(output).unwrap(), PivotDecimal::from(2));
        }
        old.version
    });
    shared
        .apply(
            shared.expected(),
            &[Edit::SetDecimal {
                position: position(0),
                value: DecimalInput::from_decimal(PivotDecimal::from(9)),
            }],
        )
        .unwrap();
    assert_eq!(
        shared.capture().unwrap().value(output).unwrap(),
        PivotDecimal::from(10)
    );
    shared
        .apply(
            shared.expected(),
            &[Edit::SetFormulaText {
                position: position(999),
                source: "=A1*2".into(),
            }],
        )
        .unwrap();
    assert_eq!(
        shared.capture().unwrap().value(output).unwrap(),
        PivotDecimal::from(18)
    );
    assert_ne!(worker.join().unwrap(), shared.expected());
}
