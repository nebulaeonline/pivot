use pivot_protocol::{authoring::v1 as a, foundation::*, v1};
use prost::Message;
const ID: &str = "01234567-89ab-4cde-8fab-0123456789ab";
fn revision() -> a::WorkingRevision {
    a::WorkingRevision {
        service_instance_id: ID.into(),
        working_model_id: ID.into(),
        sequence: 0,
    }
}
fn create() -> a::AuthoringRequest {
    a::AuthoringRequest {
        operation: Some(a::authoring_request::Operation::CreateWorking(
            a::CreateWorking {
                service_instance_id: ID.into(),
                operation_id: "op".into(),
                name: "Model".into(),
            },
        )),
    }
}
fn wrap(bytes: Vec<u8>) -> Vec<u8> {
    assert!(bytes.len() < 128);
    let mut out = vec![0x12, bytes.len() as u8];
    out.extend(bytes);
    out
}
#[test]
fn strict_unknown_duplicate_singular_and_conflicting_oneofs_are_rejected() {
    let mut bytes = create().encode_to_vec();
    bytes.extend([0xf8, 7, 1]);
    assert!(decode_authoring(&bytes, Limits::default()).is_err());
    let mut c = a::CreateWorking {
        service_instance_id: ID.into(),
        operation_id: "op".into(),
        name: "M".into(),
    }
    .encode_to_vec();
    c.extend([0x1a, 1, b'X']);
    assert!(decode_authoring(&wrap(c), Limits::default()).is_err());
    let mut b = create().encode_to_vec();
    b.extend([0x0a, 0]);
    assert!(decode_authoring(&b, Limits::default()).is_err());
    assert!(decode_authoring(&[0x12, 0xff], Limits::default()).is_err());
    assert!(decode_authoring(&[0x12, 1, 0xff], Limits::default()).is_err());
}
#[test]
fn nested_unknown_duplicate_and_wrong_wire_types_are_rejected() {
    let mut decimal = v1::Decimal128::default().encode_to_vec();
    decimal.extend([0x48, 1]);
    let mut value = vec![0x0a, decimal.len() as u8];
    value.extend(decimal);
    // Isolated message validation uses the same recursive boundary as mutations.
    let mut definition = vec![0x0a, value.len() as u8];
    definition.extend(value);
    assert!(
        validate_message(
            ".pivot.authoring.v1.NodeDefinition",
            &definition,
            Limits::default()
        )
        .is_err()
    );
    assert!(
        validate_message(
            ".pivot.authoring.v1.NodeDefinition",
            &[0x10, 1],
            Limits::default()
        )
        .is_err()
    );
    assert!(
        validate_message(
            ".pivot.authoring.v1.NodeDefinition",
            &[0x12, 1, 0xff],
            Limits::default()
        )
        .is_err()
    );
}
#[test]
fn field_order_and_permitted_overlong_varints_preserve_typed_intent() {
    let mut c = vec![0x1a, 1, b'M', 0x12, 2, b'o', b'p', 0x0a, 36];
    c.extend(ID.as_bytes());
    let reordered = wrap(c);
    let canonical = a::AuthoringRequest {
        operation: Some(a::authoring_request::Operation::CreateWorking(
            a::CreateWorking {
                service_instance_id: ID.into(),
                operation_id: "op".into(),
                name: "M".into(),
            },
        )),
    };
    assert_eq!(
        mutation_intent(&decode_authoring(&reordered, Limits::default()).unwrap()),
        mutation_intent(&canonical)
    );
    // A zero sequence encoded explicitly with an overlong varint is valid.
    let mut bytes = revision().encode_to_vec();
    bytes.extend([0x18, 0x80, 0]);
    validate_message(
        ".pivot.authoring.v1.WorkingRevision",
        &bytes,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        a::WorkingRevision::decode(bytes.as_slice()).unwrap(),
        revision()
    );
}
#[test]
fn empty_formula_and_initial_revision_are_present_valid_values() {
    validate_message(
        ".pivot.authoring.v1.NodeDefinition",
        &[0x12, 0],
        Limits::default(),
    )
    .unwrap();
    assert!(
        validate_message(".pivot.authoring.v1.NodeDefinition", &[], Limits::default()).is_err()
    );
    validate_message(
        ".pivot.authoring.v1.WorkingRevision",
        &revision().encode_to_vec(),
        Limits::default(),
    )
    .unwrap();
    let r = a::WorkingRevision {
        sequence: u64::MAX,
        ..revision()
    };
    validate_message(
        ".pivot.authoring.v1.WorkingRevision",
        &r.encode_to_vec(),
        Limits::default(),
    )
    .unwrap();
    let location = a::Location {
        expression_byte_offset: Some(0),
        ..Default::default()
    };
    assert_eq!(
        a::Location::decode(location.encode_to_vec().as_slice())
            .unwrap()
            .expression_byte_offset,
        Some(0)
    );
}
#[test]
fn partial_port_patches_distinguish_absent_from_replacement() {
    let patch = a::SetInputPort {
        port: Some(a::PortReference {
            kind: Some(a::port_reference::Kind::PortId(ID.into())),
        }),
        node: None,
        requirement: Some(a::Requirement {
            kind: Some(a::requirement::Kind::Required(a::Required {})),
        }),
    };
    validate_message(
        ".pivot.authoring.v1.SetInputPort",
        &patch.encode_to_vec(),
        Limits::default(),
    )
    .unwrap();
    let empty = a::SetInputPort {
        requirement: None,
        ..patch.clone()
    };
    assert!(
        validate_message(
            ".pivot.authoring.v1.SetInputPort",
            &empty.encode_to_vec(),
            Limits::default()
        )
        .is_err()
    );
    let target = a::SetInputPort {
        node: Some(a::NodeReference {
            kind: Some(a::node_reference::Kind::NodeId(ID.into())),
        }),
        requirement: None,
        ..patch
    };
    validate_message(
        ".pivot.authoring.v1.SetInputPort",
        &target.encode_to_vec(),
        Limits::default(),
    )
    .unwrap();
}
#[test]
fn transport_budgets_are_configurable_and_model_limits_are_not_protocol_constants() {
    let bytes = create().encode_to_vec();
    assert!(
        decode_authoring(
            &bytes,
            Limits {
                bytes: 2,
                ..Limits::default()
            }
        )
        .is_err()
    );
    assert!(
        decode_authoring(
            &bytes,
            Limits {
                depth: 0,
                ..Limits::default()
            }
        )
        .is_err()
    );
    assert!(
        decode_authoring(
            &bytes,
            Limits {
                fields: 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
    let limits = a::ResourceLimits {
        live_nodes: 10_000_000,
        ..Default::default()
    };
    validate_message(
        ".pivot.authoring.v1.ResourceLimits",
        &limits.encode_to_vec(),
        Limits::default(),
    )
    .unwrap();
}
#[test]
fn unknown_enum_float_values_and_noncanonical_ids_reject() {
    let bad = a::ReadDraft {
        revision: Some(revision()),
        section: 99,
        page: None,
    };
    assert!(
        validate_message(
            ".pivot.authoring.v1.ReadDraft",
            &bad.encode_to_vec(),
            Limits::default()
        )
        .is_err()
    );
    let def = a::NodeDefinition {
        kind: Some(a::node_definition::Kind::Input(v1::Value {
            kind: Some(v1::value::Kind::Float64(1.0)),
        })),
    };
    assert!(
        validate_message(
            ".pivot.authoring.v1.NodeDefinition",
            &def.encode_to_vec(),
            Limits::default()
        )
        .is_err()
    );
    let r = a::WorkingRevision {
        service_instance_id: ID.to_uppercase(),
        ..revision()
    };
    assert!(
        validate_message(
            ".pivot.authoring.v1.WorkingRevision",
            &r.encode_to_vec(),
            Limits::default()
        )
        .is_err()
    );
}
#[test]
fn negotiation_is_opt_in_and_legacy_daemon_profile_stays_invocation_only() {
    let hello = v1::Handshake {
        supported_versions: vec![
            v1::Version { major: 1, minor: 0 },
            v1::Version { major: 1, minor: 1 },
        ],
        required_capabilities: vec!["working-models-v1".into()],
        optional_capabilities: vec![],
    };
    let accepted = negotiate(&hello).unwrap();
    assert_eq!(
        accepted.selected_version,
        Some(v1::Version { major: 1, minor: 1 })
    );
    assert!(accepted.capabilities.contains(&"decimal96".into()));
    assert!(pivot_protocol::validation::negotiate(&hello).is_err());
    let old = v1::Handshake {
        required_capabilities: vec![],
        supported_versions: vec![v1::Version { major: 1, minor: 0 }],
        ..hello
    };
    assert_eq!(
        negotiate(&old).unwrap().selected_version,
        Some(v1::Version { major: 1, minor: 0 })
    );
}
#[test]
fn exact_decimal_layout_and_presence_are_part_of_intent() {
    fn request(scale: u32) -> a::AuthoringRequest {
        a::AuthoringRequest {
            operation: Some(a::authoring_request::Operation::ApplyEdits(a::ApplyEdits {
                mutation: Some(a::MutationContext {
                    service_instance_id: ID.into(),
                    working_model_id: ID.into(),
                    expected_revision: Some(revision()),
                    operation_id: "edit".into(),
                }),
                edits: vec![a::Edit {
                    kind: Some(a::edit::Kind::AddNode(a::AddNode {
                        temporary_id: "n".into(),
                        name: "N".into(),
                        definition: Some(a::NodeDefinition {
                            kind: Some(a::node_definition::Kind::Input(v1::Value {
                                kind: Some(v1::value::Kind::Decimal(v1::Decimal128 {
                                    lo: 10u32.pow(scale),
                                    scale,
                                    ..Default::default()
                                })),
                            })),
                        }),
                    })),
                }],
            })),
        }
    }
    let a = request(1);
    let b = request(2);
    decode_authoring(&a.encode_to_vec(), Limits::default()).unwrap();
    assert_ne!(mutation_intent(&a), mutation_intent(&b));
}
#[test]
fn outer_envelope_cannot_hide_a_conflicting_mutation() {
    let request = v1::Request {
        request_id: "r".into(),
        version: Some(v1::Version { major: 1, minor: 1 }),
        operation: Some(v1::request::Operation::Authoring(create())),
    };
    let mut bytes = request.encode_to_vec();
    decode_envelope(&bytes, Limits::default()).unwrap();
    bytes.extend([0x5a, 0]);
    assert!(decode_envelope(&bytes, Limits::default()).is_err());
}

#[test]
fn read_only_responses_tolerate_unknown_metadata_but_check_correlation_and_kind() {
    let request = v1::Request {
        request_id: "response".into(),
        version: Some(v1::Version { major: 1, minor: 1 }),
        operation: Some(v1::request::Operation::Authoring(a::AuthoringRequest {
            operation: Some(a::authoring_request::Operation::CloseAttachment(
                a::CloseAttachment {
                    service_instance_id: ID.into(),
                    attachment_id: ID.into(),
                },
            )),
        })),
    };
    let response = v1::Response {
        request_id: request.request_id.clone(),
        version: request.version,
        outcome: Some(v1::response::Outcome::Authoring(a::AuthoringResponse {
            outcome: Some(a::authoring_response::Outcome::Closed(a::Closed {})),
        })),
    };
    let mut bytes = response.encode_to_vec();
    bytes.extend([0xf8, 7, 1]);
    decode_response(&bytes, &request, Limits::default()).unwrap();
    let wrong = v1::Response {
        request_id: "other".into(),
        ..response.clone()
    };
    assert!(decode_response(&wrong.encode_to_vec(), &request, Limits::default()).is_err());
    let wrong = v1::Response {
        outcome: Some(v1::response::Outcome::Authoring(a::AuthoringResponse {
            outcome: Some(a::authoring_response::Outcome::Opened(a::Opened {
                attachment_id: ID.into(),
                revision: Some(revision()),
            })),
        })),
        ..response
    };
    assert!(decode_response(&wrong.encode_to_vec(), &request, Limits::default()).is_err());
}

#[test]
fn ordered_repeated_edits_are_valid_and_scope_mismatches_are_not() {
    let mut request = a::ApplyEdits {
        mutation: Some(a::MutationContext {
            service_instance_id: ID.into(),
            working_model_id: ID.into(),
            expected_revision: Some(revision()),
            operation_id: "many".into(),
        }),
        edits: vec![
            a::Edit {
                kind: Some(a::edit::Kind::AddNode(a::AddNode {
                    temporary_id: "n".into(),
                    name: "N".into(),
                    definition: Some(a::NodeDefinition {
                        kind: Some(a::node_definition::Kind::FormulaSource(String::new())),
                    }),
                })),
            },
            a::Edit {
                kind: Some(a::edit::Kind::RenameNode(a::RenameNode {
                    node: Some(a::NodeReference {
                        kind: Some(a::node_reference::Kind::TemporaryId("n".into())),
                    }),
                    name: "Renamed".into(),
                })),
            },
        ],
    };
    let wrap = |v| a::AuthoringRequest {
        operation: Some(a::authoring_request::Operation::ApplyEdits(v)),
    };
    decode_authoring(&wrap(request.clone()).encode_to_vec(), Limits::default()).unwrap();
    request.mutation.as_mut().unwrap().working_model_id =
        "00000000-0000-0000-0000-000000000000".into();
    assert!(decode_authoring(&wrap(request).encode_to_vec(), Limits::default()).is_err());
}
