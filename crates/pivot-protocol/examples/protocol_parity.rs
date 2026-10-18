use pivot_protocol::{v1::*, validation::*};
use prost::Message;
use std::{env, fs, path::Path};

fn main() {
    let args: Vec<_> = env::args().collect();
    let dir = Path::new(&args[2]);
    fs::create_dir_all(dir).unwrap();
    if args[1] == "write" {
        let request = Request {
            request_id: "request_fixture".into(),
            version: Some(Version { major: 1, minor: 0 }),
            operation: Some(request::Operation::InvokeModel(InvokeModel {
                model: Some(ModelRef {
                    model_id: "model_profit".into(),
                    revision_id: "revision_1".into(),
                }),
                inputs: vec![InputBinding {
                    port_id: "revenue".into(),
                    value: Some(Value {
                        kind: Some(value::Kind::Decimal(Decimal128 {
                            lo: u32::MAX,
                            mid: u32::MAX,
                            hi: u32::MAX,
                            scale: 28,
                            negative: true,
                        })),
                    }),
                }],
                output_port_ids: vec!["net".into()],
            })),
        };
        validate_request(&request).unwrap();
        fs::write(dir.join("rust-request.bin"), request.encode_to_vec()).unwrap();
        for scale in 0..=28 {
            for negative in [false, true] {
                let value = Decimal128 {
                    lo: u32::MAX,
                    mid: 0x80000000,
                    hi: u32::MAX,
                    scale,
                    negative,
                };
                fs::write(
                    dir.join(format!("rust-decimal-{scale}-{negative}.bin")),
                    value.encode_to_vec(),
                )
                .unwrap();
            }
        }
    } else {
        let request = decode_request(&fs::read(dir.join("csharp-request.bin")).unwrap()).unwrap();
        let request::Operation::InvokeModel(call) = request.operation.unwrap() else {
            panic!()
        };
        let value::Kind::Decimal(value) = call.inputs[0]
            .value
            .as_ref()
            .unwrap()
            .kind
            .as_ref()
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(
            (value.lo, value.mid, value.hi, value.scale, value.negative),
            (12345, 0, 0, 2, true)
        );
        let result =
            InvocationResult::decode(fs::read(dir.join("csharp-result.bin")).unwrap().as_slice())
                .unwrap();
        assert_eq!(result.outputs.len(), 2);
        assert!(
            matches!(&result.outputs[1].outcome, Some(output_result::Outcome::Error(e)) if e.code == CalculationErrorCode::DivisionByZero as i32)
        );
        let original = fs::read(dir.join("rust-request.bin")).unwrap();
        assert_eq!(
            Request::decode(original.as_slice()).unwrap(),
            Request::decode(
                fs::read(dir.join("rust-request-roundtrip.bin"))
                    .unwrap()
                    .as_slice()
            )
            .unwrap()
        );
    }
    println!("Rust protocol fixtures: PASS");
}
