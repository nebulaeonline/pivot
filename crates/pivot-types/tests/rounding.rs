use pivot_types::{DecimalError, PivotDecimal as D, RoundingMode as M};
#[test]
fn signed_places_and_checked_edges() {
    for (source, places, mode, expected) in [
        ("2.5", 0, M::MidpointAwayFromZero, "3"),
        ("-2.5", 0, M::MidpointAwayFromZero, "-3"),
        ("-2.1", 0, M::AwayFromZero, "-3"),
        ("-2.9", 0, M::ToZero, "-2"),
        ("125", -1, M::MidpointAwayFromZero, "130"),
        ("-125", -1, M::MidpointAwayFromZero, "-130"),
        (
            "0.0000000000000000000000000001",
            27,
            M::AwayFromZero,
            "0.000000000000000000000000001",
        ),
        ("1.2300", 28, M::ToZero, "1.23"),
    ] {
        assert_eq!(
            source
                .parse::<D>()
                .unwrap()
                .round_signed_with_mode(places, mode)
                .unwrap(),
            expected.parse::<D>().unwrap()
        );
    }
    assert_eq!(
        D::MAX.round_signed_with_mode(-1, M::MidpointAwayFromZero),
        Err(DecimalError::Overflow)
    );
    assert_eq!(
        D::MIN.round_signed_with_mode(-1, M::AwayFromZero),
        Err(DecimalError::Overflow)
    );
    assert_eq!(
        "49999999999999999999999999999"
            .parse::<D>()
            .unwrap()
            .round_signed_with_mode(-29, M::MidpointAwayFromZero),
        Ok(D::ZERO)
    );
    assert_eq!(
        "50000000000000000000000000000"
            .parse::<D>()
            .unwrap()
            .round_signed_with_mode(-29, M::MidpointAwayFromZero),
        Err(DecimalError::Overflow)
    );
    assert_eq!(
        D::MAX.round_signed_with_mode(-29, M::MidpointAwayFromZero),
        Err(DecimalError::Overflow)
    );
    assert_eq!(
        D::MAX.round_signed_with_mode(-30, M::MidpointAwayFromZero),
        Ok(D::ZERO)
    );
    assert_eq!(
        D::ONE.round_signed_with_mode(i32::MIN, M::AwayFromZero),
        Err(DecimalError::Overflow)
    );
    assert_eq!(
        D::ZERO.round_signed_with_mode(i32::MIN, M::AwayFromZero),
        Ok(D::ZERO)
    );
    assert_eq!(
        D::MAX.round_signed_with_mode(i32::MAX, M::AwayFromZero),
        Ok(D::MAX)
    );
}
#[test]
fn independent_integer_reference_all_modes_and_signs() {
    let modes = [
        M::ToEven,
        M::MidpointAwayFromZero,
        M::MidpointTowardZero,
        M::ToZero,
        M::AwayFromZero,
        M::ToNegativeInfinity,
        M::ToPositiveInfinity,
    ];
    for n in -1500i64..=1500 {
        for mode in modes {
            for places in [-1, 0, 1] {
                let divisor = match places {
                    -1 => 1000,
                    0 => 100,
                    _ => 10,
                };
                let a = n.abs();
                let q = a / divisor;
                let r = a % divisor;
                let up = match mode {
                    M::ToEven => 2 * r > divisor || (2 * r == divisor && q % 2 == 1),
                    M::MidpointAwayFromZero => 2 * r >= divisor,
                    M::MidpointTowardZero => 2 * r > divisor,
                    M::ToZero => false,
                    M::AwayFromZero => r != 0,
                    M::ToNegativeInfinity => n < 0 && r != 0,
                    M::ToPositiveInfinity => n > 0 && r != 0,
                };
                let rounded = (q + i64::from(up)) * n.signum();
                let expected = match places {
                    -1 => D::from(rounded * 10),
                    0 => D::from(rounded),
                    _ => {
                        D::from_parts(rounded.unsigned_abs() as u32, 0, 0, rounded < 0, 1).unwrap()
                    }
                };
                let value = D::from_parts(n.unsigned_abs() as u32, 0, 0, n < 0, 2).unwrap();
                assert_eq!(
                    value.round_signed_with_mode(places, mode).unwrap(),
                    expected,
                    "{n} {places} {mode:?}"
                );
            }
        }
    }
}
