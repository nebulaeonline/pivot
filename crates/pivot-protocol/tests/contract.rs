use pivot_protocol::{v1::*, validation::*};
use prost::Message;

fn value(lo: u32, scale: u32) -> Value {
    Value {
        kind: Some(value::Kind::Decimal(Decimal128 {
            lo,
            scale,
            ..Default::default()
        })),
    }
}
fn model() -> ModelRef {
    ModelRef {
        model_id: "model_profit".into(),
        revision_id: "revision_1".into(),
    }
}
fn description() -> ModelDescription {
    ModelDescription {
        model: Some(model()),
        name: "Profit".into(),
        inputs: vec![
            InputPort {
                port_id: "revenue".into(),
                name: "Revenue".into(),
                value_type: ValueType::Decimal as i32,
                requirement: Some(input_port::Requirement::Required(true)),
            },
            InputPort {
                port_id: "cost".into(),
                name: "Costs".into(),
                value_type: ValueType::Decimal as i32,
                requirement: Some(input_port::Requirement::DefaultValue(value(25, 0))),
            },
        ],
        outputs: vec![OutputPort {
            port_id: "net".into(),
            name: "Net".into(),
            value_type: ValueType::Decimal as i32,
        }],
    }
}
fn invocation() -> InvokeModel {
    InvokeModel {
        model: Some(model()),
        inputs: vec![InputBinding {
            source: 0,
            port_id: "revenue".into(),
            value: Some(value(100, 0)),
        }],
        output_port_ids: vec!["net".into()],
    }
}
fn request(invocation: InvokeModel) -> Request {
    Request {
        request_id: "req_1".into(),
        version: Some(Version { major: 1, minor: 0 }),
        operation: Some(request::Operation::InvokeModel(invocation)),
    }
}

