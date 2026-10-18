//! Test peer: accept exactly one session, then disconnect after receiving Invoke.
//! It never evaluates; tests verify that the client does not replay that request.
use pivot_protocol::{v1::*, validation};
use pivot_transport::{self as transport, tokio};
use prost::Message;
use std::{
    fs,
    io::{self, Write},
    time::Duration,
};

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut listener = transport::Listener::bind(&args[1]).unwrap();
    println!("READY");
    io::stdout().flush().unwrap();
    let mut stream = listener.accept().await.unwrap();
    let model = ModelRef {
        model_id: "restaurant-profit".into(),
        revision_id: "demo-v1".into(),
    };
    let description = ModelDescription {
        model: Some(model.clone()),
        name: "RestaurantProfit".into(),
        inputs: vec![InputPort {
            port_id: "revenue".into(),
            name: "Revenue".into(),
            value_type: ValueType::Decimal as i32,
            requirement: Some(input_port::Requirement::Required(true)),
        }],
        outputs: vec![OutputPort {
            port_id: "net".into(),
            name: "Net".into(),
            value_type: ValueType::Decimal as i32,
        }],
    };
    let mut negotiated = false;
    loop {
        let bytes = transport::read_frame(&mut stream, validation::MAX_MESSAGE_BYTES)
            .await
            .unwrap()
            .expect("client disconnected before invocation");
        let request = validation::decode_request(&bytes).unwrap();
        let outcome = match request.operation.unwrap() {
            request::Operation::Handshake(hello) => {
                assert!(!negotiated);
                negotiated = true;
                response::Outcome::Handshake(validation::negotiate(&hello).unwrap())
            }
            request::Operation::ListModels(_) => {
                assert!(negotiated);
                response::Outcome::Models(ModelList {
                    models: vec![ModelSummary {
                        model: Some(model.clone()),
                        name: "RestaurantProfit".into(),
                    }],
                    continuation_token: String::new(),
                })
            }
            request::Operation::DescribeModel(_) => {
                response::Outcome::Description(description.clone())
            }
            request::Operation::InvokeModel(_) => {
                fs::write(&args[2], "invocations=1\n").unwrap();
                break;
            }
            _ => panic!("unexpected operation"),
        };
        let response = Response {
            request_id: request.request_id,
            version: Some(Version { major: 1, minor: 0 }),
            outcome: Some(outcome),
        };
        transport::write_frame(
            &mut stream,
            &response.encode_to_vec(),
            validation::MAX_MESSAGE_BYTES,
        )
        .await
        .unwrap();
    }
    drop(stream);
    // A broken client that reconnects/retries is observed rather than rewarded.
    if tokio::time::timeout(Duration::from_secs(2), listener.accept())
        .await
        .is_ok()
    {
        fs::write(&args[2], "unexpected automatic reconnect\n").unwrap();
        panic!("automatic reconnect after ambiguous invocation");
    }
}
