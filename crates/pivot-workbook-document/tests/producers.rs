use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotDecimal;
use pivot_workbook::{producers::*, *};
use pivot_workbook_document::Document;
fn setup(rt: &IntegrationRuntime) -> Workbook {
    let mut b = Workbook::new_in("native producers", Limits::default(), rt).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(2, 2),
                source: "2".into(),
            },
            Edit::SetEntry {
                position: Position::new(3, 2),
                source: "5".into(),
            },
            Edit::SetEntry {
                position: Position::new(4, 2),
                source: "3".into(),
            },
        ],
    )
    .unwrap();
    let worksheet = b
        .create_worksheet_producer(
            b.expected(),
            sheet,
            Rectangle {
                first: Position::new(2, 2),
                last: Position::new(4, 2),
            },
        )
        .unwrap();
    b.create_bound_window(b.expected(), "Items", worksheet)
        .unwrap();
    let fixture = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let t = b.begin_refresh(b.expected(), fixture).unwrap();
    b.refresh_fixture(&t, 0).unwrap();
    drop(t);
    b.create_bound_window(b.expected(), "External", fixture)
        .unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(10, 1),
                source: "=Items|A2+External|A1".into(),
            },
            Edit::SetEntry {
                position: Position::new(11, 1),
                source: "=SUM(Items|A1:A3)".into(),
            },
            Edit::SetEntry {
                position: Position::new(12, 1),
                source: "=ALLOCATEWEIGHTEDRANGE(10,2,2,Items|A1:A3)".into(),
            },
        ],
    )
    .unwrap();
    b
}
#[test]
fn wb15_restores_complete_bound_authority_and_disconnected_data() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let b = setup(&rt);
    let saved = Document::export_document(&b, b.expected()).unwrap();
    assert!(saved.bytes.starts_with(b"PIVOTWB15"));
    let (mut reopened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    let capture = reopened.capture_local(reopened.expected()).unwrap();
    let outputs = [10, 11, 12].map(|r| capture.cell_at(Position::new(r, 1)).unwrap());
    assert_eq!(
        capture
            .evaluate(&outputs)
            .unwrap()
            .values
            .iter()
            .map(|v| v.1.clone().unwrap().to_string())
            .collect::<Vec<_>>(),
        ["6", "10", "5"]
    );
    let fixture = reopened
        .producers()
        .iter()
        .find(|p| matches!(p.source, Source::Fixture { .. }))
        .unwrap()
        .id;
    assert!(
        reopened
            .begin_refresh(reopened.expected(), fixture)
            .is_err()
    );
    let GridScopeId::Worksheet(sheet) = reopened.worksheet() else {
        unreachable!()
    };
    reopened
        .restructure(
            sheet.into(),
            reopened.expected(),
            structure::Change {
                axis: structure::AxisKind::Rows,
                action: structure::Action::Insert,
                start: 3,
                count: 1,
            },
            &Default::default(),
        )
        .unwrap();
    let second = Document::export_document(&reopened, reopened.expected()).unwrap();
    let (again, _) = Document::open_document(&second.bytes, Limits::default(), &rt).unwrap();
    let values = again
        .capture_local(again.expected())
        .unwrap()
        .evaluate(&outputs)
        .unwrap()
        .values;
    assert_eq!(values[0].1, Ok(PivotDecimal::from(6)));
    assert_eq!(values[1].1, Ok(PivotDecimal::from(10)));
    assert!(values[2].1.is_err());
    drop((again, second, capture, reopened, saved, b));
    assert!(rt.finish().is_ok());
}
#[test]
fn forged_guard_relationship_refuses_even_with_valid_envelope_digest() {
    use sha2::{Digest, Sha256};
    let rt = IntegrationRuntime::standalone().unwrap();
    let b = setup(&rt);
    let saved = Document::export_document(&b, b.expected()).unwrap();
    let mut bytes = saved.bytes.clone();
    let producer_len = u64::from_le_bytes(bytes[49..57].try_into().unwrap()) as usize;
    let mut image = codec::decode(&bytes[57..57 + producer_len]).unwrap();
    image.guards[0].configuration += 1;
    let forged = codec::encode(&image).unwrap();
    assert_eq!(forged.len(), producer_len);
    bytes[57..57 + producer_len].copy_from_slice(&forged);
    let digest = Sha256::digest(&bytes[49..]);
    bytes[17..49].copy_from_slice(&digest);
    assert!(Document::open_document(&bytes, Limits::default(), &rt).is_err());
    let mut old = saved.bytes.clone();
    old[..9].copy_from_slice(b"PIVOTWB14");
    assert!(Document::open_document(&old, Limits::default(), &rt).is_err());
    drop((b, saved));
    assert!(rt.finish().is_ok());
}