#[test]
fn decimal_representation_roundtrips_without_float_conversion() {
    for scale in 0..=28 {
        for negative in [false, true] {
            for (lo, mid, hi) in [(0, 0, 0), (1, 0, 0), (u32::MAX, u32::MAX, u32::MAX)] {
                let original = Decimal128 {
                    lo,
                    mid,
                    hi,
                    scale,
                    negative,
                };
                validate_decimal(&original).unwrap();
                assert_eq!(
                    Decimal128::decode(original.encode_to_vec().as_slice()).unwrap(),
                    original
                );
            }
        }
    }
    assert!(
        validate_decimal(&Decimal128 {
            scale: 29,
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        validate_decimal(&Decimal128 {
            scale: u32::MAX,
            ..Default::default()
        })
        .is_err()
    );
}
#[test]
fn negotiation_and_untrusted_messages() {
    let hello = Handshake {
        supported_versions: vec![Version { major: 1, minor: 0 }],
        required_capabilities: vec!["decimal96".into()],
        optional_capabilities: vec!["future".into()],
    };
    assert_eq!(
        negotiate(&hello).unwrap().selected_version,
        Some(Version { major: 1, minor: 0 })
    );
    let mut bad = hello.clone();
    bad.required_capabilities.push("future".into());
    assert_eq!(
        negotiate(&bad).unwrap_err().code,
        ErrorCode::UnsupportedCapability
    );
    bad = hello;
    bad.supported_versions[0].major = 2;
    assert_eq!(
        negotiate(&bad).unwrap_err().code,
        ErrorCode::UnsupportedVersion
    );
    let request = request(invocation());
    assert_eq!(decode_request(&request.encode_to_vec()).unwrap(), request);
    assert!(decode_request(&[0xff]).is_err());
    assert_eq!(
        decode_request(&vec![0; MAX_MESSAGE_BYTES + 1])
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    let mut bad = request;
    bad.version = None;
    assert_eq!(
        validate_request(&bad).unwrap_err().code,
        ErrorCode::UnsupportedVersion
    );
}
#[test]
fn interface_bindings_and_required_defaults() {
    let description = description();
    let mut call = invocation();
    validate_invocation(&call, &description).unwrap();
    call.inputs.clear();
    assert_eq!(
        validate_invocation(&call, &description).unwrap_err().code,
        ErrorCode::MissingInput
    );
    call = invocation();
    call.inputs.push(call.inputs[0].clone());
    assert_eq!(
        validate_invocation(&call, &description).unwrap_err().code,
        ErrorCode::DuplicateBinding
    );
    call = invocation();
    call.inputs[0].port_id = "internal_node".into();
    assert_eq!(
        validate_invocation(&call, &description).unwrap_err().code,
        ErrorCode::UnknownPort
    );
    call = invocation();
    call.output_port_ids = vec!["missing".into()];
    assert_eq!(
        validate_invocation(&call, &description).unwrap_err().code,
        ErrorCode::UnknownPort
    );
    call = invocation();
    call.model.as_mut().unwrap().revision_id = "other_revision".into();
    assert_eq!(
        validate_invocation(&call, &description).unwrap_err().code,
        ErrorCode::RevisionUnavailable
    );
    let mut bad = description.clone();
    bad.inputs[0].requirement = Some(input_port::Requirement::Required(false));
    assert!(validate_description(&bad).is_err());
    bad = description.clone();
    bad.inputs[1].name = bad.inputs[0].name.clone();
    assert!(validate_description(&bad).is_err());
    bad = description;
    bad.inputs[0].value_type = 999;
    assert_eq!(
        validate_description(&bad).unwrap_err().code,
        ErrorCode::UnsupportedValueType
    );
}
#[test]
fn future_values_fail_closed_and_unknown_metadata_is_tolerated() {
    // Future oneof tag 3: unknown protobuf payload is not silently a decimal zero.
    let future = Value::decode(&[0x1a, 0x00][..]).unwrap();
    assert_eq!(
        validate_value(&future).unwrap_err().code,
        ErrorCode::UnsupportedValueType
    );
    assert_eq!(
        validate_value(&Value {
            kind: Some(value::Kind::Float64(1.5))
        })
        .unwrap_err()
        .code,
        ErrorCode::UnsupportedValueType
    );
    let request = request(invocation());
    let mut bytes = request.encode_to_vec();
    bytes.extend([0xa0, 0x06, 0x01]); // Unknown field 100, varint 1.
    assert_eq!(decode_request(&bytes).unwrap(), request);
}
#[test]
fn successful_failed_and_malformed_results() {
    let call = invocation();
    let description = description();
    let mut result = InvocationResult {
        invocation_id: "invocation_1".into(),
        model: Some(model()),
        effective_inputs: vec![
            call.inputs[0].clone(),
            InputBinding {
                source: 0,
                port_id: "cost".into(),
                value: Some(value(25, 0)),
            },
        ],
        outputs: vec![OutputResult {
            port_id: "net".into(),
            outcome: Some(output_result::Outcome::Value(value(75, 0))),
        }],
    };
    for input in &mut result.effective_inputs {
        input.source = if input.port_id == "revenue" {
            InputSource::Submitted as i32
        } else {
            InputSource::Defaulted as i32
        };
    }
    validate_result(&result, &call, &description).unwrap();
    assert_eq!(
        InvocationResult::decode(result.encode_to_vec().as_slice()).unwrap(),
        result
    );
    result.outputs[0].outcome = Some(output_result::Outcome::Error(CalculationError {
        code: CalculationErrorCode::DivisionByZero as i32,
        message: "division by zero".into(),
    }));
    validate_result(&result, &call, &description).unwrap();
    let response = Response {
        request_id: "req_1".into(),
        version: Some(Version { major: 1, minor: 0 }),
        outcome: Some(response::Outcome::Invocation(result.clone())),
    };
    assert_eq!(
        Response::decode(response.encode_to_vec().as_slice()).unwrap(),
        response
    );
    let failure = Response {
        outcome: Some(response::Outcome::Error(ProtocolError {
            code: ErrorCode::MissingInput as i32,
            message: "required port".into(),
            port_id: "revenue".into(),
        })),
        ..response
    };
    let original_request = request(call.clone());
    assert!(decode_response(&failure.encode_to_vec(), &original_request).is_ok());
    assert_eq!(
        Response::decode(failure.encode_to_vec().as_slice()).unwrap(),
        failure
    );
    result.effective_inputs[1].value = Some(value(999, 0));
    assert!(validate_result(&result, &call, &description).is_err());
}

#[test]
fn response_correlation_and_payload_validation() {
    let call = request(invocation());
    let mut response = Response {
        request_id: call.request_id.clone(),
        version: call.version,
        outcome: Some(response::Outcome::Invocation(InvocationResult {
            invocation_id: "inv_1".into(),
            model: Some(model()),
            effective_inputs: vec![],
            outputs: vec![OutputResult {
                port_id: "net".into(),
                outcome: Some(output_result::Outcome::Value(value(75, 0))),
            }],
        })),
    };
    decode_response(&response.encode_to_vec(), &call).unwrap();
    response.request_id = "other_request".into();
    assert!(decode_response(&response.encode_to_vec(), &call).is_err());
    response.request_id = call.request_id.clone();
    response.outcome = Some(response::Outcome::Models(ModelList::default()));
    assert!(decode_response(&response.encode_to_vec(), &call).is_err());
}

#[test]
fn numerical_equality_is_not_representation_equality() {
    assert_ne!(value(10, 1), value(100, 2));
    assert!(numeric_equal(&value(10, 1), &value(100, 2)).unwrap());
    let mut zero = value(0, 28);
    let Some(value::Kind::Decimal(d)) = &mut zero.kind else {
        panic!()
    };
    d.negative = true;
    assert!(numeric_equal(&zero, &value(0, 0)).unwrap());
    assert!(!numeric_equal(&value(11, 1), &value(1, 0)).unwrap());
    assert!(numeric_equal(&value(1, 29), &value(1, 0)).is_err());
}

#[test]
fn provenance_accepts_equivalent_values_but_rejects_false_source() {
    let call = invocation();
    let description = description();
    let mut result = InvocationResult {
        invocation_id: "inv_1".into(),
        model: Some(model()),
        effective_inputs: vec![
            InputBinding {
                port_id: "revenue".into(),
                value: Some(value(10000, 2)),
                source: InputSource::Submitted as i32,
            },
            InputBinding {
                port_id: "cost".into(),
                value: Some(value(2500, 2)),
                source: InputSource::Defaulted as i32,
            },
        ],
        outputs: vec![OutputResult {
            port_id: "net".into(),
            outcome: Some(output_result::Outcome::Value(value(75, 0))),
        }],
    };
    validate_result(&result, &call, &description).unwrap();
    result.effective_inputs[0].source = InputSource::Defaulted as i32;
    assert!(validate_result(&result, &call, &description).is_err());
}
