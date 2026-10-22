use pivot_core::{captured::*, model::integration::IntegrationRuntime};
use pivot_workbook::*;
use uuid::Uuid;
#[test]
fn actual_worksheet_source_dependents_sum_and_old_context() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("binding", Limits::default(), &runtime).unwrap();
    b.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let source = "=@{InvoiceLines.TotalAmount}*1.1";
    let occurrences = b.resolve_structured(source).unwrap();
    b.apply_structured(b.expected(), Position::new(1, 1), source, &occurrences)
        .unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::SetFormulaText {
                position: Position::new(2, 1),
                source: "=A1+1".into(),
            },
            Edit::SetFormulaText {
                position: Position::new(3, 1),
                source: "=SUM(A1:A2)".into(),
            },
        ],
    )
    .unwrap();
    let mut first = b.capture_local(b.expected()).unwrap();
    first
        .prepare_structured_context(ScalarValue::Decimal(60.into()), &[])
        .unwrap();
    let outputs = [
        first.cell_at(Position::new(1, 1)).unwrap(),
        first.cell_at(Position::new(3, 1)).unwrap(),
    ];
    let result = first.evaluate(&outputs).unwrap();
    assert_eq!(result.values[0].1, Ok(66.into()));
    assert_eq!(result.values[1].1, Ok(133.into()));
    let mut second = b.capture_local(b.expected()).unwrap();
    second
        .prepare_structured_context(ScalarValue::Decimal(150.into()), &[])
        .unwrap();
    assert_eq!(
        second.evaluate(&outputs).unwrap().values[0].1,
        Ok(165.into())
    );
    assert_eq!(first.evaluate(&outputs).unwrap().values[0].1, Ok(66.into()));
    drop((first, second, b));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn distinct_formula_payloads_are_shared_across_captures_and_retained_to_final_owner() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("ownership", Limits::default(), &runtime).unwrap();
    book.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let original = "=@{InvoiceLines.TotalAmount}";
    for row in 1..=32 {
        let occurrences = book.resolve_structured(original).unwrap();
        book.apply_structured(
            book.expected(),
            Position::new(row, 1),
            original,
            &occurrences,
        )
        .unwrap();
    }
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().versions, 32);
    let mut first = book.capture_local(book.expected()).unwrap();
    first
        .prepare_structured_context(ScalarValue::Decimal(60.into()), &[])
        .unwrap();
    let mut second = book.capture_local(book.expected()).unwrap();
    second
        .prepare_structured_context(ScalarValue::Decimal(60.into()), &[])
        .unwrap();
    assert_eq!(
        book.structured_usage().unwrap().versions,
        32,
        "captures share immutable payload allocations"
    );
    assert_eq!(book.structured_usage().unwrap().slots, 1);
    let mut third = book.capture_local(book.expected()).unwrap();
    assert!(
        third
            .prepare_structured_context(ScalarValue::Decimal(60.into()), &[])
            .is_err()
    );
    drop(third);
    let overflow = book.resolve_structured(original).unwrap();
    let before = book.expected();
    assert!(
        book.apply_structured(before, Position::new(33, 1), original, &overflow)
            .is_err()
    );
    assert_eq!(book.expected(), before);
    let too_many = std::iter::repeat_n("@{InvoiceLines.TotalAmount}", 17)
        .collect::<Vec<_>>()
        .join("+");
    assert!(book.resolve_structured(&format!("={too_many}")).is_err());
    for row in 1..=32 {
        let source = "=@{InvoiceLines.TotalAmount}+1";
        let occurrences = book.resolve_structured(source).unwrap();
        book.apply_structured(book.expected(), Position::new(row, 1), source, &occurrences)
            .unwrap();
        runtime.flush();
    }
    assert_eq!(book.structured_usage().unwrap().versions, 64);
    let expected = book.expected();
    let source = "=@{InvoiceLines.TotalAmount}+2";
    let occurrences = book.resolve_structured(source).unwrap();
    assert!(
        book.apply_structured(expected, Position::new(1, 1), source, &occurrences)
            .is_err()
    );
    runtime.flush();
    assert_eq!(book.expected(), expected);
    assert_eq!(book.structured_usage().unwrap().versions, 64);
    drop(first);
    runtime.flush();
    assert_eq!(
        book.structured_usage().unwrap().versions,
        64,
        "second owner retains the same old allocations"
    );
    drop(second);
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().versions, 32);
    book.apply_structured(expected, Position::new(1, 1), source, &occurrences)
        .unwrap();
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().versions, 32);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn spans_identity_copy_and_fault_propagation_use_the_canonical_graph() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("contracts", Limits::default(), &runtime).unwrap();
    assert!(
        book.enable_structured_proof(Uuid::nil(), Uuid::nil())
            .is_err()
    );
    book.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let source = "  =@{InvoiceLines.TotalAmount}+@{InvoiceLines.TotalAmount}+A1";
    let occurrences = book.resolve_structured(source).unwrap();
    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        &source[occurrences[0].span.clone()],
        "@{InvoiceLines.TotalAmount}"
    );
    let expected = book.expected();
    let mut malformed = occurrences.clone();
    malformed[0].span.start += 1;
    assert!(
        book.apply_structured(expected, Position::new(1, 2), source, &malformed)
            .is_err()
    );
    malformed = occurrences.clone();
    malformed[0].binding.document = Uuid::new_v4();
    assert!(
        book.apply_structured(expected, Position::new(1, 2), source, &malformed)
            .is_err()
    );
    assert_eq!(book.expected(), expected);
    book.apply_structured(expected, Position::new(1, 2), source, &occurrences)
        .unwrap();
    book.rename_structured_producer(book.expected(), "Renamed")
        .unwrap();
    book.rename_structured_output(book.expected(), "Subtotal")
        .unwrap();
    assert!(book.resolve_structured(source).is_err());
    book.duplicate_structured_block(
        book.expected(),
        Rectangle {
            first: Position::new(1, 2),
            last: Position::new(1, 2),
        },
        Rectangle {
            first: Position::new(2, 2),
            last: Position::new(2, 2),
        },
    )
    .unwrap();
    assert!(
        book.copy_block(
            book.expected(),
            Rectangle {
                first: Position::new(1, 2),
                last: Position::new(1, 2)
            }
        )
        .is_err()
    );
    let expected = book.expected();
    assert!(
        book.duplicate_structured_block(
            expected,
            Rectangle {
                first: Position::new(0, 1),
                last: Position::new(u32::MAX, 1)
            },
            Rectangle {
                first: Position::new(1, 1),
                last: Position::new(1, 1)
            }
        )
        .is_err()
    );
    assert_eq!(book.expected(), expected);
    assert!(
        book.paste_block(
            expected,
            Rectangle {
                first: Position::new(3, 1),
                last: Position::new(3, 1)
            },
            b"7\t=@{Renamed.Subtotal}",
            false,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(book.expected(), expected);
    let mut capture = book.capture_local(book.expected()).unwrap();
    capture
        .prepare_structured_failure(pivot_workbook::structured::SourceFault::UnavailableResult)
        .unwrap();
    let outputs = [
        capture.cell_at(Position::new(1, 2)).unwrap(),
        capture.cell_at(Position::new(2, 2)).unwrap(),
    ];
    let result = capture.evaluate(&outputs).unwrap();
    assert_eq!(
        result.values[0].1, result.values[1].1,
        "one capture-scoped fault identity survives repeated references and duplicate consumers"
    );
    drop((capture, book));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn retired_slot_and_registry_survive_until_the_final_capture_releases_them() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("retirement", Limits::default(), &runtime).unwrap();
    book.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let source = "=@{InvoiceLines.TotalAmount}";
    let old_binding = book.resolve_structured(source).unwrap();
    book.apply_structured(book.expected(), Position::new(1, 1), source, &old_binding)
        .unwrap();
    let mut old = book.capture_local(book.expected()).unwrap();
    old.prepare_structured_context(ScalarValue::Decimal(60.into()), &[])
        .unwrap();
    book.retire_structured_producer(book.expected(), false)
        .unwrap();
    let mut missing = book.capture_local(book.expected()).unwrap();
    missing
        .prepare_structured_context(ScalarValue::Decimal(999.into()), &[])
        .unwrap();
    let reference = old.cell_at(Position::new(1, 1)).unwrap();
    assert_eq!(
        old.evaluate(&[reference]).unwrap().values[0].1,
        Ok(60.into())
    );
    assert!(matches!(
        missing.evaluate(&[reference]).unwrap().values[0].1,
        Err(CellError::CapturedSourceFailed {
            kind: pivot_workbook::structured::SourceFault::MissingProducer,
            ..
        })
    ));
    book.retire_structured_producer(book.expected(), true)
        .unwrap();
    let new_binding = book.resolve_structured(source).unwrap();
    assert_ne!(old_binding[0].binding, new_binding[0].binding);
    let expected = book.expected();
    assert!(
        book.apply_structured(expected, Position::new(1, 1), source, &new_binding)
            .is_err()
    );
    assert_eq!(book.expected(), expected);
    book.apply(book.expected(), &[Edit::Clear(Position::new(1, 1))])
        .unwrap();
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().slots, 1);
    assert!(
        book.structured_usage().unwrap().entries >= 4,
        "retired registry entries stay owned with the old adapter"
    );
    let expected = book.expected();
    assert!(
        book.apply_structured(expected, Position::new(1, 1), source, &new_binding)
            .is_err()
    );
    drop(old);
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().slots, 1);
    drop(missing);
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().slots, 0);
    assert_eq!(book.structured_usage().unwrap().entries, 2);
    book.apply_structured(expected, Position::new(1, 1), source, &new_binding)
        .unwrap();
    let mut current = book.capture_local(book.expected()).unwrap();
    current
        .prepare_structured_context(ScalarValue::Decimal(150.into()), &[])
        .unwrap();
    assert_eq!(
        current
            .evaluate(&[current.cell_at(Position::new(1, 1)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(150.into())
    );
    drop((current, book));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn captured_fault_and_arithmetic_precedence_and_row_major_sum_use_real_evaluator() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("error order", Limits::default(), &runtime).unwrap();
    book.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    for (position, source) in [
        (Position::new(1, 1), "=@{InvoiceLines.TotalAmount}+(1/0)"),
        (Position::new(1, 2), "=(1/0)+@{InvoiceLines.TotalAmount}"),
    ] {
        let occurrences = book.resolve_structured(source).unwrap();
        book.apply_structured(book.expected(), position, source, &occurrences)
            .unwrap();
    }
    book.apply(
        book.expected(),
        &[
            Edit::SetFormulaText {
                position: Position::new(2, 1),
                source: "=A1".into(),
            },
            Edit::SetFormulaText {
                position: Position::new(3, 1),
                source: "=1/0".into(),
            },
            Edit::SetFormulaText {
                position: Position::new(4, 1),
                source: "=A1".into(),
            },
            Edit::SetFormulaText {
                position: Position::new(2, 2),
                source: "=SUM(A2:A3)".into(),
            },
            Edit::SetFormulaText {
                position: Position::new(3, 2),
                source: "=SUM(A3:A4)".into(),
            },
            Edit::SetEntry {
                position: Position::new(5, 1),
                source: "7".into(),
            },
        ],
    )
    .unwrap();
    let mut capture = book.capture_local(book.expected()).unwrap();
    capture
        .prepare_structured_failure(pivot_workbook::structured::SourceFault::UnavailableResult)
        .unwrap();
    let positions = [
        Position::new(1, 1),
        Position::new(1, 2),
        Position::new(2, 2),
        Position::new(3, 2),
        Position::new(5, 1),
    ];
    let refs: Vec<_> = positions
        .iter()
        .map(|p| capture.cell_at(*p).unwrap())
        .collect();
    let values = capture.evaluate(&refs).unwrap().values;
    assert!(matches!(
        values[0].1,
        Err(CellError::CapturedSourceFailed { .. })
    ));
    assert_eq!(
        values[1].1,
        Err(CellError::Arithmetic(
            pivot_types::DecimalError::DivisionByZero
        ))
    );
    assert_eq!(
        values[2].1, values[0].1,
        "SUM source-first retains the same fault identity"
    );
    assert_eq!(
        values[3].1,
        Err(CellError::DependencyFailed),
        "SUM arithmetic-first must not select the later source fault"
    );
    assert_eq!(
        values[4].1,
        Ok(7.into()),
        "independent outputs remain calculable"
    );
    drop((capture, book));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn retained_generations_and_formula_versions_share_one_budget_and_reject_atomically() {
    use pivot_workbook::projection::{self, Record, State};
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut state = State::new(&runtime).unwrap();
    let description_bytes = (260_000 - 256) / 64 - std::mem::size_of::<Record>() - 256 - 16;
    let rows: Vec<_> = (0..64)
        .map(|i| Record {
            key: format!("L{i}"),
            description: "x".repeat(description_bytes),
            quantity: 1.into(),
            unit_price: 1.into(),
            amount: 1.into(),
        })
        .collect();
    state.publish(state.epoch, state.schema, &rows).unwrap();
    let mut book = Workbook::new_in("joint budget", Limits::default(), &runtime).unwrap();
    state.enable_binding_book(&mut book).unwrap();
    for row in 1..=32 {
        let source = "=@{InvoiceLines.TotalAmount}";
        let occurrences = book.resolve_structured(source).unwrap();
        book.apply_structured(book.expected(), Position::new(row, 1), source, &occurrences)
            .unwrap();
    }
    let producer_capture = state.capture().unwrap();
    let mut worksheet_capture = book.capture_local(book.expected()).unwrap();
    worksheet_capture
        .prepare_structured_context(ScalarValue::Decimal(state.current_total().unwrap()), &[])
        .unwrap();
    for row in 1..=32 {
        let source = "=@{InvoiceLines.TotalAmount}+1";
        let occurrences = book.resolve_structured(source).unwrap();
        book.apply_structured(book.expected(), Position::new(row, 1), source, &occurrences)
            .unwrap();
    }
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().versions, 64);
    let epoch = state.epoch;
    let generation = state.current_generation();
    assert_eq!(
        state.publish(epoch, state.schema, &rows),
        Err(projection::Error::Budget)
    );
    assert_eq!(state.epoch, epoch);
    assert_eq!(state.current_generation(), generation);
    assert_eq!(state.current_total().unwrap(), 64.into());
    let output = worksheet_capture.cell_at(Position::new(1, 1)).unwrap();
    assert_eq!(
        worksheet_capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(64.into())
    );
    drop((producer_capture, worksheet_capture));
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().versions, 32);
    state.publish(epoch, state.schema, &rows).unwrap();
    drop((state, book));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn retained_registry_limit_and_explicit_document_fault_kinds_are_bounded() {
    use pivot_workbook::structured::SourceFault;
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("registry", Limits::default(), &runtime).unwrap();
    book.enable_structured_proof(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let mut captures = Vec::new();
    for index in 0..31 {
        captures.push(book.capture_local(book.expected()).unwrap());
        book.rename_structured_output(book.expected(), &format!("Output{index}"))
            .unwrap();
        runtime.flush();
    }
    assert_eq!(book.structured_usage().unwrap().entries, 64);
    let expected = book.expected();
    assert!(book.rename_structured_output(expected, "Overflow").is_err());
    assert_eq!(book.expected(), expected);
    drop(captures);
    runtime.flush();
    assert_eq!(book.structured_usage().unwrap().entries, 2);
    book.rename_structured_output(expected, "TotalAmount")
        .unwrap();
    let source = "=@{InvoiceLines.TotalAmount}";
    let occurrences = book.resolve_structured(source).unwrap();
    book.apply_structured(book.expected(), Position::new(1, 1), source, &occurrences)
        .unwrap();
    for kind in [
        SourceFault::SchemaMismatch,
        SourceFault::TypeMismatch,
        SourceFault::NumericOverflow,
    ] {
        let mut capture = book.capture_local(book.expected()).unwrap();
        capture.prepare_structured_failure(kind).unwrap();
        let output = capture.cell_at(Position::new(1, 1)).unwrap();
        assert!(
            matches!(capture.evaluate(&[output]).unwrap().values[0].1,Err(CellError::CapturedSourceFailed{kind:actual,..}) if actual==kind)
        );
        drop(capture);
        runtime.flush();
    }
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
