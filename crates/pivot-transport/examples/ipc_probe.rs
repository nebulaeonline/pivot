use pivot_protocol::{v1::*, validation};
use pivot_transport::{self as transport, tokio};
use prost::Message;
use tokio::io::AsyncWriteExt;

async fn exchange(stream: &mut transport::Stream, operation: request::Operation) -> Response {
    let request = Request {
        request_id: "probe".into(),
        version: Some(Version { major: 1, minor: 0 }),
        operation: Some(operation),
    };
    transport::write_frame(
        stream,
        &request.encode_to_vec(),
        validation::MAX_MESSAGE_BYTES,
    )
    .await
    .unwrap();
    let bytes = transport::read_frame(stream, validation::MAX_MESSAGE_BYTES)
        .await
        .unwrap()
        .unwrap();
    validation::decode_response(&bytes, &request).unwrap()
}
fn error(response: Response) -> ErrorCode {
    let Some(response::Outcome::Error(error)) = response.outcome else {
        panic!("{response:?}")
    };
    ErrorCode::try_from(error.code).unwrap()
}
async fn hello(endpoint: &str) -> transport::Stream {
    let mut stream = transport::connect(endpoint).await.unwrap();
    assert!(matches!(
        exchange(
            &mut stream,
            request::Operation::Handshake(Handshake {
                supported_versions: vec![Version { major: 1, minor: 0 }],
                ..Default::default()
            })
        )
        .await
        .outcome,
        Some(response::Outcome::Handshake(_))
    ));
    stream
}
#[tokio::main]
async fn main() {
    let endpoint = std::env::args().nth(1).expect("endpoint");
    let mut stream = transport::connect(&endpoint).await.unwrap();
    assert_eq!(
        error(
            exchange(
                &mut stream,
                request::Operation::ListModels(ListModels::default())
            )
            .await
        ),
        ErrorCode::HandshakeRequired
    );
    assert_eq!(
        error(
            exchange(
                &mut stream,
                request::Operation::Handshake(Handshake {
                    supported_versions: vec![Version {
                        major: 99,
                        minor: 0
                    }],
                    ..Default::default()
                })
            )
            .await
        ),
        ErrorCode::UnsupportedVersion
    );
    assert_eq!(
        error(
            exchange(
                &mut stream,
                request::Operation::Handshake(Handshake {
                    supported_versions: vec![Version { major: 1, minor: 0 }],
                    required_capabilities: vec!["unknown-required".into()],
                    optional_capabilities: vec![]
                })
            )
            .await
        ),
        ErrorCode::UnsupportedCapability
    );
    assert!(matches!(
        exchange(
            &mut stream,
            request::Operation::Handshake(Handshake {
                supported_versions: vec![Version { major: 1, minor: 0 }],
                ..Default::default()
            })
        )
        .await
        .outcome,
        Some(response::Outcome::Handshake(_))
    ));
    for _ in 0..3 {
        assert!(matches!(
            exchange(
                &mut stream,
                request::Operation::ListModels(ListModels::default())
            )
            .await
            .outcome,
            Some(response::Outcome::Models(_))
        ));
    }
    drop(stream);
    // Peer survives a malformed payload without terminating the daemon.
    let mut malformed = transport::connect(&endpoint).await.unwrap();
    transport::write_frame(&mut malformed, &[0xff], validation::MAX_MESSAGE_BYTES)
        .await
        .unwrap();
    let response = Response::decode(
        transport::read_frame(&mut malformed, validation::MAX_MESSAGE_BYTES)
            .await
            .unwrap()
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    assert_eq!(error(response), ErrorCode::InvalidArgument);
    drop(malformed);
    for bytes in [u32::MAX.to_le_bytes().to_vec(), 0u32.to_le_bytes().to_vec()] {
        let mut bad = transport::connect(&endpoint).await.unwrap();
        bad.write_all(&bytes).await.unwrap();
        let closed = transport::read_frame(&mut bad, validation::MAX_MESSAGE_BYTES).await;
        assert!(matches!(closed, Ok(None) | Err(_)));
    }
    for bytes in [vec![1, 2], vec![4, 0, 0, 0, 1]] {
        let mut bad = transport::connect(&endpoint).await.unwrap();
        bad.write_all(&bytes).await.unwrap();
        drop(bad);
    }
    // Disconnect immediately after a complete request, before reading its result.
    let mut abandoned = hello(&endpoint).await;
    let request = Request {
        request_id: "abandoned".into(),
        version: Some(Version { major: 1, minor: 0 }),
        operation: Some(request::Operation::ListModels(ListModels::default())),
    };
    transport::write_frame(
        &mut abandoned,
        &request.encode_to_vec(),
        validation::MAX_MESSAGE_BYTES,
    )
    .await
    .unwrap();
    drop(abandoned);
    let mut valid = hello(&endpoint).await;
    assert!(matches!(
        exchange(
            &mut valid,
            request::Operation::ListModels(ListModels::default())
        )
        .await
        .outcome,
        Some(response::Outcome::Models(_))
    ));
    // A stalled partial header cannot occupy a connection indefinitely.
    let mut slow = transport::connect(&endpoint).await.unwrap();
    slow.write_all(&[1]).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(5250)).await;
    assert!(matches!(
        transport::read_frame(&mut slow, validation::MAX_MESSAGE_BYTES).await,
        Ok(None) | Err(_)
    ));
    println!("IPC framing/session/survival probe: PASS");
}
