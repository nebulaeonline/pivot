use pivot_core::{captured::*, evaluation::*, model::integration::*};

#[test]
fn actual_canonical_source_faults_context_identity_and_precedence() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut m = runtime.create_model("captured").unwrap();
    let mut batch = m.begin_batch(8, 4096).unwrap();
    let (source, slot) = batch.add_captured_scalar("source".into(), [1; 16]).unwrap();
    let first = batch.add_formula("first", "source+(1/0)").unwrap();
    let second = batch.add_formula("second", "(1/0)+source").unwrap();
    let repeat = batch.add_formula("repeat", "source+source").unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    assert!(
        m.snapshot().is_err(),
        "context-free scalar snapshots must reject captured leaves"
    );
    let c = m.capture_calculation().unwrap();
    assert!(c.evaluate(&[first]).is_err());
    assert!(matches!(
        c.scalar_context(&[], &[]),
        Err(ContextError::Missing)
    ));
    let fault = CapturedFaultId([2; 16]);
    assert!(matches!(
        c.scalar_context(&[(slot, ScalarValue::Fault(fault))], &[]),
        Err(ContextError::Fault)
    ));
    assert!(matches!(
        c.scalar_context(
            &[
                (slot, ScalarValue::Decimal(1.into())),
                (slot, ScalarValue::Decimal(2.into()))
            ],
            &[]
        ),
        Err(ContextError::Duplicate)
    ));
    let mut foreign_model = runtime.create_model("foreign context").unwrap();
    let mut foreign_batch = foreign_model.begin_batch(2, 128).unwrap();
    let (_, foreign_slot) = foreign_batch
        .add_captured_scalar("foreign".into(), [9; 16])
        .unwrap();
    foreign_model
        .apply_batch(foreign_batch, &CancellationToken::default())
        .unwrap();
    assert!(matches!(
        c.scalar_context(&[(foreign_slot, ScalarValue::Decimal(1.into()))], &[]),
        Err(ContextError::Unknown)
    ));
    assert!(matches!(
        c.scalar_context(&[(slot, ScalarValue::Decimal(1.into()))], &[fault]),
        Err(ContextError::Fault)
    ));
    drop(foreign_model);
    let context = c
        .scalar_context(&[(slot, ScalarValue::Fault(fault))], &[fault])
        .unwrap();
    let result = c
        .evaluate_scalar_context(&[first, second, repeat], &context)
        .unwrap();
    for consumer in [first, repeat] {
        assert!(
            matches!(result.values()[&consumer],Err(EvaluationError::DependencyFailed{node,dependency,origin,fault_id:Some(id)}) if node==consumer && dependency==source && origin==source && id==fault)
        );
    }
    assert_eq!(
        result.values()[&source],
        Err(EvaluationError::CapturedSourceFailed {
            node: source,
            fault_id: fault
        })
    );
    assert!(matches!(
        result.values()[&second],
        Err(EvaluationError::Arithmetic {
            error: pivot_types::DecimalError::DivisionByZero,
            ..
        })
    ));
    let other = m.capture_calculation().unwrap();
    assert!(other.evaluate_scalar_context(&[first], &context).is_err());
    let numeric = c
        .scalar_context(&[(slot, ScalarValue::Decimal(60.into()))], &[])
        .unwrap();
    assert_eq!(
        c.evaluate_scalar_context(&[repeat], &numeric)
            .unwrap()
            .values()[&repeat],
        Ok(120.into())
    );
    drop((result, numeric, context, other, c, m));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn a_retained_scalar_context_keeps_its_snapshot_and_runtime_owned() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut model = runtime.create_model("context lifetime").unwrap();
    let mut batch = model.begin_batch(2, 128).unwrap();
    let (_node, slot) = batch.add_captured_scalar("source".into(), [7; 16]).unwrap();
    model
        .apply_batch(batch, &CancellationToken::default())
        .unwrap();
    let capture = model.capture_calculation().unwrap();
    let context = capture
        .scalar_context(&[(slot, ScalarValue::Decimal(1.into()))], &[])
        .unwrap();
    drop((capture, model));
    runtime.flush();
    let runtime = match runtime.finish() {
        Err(runtime) => runtime,
        Ok(_) => panic!("retained context must keep the runtime owned"),
    };
    drop(context);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
