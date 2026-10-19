//! Frozen v1.0 C# request -> current host -> frozen v1.0 C# response decoder.
use pivot_core::{interface::InputRequirement, model::WorkingModel};
use pivot_host::{AllowAll, Limits, ModelHost, Session};
use pivot_protocol::v1 as wire;
use prost::Message;
use std::{collections::BTreeMap, fs, sync::Arc};
fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let mut model = WorkingModel::new_reference("Legacy").unwrap();
    let revenue = model.add_input("Revenue", "1".parse().unwrap()).unwrap();
    let net = model.add_computed("Net", "Revenue - 1").unwrap();
    model
        .add_input_port("Revenue", revenue, InputRequirement::Required)
        .unwrap();
    model.add_output_port("Net", net).unwrap();
    let mut host = ModelHost::new("legacy_probe").unwrap();
    host.register(
        wire::ModelRef {
            model_id: "model_profit".into(),
            revision_id: "revision_1".into(),
        },
        model.snapshot().unwrap(),
        BTreeMap::from([
            ("Revenue".into(), "revenue".into()),
            ("Net".into(), "net".into()),
        ]),
    )
    .unwrap();
    let mut session = Session::new(Limits::default(), Arc::new(AllowAll)).unwrap();
    let hello = wire::Request {
        request_id: "hello".into(),
        version: Some(wire::Version { major: 1, minor: 0 }),
        operation: Some(wire::request::Operation::Handshake(wire::Handshake {
            supported_versions: vec![wire::Version { major: 1, minor: 0 }],
            ..Default::default()
        })),
    };
    host.dispatch(&mut session, &hello.encode_to_vec());
    let bytes = host.dispatch(
        &mut session,
        &fs::read(dir.join("csharp-request.bin")).unwrap(),
    );
    let response = wire::Response::decode(bytes.as_slice()).unwrap();
    assert!(matches!(
        response.outcome,
        Some(wire::response::Outcome::Invocation(_))
    ));
    fs::write(dir.join("new-host-response.bin"), bytes).unwrap();
}
