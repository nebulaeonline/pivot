use pivot_core::{
    model::integration::AllocationDomain,
    typed::{self, Error, ExecutionError, Op},
};
use pivot_types::{PivotValue as V, weighted};
use std::cell::Cell;
fn args(n: usize) -> Vec<V> {
    let mut a = vec![
        V::Decimal(10.into()),
        V::Decimal(1.into()),
        V::Decimal(2.into()),
    ];
    a.extend((0..n).map(|_| V::Decimal(1.into())));
    a
}
#[test]
fn direct_typed_calls_require_work_memory_and_honor_cancellation() {
    let op = Op::AllocateWeighted;
    let a = args(64);
    let cost = weighted::visit_charge(64);
    let memory = AllocationDomain::new(weighted::SCRATCH_BYTES);
    let mut visits = cost - 1;
    assert_eq!(
        typed::apply(&op, &a, &mut visits, &memory, &|| false),
        Err(ExecutionError::WorkBudget)
    );
    assert_eq!(memory.stats().peak, 0);
    let small = AllocationDomain::new(weighted::SCRATCH_BYTES - 1);
    visits = cost;
    assert_eq!(
        typed::apply(&op, &a, &mut visits, &small, &|| false),
        Err(ExecutionError::Admission)
    );
    assert_eq!(visits, 0);
    assert_eq!(small.stats().charged, 0);
    visits = cost;
    let calls = Cell::new(0);
    let result = typed::apply(&op, &a, &mut visits, &memory, &|| {
        calls.set(calls.get() + 1);
        calls.get() > 100
    });
    assert_eq!(result, Err(ExecutionError::Cancelled));
    assert_eq!(visits, 0);
    assert_eq!(memory.stats().charged, 0);
    assert_eq!(
        typed::apply(&op, &a, &mut { cost }, &memory, &|| false)
            .unwrap()
            .unwrap(),
        V::Decimal("0.16".parse().unwrap())
    );
    assert_eq!(memory.stats().charged, 0);
    assert_eq!(memory.stats().peak, weighted::SCRATCH_BYTES);
}
#[test]
fn excessive_arity_and_operand_domains_refuse_before_helper_admission() {
    let memory = AllocationDomain::new(0);
    for n in [0, 65, 128] {
        let mut visits = 0;
        assert_eq!(
            typed::apply(
                &Op::AllocateWeighted,
                &args(n),
                &mut visits,
                &memory,
                &|| panic!("invalid arity executed")
            ),
            Ok(Err(Error::InvalidArgument))
        );
    }
    for (at, value, error) in [
        (3, V::Blank, Error::TypeMismatch),
        (0, V::Boolean(false), Error::TypeMismatch),
        (2, V::String("2".into()), Error::TypeMismatch),
        (3, V::Decimal((-1).into()), Error::InvalidArgument),
    ] {
        let mut a = args(2);
        a[at] = value;
        assert_eq!(
            typed::apply(&Op::AllocateWeighted, &a, &mut 0, &memory, &|| panic!(
                "invalid operand executed"
            )),
            Ok(Err(error))
        );
    }
    assert_eq!(Op::from_code(20), Some(Op::AllocateWeighted));
    for n in 0..=100 {
        assert_eq!(Op::AllocateWeighted.valid_arity(n), (4..=67).contains(&n));
    }
    assert_eq!(memory.stats().peak, 0);
}

#[test]
fn resolved_tokens_and_scenario_adapters_enforce_the_same_operation_contract() {
    use pivot_core::{
        evaluation::InvocationRequest,
        model::{
            WorkingModel,
            transfer::{self, ResolvedToken as T},
        },
    };
    let mut model = WorkingModel::new("Weighted tokens").unwrap();
    let input = model.add_input("Amount", 10.into()).unwrap();
    let output = model.add_computed("Share", "0").unwrap();
    for n in [1, 64] {
        let mut tokens = vec![T::Node(input), T::Decimal(1.into()), T::Decimal(2.into())];
        tokens.extend((0..n).map(|_| T::Decimal(1.into())));
        tokens.push(T::Typed(Op::AllocateWeighted, n + 3));
        transfer::set_resolved_expression(&mut model, output, "weighted", &tokens).unwrap();
        let cap = model.capture_local().unwrap();
        let expected = if n == 1 { "10" } else { "0.16" };
        assert_eq!(
            cap.evaluate(&[output]).unwrap().values()[&output],
            Ok(expected.parse().unwrap())
        );
        let scenario = cap
            .invoke(&InvocationRequest {
                inputs: vec![(input, 20.into())],
                outputs: vec![output],
            })
            .unwrap();
        assert_eq!(
            scenario.values()[&output],
            Ok(if n == 1 { "20" } else { "0.32" }.parse().unwrap())
        );
    }
    let before = model.revision();
    for arity in [0, 3, 68, 4097] {
        let mut tokens = vec![T::Decimal(1.into()); arity];
        tokens.push(T::Typed(Op::AllocateWeighted, arity));
        assert!(
            transfer::set_resolved_expression(&mut model, output, "malformed weighted", &tokens)
                .is_err()
        );
        assert_eq!(model.revision(), before);
    }
}