#[test]
fn captured_identity_order_is_independent_of_typed_cell_overlay_slots() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = setup(&rt);
    b.apply(
        b.expected(),
        &[Edit::SetText {
            position: Position::new(1, 3),
            text: "0012".into(),
        }],
    )
    .unwrap();
    let mut image = b.export_portable(b.expected()).unwrap();
    image.producers.outputs[0].portable = GraphNodeId::from_uuid(uuid::Uuid::from_u128(1));
    let restored = Workbook::restore_portable(image, Limits::default(), &rt).unwrap();
    let saved = Document::export_document(&restored, restored.expected()).unwrap();
    let (opened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    let capture = opened.capture_local(opened.expected()).unwrap();
    let cell = capture.cell_at(Position::new(1, 3)).unwrap();
    assert_eq!(
        capture.evaluate_values(&[cell]).unwrap().values[0].1,
        Ok(pivot_types::PivotValue::String("0012".into()))
    );
    drop((capture, opened, saved, restored, b));
    assert!(rt.finish().is_ok());
}

#[test]
fn forged_fixture_schema_refuses_with_recomputed_envelope_digest() {
    use sha2::{Digest, Sha256};
    let rt = IntegrationRuntime::standalone().unwrap();
    let b = setup(&rt);
    let saved = Document::export_document(&b, b.expected()).unwrap();
    let mut bytes = saved.bytes.clone();
    let length = u64::from_le_bytes(bytes[49..57].try_into().unwrap()) as usize;
    let mut image = codec::decode(&bytes[57..57 + length]).unwrap();
    let source = image
        .producers
        .iter_mut()
        .find_map(|p| {
            if let SourceRecord::Fixture { schema, .. } = &mut p.source {
                Some(schema)
            } else {
                None
            }
        })
        .unwrap();
    // Labels are not covered by the snapshot's typed-data digest. They still
    // must match the installed provider's immutable schema contract.
    source.fields[0].label = "Forged".into();
    let forged = codec::encode(&image).unwrap();
    assert_eq!(forged.len(), length);
    bytes[57..57 + length].copy_from_slice(&forged);
    let digest = Sha256::digest(&bytes[49..]);
    bytes[17..49].copy_from_slice(&digest);
    assert!(Document::open_document(&bytes, Limits::default(), &rt).is_err());
    drop((saved, b));
    assert!(rt.finish().is_ok());
}

#[test]
fn portable_restore_refuses_forged_producer_authority_atomically() {
    use pivot_workbook::transfer::PortableWorkbook;
    let rt = IntegrationRuntime::standalone().unwrap();
    let b = setup(&rt);
    let mutations: [fn(&mut PortableWorkbook); 6] = [
        |image| image.producers.outputs[0].key.configuration += 1,
        |image| image.producers.outputs[0].key.key = "invented-key".into(),
        |image| image.producers.guards[0].window = WindowId::from_uuid(uuid::Uuid::new_v4()),
        |image| {
            let id = image.producers.producers[0].id;
            image.producers.producers[1].id = id;
        },
        |image| {
            let window = image.producers.windows[0].window;
            image
                .style_defaults
                .push((window.into(), formatting::Style::default()));
        },
        |image| {
            if let Some(SourceRecord::Fixture { policy, .. }) =
                image.producers.producers.iter_mut().find_map(|p| {
                    matches!(p.source, SourceRecord::Fixture { .. }).then_some(&mut p.source)
                })
            {
                *policy = KeyPolicy::GenerationQualified;
            }
        },
    ];
    for mutate in mutations {
        let mut image = b.export_portable(b.expected()).unwrap();
        mutate(&mut image);
        rt.flush();
        let charged = rt.memory().stats().charged;
        assert!(Workbook::restore_portable(image, Limits::default(), &rt).is_err());
        rt.flush();
        assert_eq!(
            rt.memory().stats().charged,
            charged,
            "failed candidate released all charged ownership"
        );
    }
    drop(b);
    assert!(rt.finish().is_ok());
}
