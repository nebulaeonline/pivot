use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{Edit, Limits, Position as P, Rectangle, Workbook, formatting::*};
use pivot_workbook_document::Document;
#[test]
fn wb10_preserves_explicit_defaults_inheritance_exact_text_and_old_capture() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut b = Workbook::new_in("WB10", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.apply(
        b.expected(),
        &[Edit::SetText {
            position: P::new(1, 1),
            text: "001234".into(),
        }],
    )
    .unwrap();
    b.set_worksheet_default(
        sheet,
        b.expected(),
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        &Default::default(),
    )
    .unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let one = Rectangle {
        first: P::new(1, 1),
        last: P::new(1, 1),
    };
    b.format_sheet(
        sheet,
        b.expected(),
        one,
        Default::default(),
        true,
        &Default::default(),
    )
    .unwrap();
    assert!(old.style_at(sheet, P::new(1, 1)).unwrap().bold);
    let saved = Document::export_document(&b, b.expected()).unwrap();
    assert!(saved.bytes.starts_with(b"PIVOTWB11"));
    let mut restored = Document::open_document(&saved.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert!(!restored.style_at(sheet, P::new(1, 1)).unwrap().bold);
    assert!(restored.style_at(sheet, P::new(65536, 1024)).unwrap().bold);
    let capture = restored.capture_local(restored.expected()).unwrap();
    assert_eq!(
        capture
            .evaluate_values(&[capture.cell_at(P::new(1, 1)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(pivot_types::PivotValue::String("001234".into()))
    );
    restored
        .format_sheet(
            sheet,
            restored.expected(),
            one,
            StylePatch::inherit(),
            false,
            &Default::default(),
        )
        .unwrap();
    assert!(restored.style_at(sheet, P::new(1, 1)).unwrap().bold);
    let saved2 = Document::export_document(&restored, restored.expected()).unwrap();
    let final_book = Document::open_document(&saved2.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert_eq!(final_book.style_regions(), 0);
    assert_eq!(final_book.stats().allocated_nodes, 1);
    drop((old, capture, final_book, restored, saved2, saved, b));
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn portable_region_overlap_duplicate_defaults_and_bytes_are_rejected() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let b = Workbook::new_in("bad regions", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    for bad in 0..4 {
        let mut image = b.export_portable(b.expected()).unwrap();
        let r = Region {
            sheet,
            property: 5,
            rectangle: Rectangle {
                first: P::new(1, 1),
                last: P::new(5, 5),
            },
            value: 1,
        };
        image.styles.push(r);
        match bad {
            0 => image.styles.push(Region {
                rectangle: Rectangle {
                    first: P::new(2, 2),
                    last: P::new(3, 3),
                },
                ..r
            }),
            1 => {
                image.style_defaults = vec![
                    (
                        sheet,
                        Style {
                            bold: true,
                            ..Default::default()
                        }
                    );
                    2
                ]
            }
            2 => image.styles[0].property = 10,
            _ => image.styles[0].value = 2,
        }
        assert!(Workbook::restore_portable(image, Limits::default(), &rt).is_err());
    }
    let mut image = b.export_portable(b.expected()).unwrap();
    image.styles.push(Region {
        sheet,
        property: 5,
        rectangle: Rectangle {
            first: P::new(1, 1),
            last: P::new(65536, 1024),
        },
        value: 1,
    });
    let mut limits = Limits::default();
    limits.styles.bytes = 0;
    assert!(Workbook::restore_portable(image, limits, &rt).is_err());
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
