use pivot_core::{
    model::integration::AllocationDomain,
    typed::{self, Op},
};
use pivot_types::PivotValue;
fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let memory = AllocationDomain::new(64 * 1024);
    for line in text.lines() {
        let f = line.split('\t').collect::<Vec<_>>();
        let mut args = vec![f[1], f[2], f[3]];
        if !f[4].is_empty() {
            args.extend(f[4].split(','));
        }
        let args = args
            .into_iter()
            .map(|s| PivotValue::Decimal(s.parse().unwrap()))
            .collect::<Vec<_>>();
        let answer = typed::apply(
            &Op::AllocateWeighted,
            &args,
            &mut 2_000_000,
            &memory,
            &|| false,
        )
        .unwrap();
        match answer {
            Ok(PivotValue::Decimal(d)) => {
                let (lo, mid, hi, negative, scale) = d.parts();
                let c = u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64);
                println!("{}\tok\t{}\t{}\t{}", f[0], c, u8::from(negative), scale);
            }
            Err(typed::Error::Arithmetic(_)) => println!("{}\tOverflow", f[0]),
            Err(e) => println!("{}\t{e:?}", f[0]),
            _ => panic!("weighted result must be Decimal"),
        }
        assert_eq!(memory.stats().charged, 0);
    }
}
