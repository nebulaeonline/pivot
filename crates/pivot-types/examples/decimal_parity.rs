use pivot_types::{PivotDecimal, RoundingMode};
use std::{env, fs};

fn main() {
    let native = env::args().any(|s| s == "--native");
    let input = fs::read_to_string(env::args().nth(1).expect("vector file")).unwrap();
    for line in input
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
    {
        let f: Vec<_> = line.split('\t').collect();
        let a: PivotDecimal = f[2].parse().unwrap();
        let result = match f[1] {
            "add" => a.checked_add(f[3].parse().unwrap()),
            "sub" => a.checked_sub(f[3].parse().unwrap()),
            "mul" => a.checked_mul(f[3].parse().unwrap()),
            "div" => a.checked_div(f[3].parse().unwrap()),
            "round" => {
                let mode = match f[4] {
                    "ToEven" => RoundingMode::ToEven,
                    "MidpointAwayFromZero" => RoundingMode::MidpointAwayFromZero,
                    "MidpointTowardZero" => RoundingMode::MidpointTowardZero,
                    "ToZero" => RoundingMode::ToZero,
                    "AwayFromZero" => RoundingMode::AwayFromZero,
                    "ToNegativeInfinity" => RoundingMode::ToNegativeInfinity,
                    "ToPositiveInfinity" => RoundingMode::ToPositiveInfinity,
                    _ => panic!("unknown rounding mode"),
                };
                a.round_with_mode(f[3].parse().unwrap(), mode)
            }
            "default" => a.round(f[3].parse().unwrap()),
            "normalize" => Ok(a.normalize()),
            _ => Ok(a),
        };
        if native && matches!(f[1], "add" | "sub" | "mul" | "div") {
            let a = rust_decimal::Decimal::from_str_exact(f[2]).unwrap();
            let b = rust_decimal::Decimal::from_str_exact(f[3]).unwrap();
            let raw = match f[1] {
                "add" => a.checked_add(b),
                "sub" => a.checked_sub(b),
                "mul" => a.checked_mul(b),
                _ => a.checked_div(b),
            };
            if let Some(value) = raw {
                println!("{}\tok\t{}\t{}", f[0], value, value.scale());
                continue;
            }
        }
        match result {
            Ok(value) => println!("{}\tok\t{}\t{}", f[0], value, value.scale()),
            Err(error) => println!("{}\t{:?}", f[0], error),
        }
    }
}
