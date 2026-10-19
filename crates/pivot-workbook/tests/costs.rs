//! Measured deterministic proof costs, not a performance gate or scale benchmark.
use pivot_core::expression::BinaryOp;
use pivot_workbook::*;
use std::time::Instant;

#[test]
fn report_small_workload_costs() {
    println!("cells,build_us,single_edit_us,capture_us,evaluate_us,read_us,estimated_owned_bytes");
    for count in [16u32, 128, 512] {
        let mut w = Workbook::new("Costs", Limits::default()).unwrap();
        let value = DecimalInput::from_parts(1, 0, 0, false, 0).unwrap();
        let edits: Vec<_> = (1..=count)
            .map(|column| Edit::SetDecimal {
                position: Position::new(1, column),
                value,
            })
            .collect();
        let start = Instant::now();
        for batch in edits.chunks(256) {
            w.apply(w.expected(), batch).unwrap();
        }
        w.apply(
            w.expected(),
            &[Edit::SetFormula {
                position: Position::new(2, 1),
                source: "typed sum of two cells".into(),
                tokens: vec![
                    Token::At(Position::new(1, 1)),
                    Token::At(Position::new(1, 2)),
                    Token::Binary(BinaryOp::Add),
                ],
            }],
        )
        .unwrap();
        let build = start.elapsed().as_micros();
        let start = Instant::now();
        w.apply(
            w.expected(),
            &[Edit::SetDecimal {
                position: Position::new(1, 1),
                value: DecimalInput::from_parts(2, 0, 0, false, 0).unwrap(),
            }],
        )
        .unwrap();
        let single = start.elapsed().as_micros();
        let output = w.cell_at(Position::new(2, 1)).unwrap().unwrap().reference;
        let start = Instant::now();
        let captured = w.capture(w.expected()).unwrap();
        let capture = start.elapsed().as_micros();
        let start = Instant::now();
        let result = captured.evaluate(&[output]).unwrap();
        let evaluate = start.elapsed().as_micros();
        assert_eq!(result.values[0].1.clone().unwrap(), "3".parse().unwrap());
        let start = Instant::now();
        let read = w
            .read(
                w.expected(),
                Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(1, count),
                },
                1024,
            )
            .unwrap();
        let read_time = start.elapsed().as_micros();
        assert_eq!(read.cells.len(), count as usize);
        assert_eq!(w.stats().live_nodes, count as usize + 1);
        println!(
            "{},{build},{single},{capture},{evaluate},{read_time},{}",
            count + 1,
            w.stats().estimated_owned_bytes
        );
    }
}
