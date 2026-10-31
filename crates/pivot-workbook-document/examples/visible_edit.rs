//! Declared bounded visible-edit corpora; core timings, not OS/dialog latency.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    csv,
    formatting::{Field, Style, StylePatch},
    visible_edit::ClearMode,
    *,
};
use std::{error::Error, time::Instant};
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or("clear-both".into());
    let clear = mode.starts_with("clear-");
    let direction = match mode.as_str() {
        "fill-down" => FillDirection::Down,
        "fill-up" => FillDirection::Up,
        "fill-left" => FillDirection::Left,
        "fill-right" => FillDirection::Right,
        "clear-contents" | "clear-format" | "clear-both" => FillDirection::Down,
        _ => return Err("unknown visible edit corpus".into()),
    };
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let text = (1..=65536)
        .map(|r| {
            if r % 4 == 1 {
                format!("{r}\n")
            } else {
                "\n".into()
            }
        })
        .collect::<String>();
    let mut b = csv::parse(text.as_bytes(), csv::Policy::default(), Limits::capacity())?.workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        Limits::capacity(),
        &rt,
    )?;
    let sheet = b.worksheet();
    let rows = if clear {
        (0..4096).map(|i| 1 + i * 16).collect::<Vec<_>>()
    } else {
        (0..128).map(|i| 1 + i * 516).collect::<Vec<_>>()
    };
    let columns = if clear { (2, 2) } else { (2, 3) };
    let mut roots = Vec::new();
    let mut commits = Vec::new();
    let mut unchanged = Vec::new();
    for turn in 0..4 {
        // Setup is outside the measured operation; every sample causes real work.
        let edits = rows
            .iter()
            .flat_map(|&r| {
                (columns.0..=columns.1).map(move |c| Edit::SetFormulaText {
                    position: p(r, c),
                    source: format!("=A{r}+{turn}"),
                })
            })
            .collect::<Vec<_>>();
        for batch in edits.chunks(256) {
            b.apply(b.expected(), batch)?;
        }
        b.format_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(1, 2),
                last: p(65536, 3),
            },
            StylePatch {
                bold: Field::Set(true),
                background: Field::Set(Some(0xabcdef)),
                ..Default::default()
            },
            false,
            &Default::default(),
        )?;
        // Alternate sources to ensure every Fill changes definitions, using actual coordinates.
        if !clear {
            let source_row = if direction == FillDirection::Up {
                *rows.last().unwrap()
            } else {
                rows[0]
            };
            b.apply(
                b.expected(),
                &[Edit::SetEntry {
                    position: p(
                        source_row,
                        if direction == FillDirection::Left {
                            3
                        } else {
                            2
                        },
                    ),
                    source: format!("'00{turn}"),
                }],
            )?;
            if matches!(direction, FillDirection::Left | FillDirection::Right) {
                for batch in rows.chunks(256) {
                    let edits = batch
                        .iter()
                        .map(|&r| Edit::SetEntry {
                            position: p(
                                r,
                                if direction == FillDirection::Left {
                                    3
                                } else {
                                    2
                                },
                            ),
                            source: format!("'00{turn}"),
                        })
                        .collect::<Vec<_>>();
                    b.apply(b.expected(), &edits)?;
                }
            }
        }
        roots.push(b.capture_sheet(sheet, b.expected())?);
        let before = b.expected();
        let t = Instant::now();
        if clear {
            let kind = match mode.as_str() {
                "clear-contents" => ClearMode::Contents,
                "clear-format" => ClearMode::Formatting,
                _ => ClearMode::Both,
            };
            b.clear_visible_sheet(sheet, before, &rows, columns, kind, &Default::default())?;
        } else {
            b.fill_visible_sheet(
                sheet,
                before,
                &rows,
                columns,
                direction,
                &Default::default(),
            )?;
        }
        commits.push(ms(t));
        assert_ne!(b.expected(), before);
        for &r in &rows {
            if clear {
                if mode != "clear-format" {
                    assert_eq!(b.cell_at(p(r, 2))?.unwrap().content, Content::Blank);
                } else {
                    assert_eq!(b.expected().graph, before.graph);
                }
                if mode != "clear-contents" {
                    assert_eq!(b.style_at(sheet, p(r, 2))?, Style::default());
                }
            } else {
                let is_source = match direction {
                    FillDirection::Down => r == rows[0],
                    FillDirection::Up => r == *rows.last().unwrap(),
                    _ => false,
                };
                if !is_source {
                    let c = if direction == FillDirection::Left {
                        2
                    } else if direction == FillDirection::Right {
                        3
                    } else {
                        2
                    };
                    assert_eq!(
                        b.cell_at(p(r, c))?.unwrap().content,
                        Content::LiteralText(format!("00{turn}"))
                    );
                }
            }
        }
        // Hidden row 2 and source-populated column A stay exact throughout.
        assert!(b.cell_at(p(2, 2))?.is_none());
        let same = b.expected();
        let t = Instant::now();
        if clear {
            b.clear_visible_sheet(
                sheet,
                same,
                &rows,
                columns,
                match mode.as_str() {
                    "clear-contents" => ClearMode::Contents,
                    "clear-format" => ClearMode::Formatting,
                    _ => ClearMode::Both,
                },
                &Default::default(),
            )?;
        } else {
            b.fill_visible_sheet(sheet, same, &rows, columns, direction, &Default::default())?;
        }
        unchanged.push(ms(t));
        assert_eq!(b.expected(), same);
    }
    let base_positioned = b.stats().live_cells;
    let steady = memory.stats().charged;
    let peak = memory.stats().peak;
    let slots = if clear { 4096 } else { 256 };
    println!(
        "{{\"mode\":\"{mode}\",\"slots\":{slots},\"positioned\":{base_positioned},\"retained_roots\":4,\"commit_ms\":{commits:?},\"noop_ms\":{unchanged:?},\"steady_with_four_roots\":{steady},\"peak\":{peak}}}"
    );
    assert!(commits.iter().chain(&unchanged).all(|&v| v <= 1000.0));
    assert!(steady <= 256 * 1024 * 1024 && peak <= 512 * 1024 * 1024);
    drop(roots);
    drop(b);
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}
