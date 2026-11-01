use pivot_types::{AllocationError, PivotDecimal};
fn main() {
    let input = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    for line in input.lines() {
        let f = line.split('\t').collect::<Vec<_>>();
        let value = (|| {
            let total: PivotDecimal = f[1].parse().unwrap();
            let n = f[2].parse().map_err(|_| AllocationError::InvalidArgument)?;
            let i = f[3].parse().map_err(|_| AllocationError::InvalidArgument)?;
            let p = f[4].parse().map_err(|_| AllocationError::InvalidArgument)?;
            total.allocate_equal(n, i, p)
        })();
        match value {
            Ok(v) => {
                let (lo, mid, hi, negative, scale) = v.parts();
                let c = u128::from(lo) | (u128::from(mid) << 32) | (u128::from(hi) << 64);
                println!("{}\tok\t{}\t{}\t{}", f[0], c, u8::from(negative), scale);
            }
            Err(e) => println!("{}\t{e:?}", f[0]),
        }
    }
}
