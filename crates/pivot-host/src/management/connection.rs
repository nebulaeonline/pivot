//! Local daemon session routing. Peer authentication belongs to the transport.
use super::{
    WorkingModelService,
    adapter::{AuthoringAdapter, AuthoringSession},
};
use crate::{Authorizer, Limits, Session};
use pivot_protocol::{foundation, v1 as p};
use prost::Message;
use std::sync::{Arc, Mutex};

pub struct Connection {
    service: Arc<Mutex<WorkingModelService>>,
    adapter: Option<Arc<AuthoringAdapter>>,
    principal: String,
    invocation: Session,
    authoring: Option<AuthoringSession>,
    preflight: foundation::Limits,
}
impl Connection {
    pub fn new(
        service: Arc<Mutex<WorkingModelService>>,
        adapter: Option<Arc<AuthoringAdapter>>,
        principal: String,
        policy: Arc<dyn Authorizer>,
    ) -> Result<Self, super::Error> {
        if principal.is_empty() || principal.len() > 256 {
            return Err(super::Error::InvalidArgument);
        }
        let preflight = adapter
            .as_ref()
            .map_or_else(foundation::Limits::default, |a| a.request_limits());
        Ok(Self {
            service,
            adapter,
            principal,
            invocation: Session::new(Limits::default(), policy)
                .map_err(|_| super::Error::InvalidArgument)?,
            authoring: None,
            preflight,
        })
    }
    fn host(&self) -> crate::ModelHost {
        self.service
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .published_host()
            .clone()
    }
    pub fn dispatch(&mut self, bytes: &[u8]) -> Vec<u8> {
        if let Some(authoring) = &mut self.authoring {
            let request = match foundation::decode_session_request(bytes, self.preflight) {
                Ok(r) => r,
                Err(e) => return failure("", 1, e.code),
            };
            if request.version != Some(p::Version { major: 1, minor: 1 }) {
                return failure(&request.request_id, 1, p::ErrorCode::UnsupportedVersion);
            }
            if matches!(request.operation, Some(p::request::Operation::Authoring(_))) {
                return authoring.dispatch(bytes);
            }
            if matches!(request.operation, Some(p::request::Operation::Handshake(_))) {
                return failure(&request.request_id, 1, p::ErrorCode::InvalidArgument);
            }
            // v1.1 preserves invocation semantics. Reuse the existing validator,
            // policy and execution path with an internal v1.0 envelope.
            let mut legacy = request;
            legacy.version = Some(p::Version { major: 1, minor: 0 });
            let encoded = self
                .host()
                .dispatch(&mut self.invocation, &legacy.encode_to_vec());
            let mut response = p::Response::decode(encoded.as_slice()).expect("host response");
            response.version = Some(p::Version { major: 1, minor: 1 });
            if response.encoded_len() > self.preflight.bytes {
                return failure(&response.request_id, 1, p::ErrorCode::ResourceExhausted);
            }
            return response.encode_to_vec();
        }
        if !self.invocation.negotiated()
            && let Some(adapter) = &self.adapter
        {
            let request = match foundation::decode_handshake(bytes, self.preflight) {
                Ok(r) => r,
                Err(e) => {
                    return failure(
                        "",
                        0,
                        if e.code == p::ErrorCode::ResourceExhausted {
                            e.code
                        } else {
                            p::ErrorCode::HandshakeRequired
                        },
                    );
                }
            };
            let Some(p::request::Operation::Handshake(hello)) = request.operation.as_ref() else {
                unreachable!()
            };
            if let Ok(accepted) = foundation::negotiate(hello)
                && accepted.selected_version == Some(p::Version { major: 1, minor: 1 })
            {
                let mut session = match adapter.open_session(&self.principal) {
                    Ok(s) => s,
                    Err(_) => {
                        return failure(&request.request_id, 1, p::ErrorCode::ResourceExhausted);
                    }
                };
                let response = session.dispatch(bytes);
                let p::Response {
                    outcome: Some(p::response::Outcome::Handshake(_)),
                    ..
                } = p::Response::decode(response.as_slice()).expect("adapter response")
                else {
                    return response;
                };
                let legacy = p::Request {
                    request_id: "bootstrap".into(),
                    version: Some(p::Version { major: 1, minor: 0 }),
                    operation: Some(p::request::Operation::Handshake(p::Handshake {
                        supported_versions: vec![p::Version { major: 1, minor: 0 }],
                        ..Default::default()
                    })),
                };
                self.host()
                    .dispatch(&mut self.invocation, &legacy.encode_to_vec());
                self.authoring = Some(session);
                return response;
            }
        }
        self.host().dispatch(&mut self.invocation, bytes)
    }
}
fn failure(id: &str, minor: u32, code: p::ErrorCode) -> Vec<u8> {
    p::Response {
        request_id: id.into(),
        version: Some(p::Version { major: 1, minor }),
        outcome: Some(p::response::Outcome::Error(p::ProtocolError {
            code: code as i32,
            message: "request rejected".into(),
            port_id: String::new(),
        })),
    }
    .encode_to_vec()
}
