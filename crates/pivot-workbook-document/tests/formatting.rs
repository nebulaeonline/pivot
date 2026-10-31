use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::formatting::*;
use pivot_workbook::{Edit, Limits, Position, Rectangle, Workbook};
use pivot_workbook_document::Document;
#[test]
fn sparse_styles_roundtrip_and_old_formats_refused() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("styles", Limits::default(), &runtime).unwrap();
    let sheet = b.sheets()[0].id;
    let other = b.add_sheet(b.expected(), Some("O'Brien Sales")).unwrap();
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "1234.5678".into(),
        }],
    )
    .unwrap();
    let rectangle = Rectangle {
        first: Position::new(1, 1),
        last: Position::new(4, 4),
    };
    let patch = StylePatch {
        numeric: Field::Set(NumericFormat::Currency),
        bold: Field::Set(true),
        background: Field::Set(Some(0xffeb9c)),
        ..Default::default()
    };
    b.format_sheet(
        other,
        b.expected(),
        rectangle,
        patch,
        false,
        &Default::default(),
    )
    .unwrap();
    let graph = b.expected().graph;
    let count = b.stats().allocated_nodes;
    let bytes = Document::export_document(&b, b.expected()).unwrap().bytes;
    assert_eq!(&bytes[..9], b"PIVOTWB11");
    let (opened, _) = Document::open_document(&bytes, Limits::default(), &runtime).unwrap();
    assert_eq!(opened.style_regions(), 3);
    assert_eq!(opened.stats().allocated_nodes, count);
    assert_eq!(
        opened
            .style_at(other, Position::new(4, 4))
            .unwrap()
            .background,
        Some(0xffeb9c)
    );
    assert!(
        opened
            .cell_at_sheet(other, Position::new(4, 4))
            .unwrap()
            .is_none()
    );
    assert_eq!(b.expected().graph, graph);
    for old in b'1'..=b'9' {
        let mut obsolete = bytes.clone();
        obsolete.remove(8);
        obsolete[7] = old;
        assert!(Document::open_document(&obsolete, Limits::default(), &runtime).is_err());
    }
    let mut low = Limits::default();
    low.styles.positions = 2;
    assert!(Document::open_document(&bytes, low, &runtime).is_err());
    drop(opened);
    drop(b);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn checksum_valid_invalid_styles_are_rejected() {
    use sha2::{Digest, Sha256};
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("invalid styles", Limits::default(), &runtime).unwrap();
    let sheet = b.sheets()[0].id;
    let style = Style {
        numeric: NumericFormat::Currency,
        bold: true,
        background: Some(0x123456),
        ..Default::default()
    };
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: Position::new(1, 1),
            last: Position::new(1, 1),
        },
        StylePatch {
            numeric: Field::Set(style.numeric),
            bold: Field::Set(true),
            background: Field::Set(style.background),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let bytes = Document::export_document(&b, b.expected()).unwrap().bytes;
    let region = b.regions().find(|r| r.property == 0).unwrap();
    let mut pattern = region.sheet.uuid().as_bytes().to_vec();
    for n in [0u64, 1, 1, 1, 1, 2] {
        pattern.extend(n.to_le_bytes());
    }
    let offset = bytes
        .windows(pattern.len())
        .position(|b| b == pattern)
        .unwrap();
    for (relative, value) in [(16, 10u64), (24, 0), (40, 65537), (56, 4)] {
        let mut malformed = bytes.clone();
        malformed[offset + relative..offset + relative + 8].copy_from_slice(&value.to_le_bytes());
        let hash = Sha256::digest(&malformed[49..]);
        malformed[17..49].copy_from_slice(&hash);
        assert!(
            Document::open_document(&malformed, Limits::default(), &runtime).is_err(),
            "field {relative}"
        );
    }
    assert_eq!(b.style_regions(), 3);
    drop(b);
    assert!(runtime.finish().is_ok());
}
