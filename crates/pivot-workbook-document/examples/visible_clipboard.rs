//! Bounded 256-slot transfers across maximal row gaps, with four old roots.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    csv,
    formatting::{Field, StylePatch},
    *,
};
use std::{error::Error, time::Instant};
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or("values".into());
    if !["values", "formulas", "styles"].contains(&mode.as_str()) {
        return Err("unknown visible corpus".into());
    }
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let mut text = String::new();
    for row in 1..=65536 {
        if row % 4 == 1 {
            text.push_str(&row.to_string());
        }
        text.push('\n');
    }
    let mut b = csv::parse(text.as_bytes(), csv::Policy::default(), Limits::default())?.workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        Limits::default(),
        &rt,
    )?;
    let sheet = b.worksheet();
    let source: Vec<u32> = (0..256).map(|i| 1 + i * 257).collect();
    let destination: Vec<u32> = (0..256).map(|i| 1 + i * 256).collect();
    let source_column = if mode == "formulas" { 2 } else { 1 };
    if mode == "formulas" {
        let edits = source
            .iter()
            .map(|row| Edit::SetFormulaText {
                position: Position::new(*row, 2),
                source: format!("=A{row}"),
            })
            .collect::<Vec<_>>();
        b.apply(b.expected(), &edits)?;
    }
    if mode == "styles" {
        for (i, row) in source.iter().enumerate() {
            b.format_sheet(
                sheet,
                b.expected(),
                Rectangle {
                    first: Position::new(*row, 1),
                    last: Position::new(*row, 1),
                },
                StylePatch {
                    bold: Field::Set(i % 2 == 0),
                    ..Default::default()
                },
                false,
                &Default::default(),
            )?;
        }
    }
    let base_positioned = b.stats().live_cells;
    eprintln!(
        "visible fixture: {base_positioned} positioned; {} format bytes",
        b.style_bytes()
    );
    let mut roots = Vec::new();
    let mut copies = Vec::new();
    let mut commits = Vec::new();
    let mut bytes = 0;
    for turn in 0..4 {
        roots.push(b.capture_sheet(sheet, b.expected())?);
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: (turn + 1).to_string(),
            }],
        )?;
        let t = Instant::now();
        let image = b.copy_visible_sheet(
            sheet,
            b.expected(),
            &source,
            source_column,
            source_column,
            &Default::default(),
        )?;
        copies.push(ms(t));
        bytes = image.native.len() + image.text.len();
        let column = source_column + 1 + turn % 2;
        eprintln!(
            "visible turn {turn}: column {column}; {} format bytes before paste",
            b.style_bytes()
        );
        let t = Instant::now();
        b.paste_visible_sheet(
            sheet,
            b.expected(),
            pivot_workbook::clipboard::VisibleDestination {
                rectangle: Rectangle {
                    first: Position::new(destination[0], column),
                    last: Position::new(*destination.last().unwrap(), column),
                },
                rows: &destination,
            },
            &image.native,
            true,
            &Default::default(),
        )?;
        commits.push(ms(t));
        for (i, (&sr, &dr)) in source.iter().zip(&destination).enumerate() {
            let cell = b.cell_at(Position::new(dr, column))?;
            if mode == "formulas" {
                assert_eq!(
                    b.display_formula(cell.unwrap().reference)?.unwrap(),
                    format!("={}{}", if column == 3 { "B" } else { "C" }, dr)
                );
            } else if sr % 4 == 1 {
                let Content::Decimal(value) = cell.unwrap().content else {
                    panic!("number")
                };
                assert_eq!(
                    value.value().to_string(),
                    if sr == 1 {
                        (turn + 1).to_string()
                    } else {
                        sr.to_string()
                    }
                );
            } else {
                assert!(cell.is_none_or(|cell| matches!(cell.content, Content::Blank)));
            }
            if mode == "styles" {
                assert_eq!(
                    b.style_at(sheet, Position::new(dr, column))?.bold,
                    i % 2 == 0
                );
            }
        }
    }
    rt.try_flush()?;
    let steady = memory.stats().charged;
    let peak = memory.stats().peak;
    println!(
        "{{\"mode\":\"{mode}\",\"slots\":256,\"source_last_row\":65536,\"destination_last_row\":65281,\"base_positioned\":{base_positioned},\"copy_ms\":{copies:?},\"commit_ms\":{commits:?},\"combined_bytes\":{bytes},\"steady_with_four_roots\":{steady},\"peak\":{peak}}}"
    );
    drop((roots, b));
    rt.finish().map_err(|_| "retained runtime")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if copies.iter().chain(&commits).any(|v| *v > 1000.0)
        || steady > 256 * 1024 * 1024
        || peak > 512 * 1024 * 1024
    {
        return Err("visible clipboard checkpoint exceeded unchanged objective".into());
    }
    Ok(())
}
