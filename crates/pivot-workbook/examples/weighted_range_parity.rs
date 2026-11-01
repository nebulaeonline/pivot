use pivot_types::PivotValue as V;
use pivot_workbook::*;
fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    for line in text.lines() {
        let f = line.split('\t').collect::<Vec<_>>();
        let weights = f[4].split(',').collect::<Vec<_>>();
        let mut b = Workbook::new("Oracle", Limits::default()).unwrap();
        let mut edits = weights
            .iter()
            .enumerate()
            .map(|(i, w)| Edit::SetEntry {
                position: Position::new(i as u32 + 1, 2),
                source: (*w).into(),
            })
            .collect::<Vec<_>>();
        edits.push(Edit::SetEntry {
            position: Position::new(1, 1),
            source: format!(
                "=ALLOCATEWEIGHTEDRANGE({},{},{},B1:B{})",
                f[1],
                f[2],
                f[3],
                weights.len()
            ),
        });
        b.apply(b.expected(), &edits).unwrap();
        let cap = b.capture_local(b.expected()).unwrap();
        let answer = cap
            .evaluate_values(&[cap.cell_at(Position::new(1, 1)).unwrap()])
            .unwrap()
            .values
            .remove(0)
            .1;
        match answer {
            Ok(V::Decimal(d)) => {
                let (lo, mid, hi, neg, scale) = d.parts();
                let c = u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64);
                println!("{}\tok\t{}\t{}\t{}", f[0], c, u8::from(neg), scale);
            }
            Err(CellError::Arithmetic(_)) => println!("{}\tOverflow", f[0]),
            Err(CellError::Typed(e)) => println!("{}\t{e:?}", f[0]),
            e => panic!("Unexpected result: {e:?}"),
        }
    }
}
