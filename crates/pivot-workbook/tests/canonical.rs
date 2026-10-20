use pivot_core::model::{WorkingModel, integration::IntegrationRuntime};
use pivot_types::PivotDecimal;
use pivot_workbook::{Content, DecimalInput, Edit, Limits, Position, Token, Workbook};

#[test]
fn convenience_books_share_model_runtime_and_captures_own_their_generation() {
    let mut a = Workbook::new("a", Limits::default()).unwrap();
    let b = Workbook::new("b", Limits::default()).unwrap();
    let model = WorkingModel::new("model").unwrap();
    a.apply(
        a.expected(),
        &[Edit::SetDecimal {
            position: Position::new(1, 1),
            value: DecimalInput::from_decimal(7.into()),
        }],
    )
    .unwrap();
    let old = a.capture_local(a.expected()).unwrap();
    let cell = old.cell_at(Position::new(1, 1)).unwrap();
    assert!(IntegrationRuntime::shutdown_convenience().is_err());
    let memory = IntegrationRuntime::with_convenience(|r| r.memory()).unwrap();
    drop((a, b, model));
    assert_eq!(old.evaluate(&[cell]).unwrap().values[0].1, Ok(7.into()));
    assert!(IntegrationRuntime::shutdown_convenience().is_err());
    drop(old);
    IntegrationRuntime::with_convenience(|r| r.flush()).unwrap();
    IntegrationRuntime::shutdown_convenience().unwrap();
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn randomized_transactions_match_small_independent_scalar_oracle() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("oracle", Limits::default(), &runtime).unwrap();
    let mut inputs = [1i64; 32];
    // None means an initial zero output; Some(None) a self-cycle; Some(Some(k)) A+k.
    let mut outputs: [Option<Option<i64>>; 32] = [None; 32];
    let seed = (1..=32)
        .flat_map(|row| {
            [
                Edit::SetDecimal {
                    position: Position::new(row, 1),
                    value: DecimalInput::from_decimal(1.into()),
                },
                Edit::SetDecimal {
                    position: Position::new(row, 2),
                    value: DecimalInput::from_decimal(0.into()),
                },
            ]
        })
        .collect::<Vec<_>>();
    book.apply(book.expected(), &seed).unwrap();
    let identities = (1..=32)
        .map(|r| {
            [
                book.cell_at(Position::new(r, 1))
                    .unwrap()
                    .unwrap()
                    .reference,
                book.cell_at(Position::new(r, 2))
                    .unwrap()
                    .unwrap()
                    .reference,
            ]
        })
        .collect::<Vec<_>>();
    let mut random = 19u64;
    for step in 0..256u32 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let index = (random % 32) as usize;
        let row = index as u32 + 1;
        let input = Position::new(row, 1);
        let output = Position::new(row, 2);
        let before = book.expected();
        let stats = book.stats();
        let before_input = book.cell_at(input).unwrap().unwrap();
        let before_output = book.cell_at(output).unwrap().unwrap();
        let old = book.capture_local(before).unwrap();
        let first = if step % 5 == 0 {
            Edit::Clear(input)
        } else {
            Edit::SetDecimal {
                position: input,
                value: DecimalInput::from_parts((step + 1) * 100, 0, 0, false, 2).unwrap(),
            }
        };
        let cycle = step % 13 == 0;
        let source = if cycle {
            format!("=B{row}+1")
        } else {
            format!("=A{row}+{}", step % 9)
        };
        let second = if step % 17 == 0 {
            Edit::SetFormula {
                position: output,
                source: "malformed".into(),
                tokens: vec![Token::Binary(pivot_core::expression::BinaryOp::Add)],
            }
        } else {
            Edit::SetFormulaText {
                position: output,
                source: source.clone(),
            }
        };
        let result = book.apply(before, &[first, second]);
        if step % 17 == 0 {
            assert!(result.is_err());
            assert_eq!(book.expected(), before);
            assert_eq!(book.stats(), stats);
            assert_eq!(book.cell_at(input).unwrap().unwrap(), before_input);
            assert_eq!(book.cell_at(output).unwrap().unwrap(), before_output);
        } else {
            result.unwrap();
            inputs[index] = if step % 5 == 0 { 0 } else { step as i64 + 1 };
            outputs[index] = Some(if cycle { None } else { Some((step % 9) as i64) });
            assert_eq!(book.expected().state, before.state + 1);
            let view = book.cell_at(output).unwrap().unwrap();
            let Content::TextFormula(formula) = view.content else {
                panic!("text metadata lost")
            };
            assert_eq!(formula.original_source, source);
            assert_eq!(formula.transaction_base, before);
            assert_eq!(formula.edit_ordinal, 1);
            assert_eq!(formula.references.len(), 1);
            assert_eq!(
                formula.references[0].target,
                identities[index][usize::from(cycle)]
            );
            if step % 5 != 0 {
                let Content::Decimal(value) = book.cell_at(input).unwrap().unwrap().content else {
                    panic!("decimal metadata lost")
                };
                assert_eq!(value.parts(), ((step + 1) * 100, 0, 0, false, 2));
            }
        }
        assert_eq!(old.pinned_content(input).unwrap().unwrap(), before_input);
        assert_eq!(old.pinned_content(output).unwrap().unwrap(), before_output);
        let capture = book.capture_local(book.expected()).unwrap();
        for i in 0..32 {
            for (col, identity) in identities[i].iter().enumerate() {
                assert_eq!(
                    book.cell_at(Position::new(i as u32 + 1, col as u32 + 1))
                        .unwrap()
                        .unwrap()
                        .reference,
                    *identity
                );
            }
            let evaluated = capture.evaluate(&[identities[i][1]]);
            if outputs[i] == Some(None) {
                assert!(evaluated.is_err());
                continue;
            }
            let value = &evaluated.unwrap().values[0].1;
            match outputs[i] {
                None => assert_eq!(*value, Ok(PivotDecimal::ZERO)),
                Some(None) => assert!(value.is_err()),
                Some(Some(k)) => assert_eq!(*value, Ok((inputs[i] + k).into())),
            }
        }
        assert_eq!(
            book.validate(book.expected()).unwrap().is_empty(),
            !outputs.contains(&Some(None))
        );
    }
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
