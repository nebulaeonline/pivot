use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotDecimal;
use pivot_workbook::projection::*;
fn run(test: impl FnOnce(&mut State)) {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut proof = State::new(&runtime).unwrap();
    test(&mut proof);
    drop(proof);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
fn publish(p: &mut State, n: usize) {
    p.publish(p.epoch, p.schema, &State::fixture(n).unwrap())
        .unwrap();
}
#[test]
fn generations_keys_canonical_order_and_detached_numeric_invocation() {
    run(|p| {
        assert!(matches!(p.capture(), Err(Error::Unavailable)));
        publish(p, 3);
        let old = p.capture().unwrap();
        let original = old.generation.id;
        assert_eq!(
            old.generation.record("L1").unwrap().description,
            old.generation.record("L2").unwrap().description
        );
        publish(p, 5);
        let current = p.capture().unwrap();
        assert_ne!(original, current.generation.id);
        std::thread::scope(|s| {
            let job = s.spawn(|| old.evaluate().unwrap());
            let mut next = State::fixture(3).unwrap();
            next.retain(|r| r.key != "L2");
            p.publish(p.epoch, p.schema, &next).unwrap();
            assert_eq!(job.join().unwrap().0.to_string(), "60");
        });
        assert_eq!(current.evaluate().unwrap().0.to_string(), "150");
        drop(current);
        let missing = p.capture().unwrap();
        assert!(matches!(
            missing.generation.record("L2"),
            Err(Error::MissingRecord)
        ));
        drop(missing);
        publish(p, 3);
        let returned = p.capture().unwrap();
        assert!(returned.generation.record("L2").is_ok());
        drop(returned);
        let epoch = p.epoch;
        let id = p.projections[0].id;
        p.layout(p.presentation, id, 4, true).unwrap();
        assert_eq!(p.epoch, epoch);
        assert_eq!(old.evaluate().unwrap().0.to_string(), "60");
        drop(old);
        publish(p, 0);
        assert_eq!(
            p.capture().unwrap().evaluate().unwrap().0,
            PivotDecimal::ZERO
        );
    });
}
#[test]
fn rejected_publication_and_canonical_overflow_preserve_last_result() {
    run(|p| {
        publish(p, 3);
        let capture = p.capture().unwrap();
        let initial = capture.generation.id;
        let epoch = p.epoch;
        let mut rows = State::fixture(3).unwrap();
        rows[1].key = rows[0].key.clone();
        assert_eq!(p.publish(epoch, p.schema, &rows), Err(Error::DuplicateKey));
        rows[1].key.clear();
        assert_eq!(p.publish(epoch, p.schema, &rows), Err(Error::MissingKey));
        assert_eq!(
            p.publish(epoch, SchemaId(uuid::Uuid::new_v4()), &rows),
            Err(Error::Schema)
        );
        assert_eq!(p.epoch, epoch);
        assert_eq!(p.capture().unwrap().generation.id, initial);
        rows = State::fixture(3).unwrap();
        rows[0].amount = "79228162514264337593543950335".parse().unwrap();
        rows[1].amount = 1.into();
        rows[2].amount = (-1).into();
        p.publish(epoch, p.schema, &rows).unwrap();
        let overflow = p.capture().unwrap();
        assert_eq!(overflow.evaluate(), Err(Error::Numeric));
        p.layout(p.presentation, p.projections[0].id, 8, true)
            .unwrap();
        assert_eq!(overflow.evaluate(), Err(Error::Numeric));
        assert_eq!(capture.evaluate().unwrap().0.to_string(), "60");
    });
}
#[test]
fn reflow_two_projections_do_not_materialize_and_frames_are_bounded() {
    run(|p| {
        publish(p, 3);
        let first = p.view(1, 20).unwrap();
        let headers = first
            .rows
            .iter()
            .filter(|r| r.kind == 1)
            .collect::<Vec<_>>();
        assert_eq!(headers.len(), 2);
        assert_ne!(headers[0].projection, headers[1].projection);
        assert_eq!(first.rows.iter().position(|r| r.ordinary == 9), Some(18));
        drop(first);
        let generation = p.capture().unwrap();
        let baseline = std::sync::Arc::strong_count(&generation.generation);
        let second = p.view(8, 20).unwrap();
        assert_eq!(
            std::sync::Arc::strong_count(&generation.generation),
            baseline
        );
        assert!(second.rows.len() <= 20);
        drop(second);
        drop(generation);
        publish(p, 5);
        let taller = p.view(19, 20).unwrap();
        assert_eq!(taller.rows.iter().position(|r| r.ordinary == 9), Some(4));
        drop(taller);
        assert!(matches!(p.view(1, 21), Err(Error::Bounds)));
        assert!(matches!(p.view(0, 20), Err(Error::Bounds)));
    });
}
#[test]
fn captured_generation_pressure_refuses_without_eviction_or_partial_install() {
    run(|p| {
        let mut rows = State::fixture(3).unwrap();
        let sample = rows[0].clone();
        rows = (0..64)
            .map(|i| {
                let mut r = sample.clone();
                r.key = format!("K{i:03}");
                r.description = "x".repeat(3700);
                r
            })
            .collect();
        p.publish(p.epoch, p.schema, &rows).unwrap();
        let one = p.capture().unwrap();
        p.publish(p.epoch, p.schema, &rows).unwrap();
        let two = p.capture().unwrap();
        assert!(matches!(p.capture(), Err(Error::Budget)));
        p.publish(p.epoch, p.schema, &rows).unwrap();
        let epoch = p.epoch;
        let before = p.used_bytes();
        assert_eq!(p.publish(epoch, p.schema, &rows), Err(Error::Budget));
        assert_eq!(p.epoch, epoch);
        assert_eq!(p.used_bytes(), before);
        assert!(one.generation.record("K000").is_ok());
        assert!(two.generation.record("K000").is_ok());
        drop(one);
        p.publish(epoch, p.schema, &rows).unwrap();
        drop(two);
    });
}
