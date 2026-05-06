mod error_conversions;

use crate::protocol;
use crate::protocol::error::{RRError, RRErrorKind, SerializationErrorKind as PSerializationErrorKind};
use crate::repr::frame::Request;

// to include codegen
include!(concat!(env!("OUT_DIR"), "/network_protocol.rs"));

// Conversions to protocol types

impl From<Frame> for Result<protocol::NetworkFrame, RRError> {
    fn from(value: Frame) -> Self {
        let request = value.request;
        let payload = value.payload;

        let request = Result::from(request.ok_or(RRErrorKind::SerializationError(
            PSerializationErrorKind::FieldNotOptional("request".into()),
        ))?)?;
        let payload = match payload {
            None => None,
            Some(payload) => Some(Result::from(payload)?),
        };
        Ok(protocol::Frame::new(request, payload))
    }
}

impl From<Request> for Result<protocol::Request<String>, RRError> {
    fn from(value: Request) -> Self {
        match value {
            Request::Get(req) => Ok(protocol::Request::Get { key: req.key }),
            Request::Set(req) => Ok(protocol::Request::Set {
                key: req.key,
                value: Result::from(req.value.ok_or(RRErrorKind::SerializationError(
                    PSerializationErrorKind::FieldNotOptional("value (set request)".into()),
                ))?)?,
            }),
            Request::Data(req) => Ok(protocol::Request::Data {
                value: Result::from(req.value.ok_or(RRErrorKind::SerializationError(
                    PSerializationErrorKind::FieldNotOptional("value (data request)".into()),
                ))?)?,
            }),
            Request::Error(e) => {
                Ok(protocol::Request::Error {
                    error: RRError::new(
                        Result::from(e.kind.ok_or(
                            RRErrorKind::SerializationError(
                                PSerializationErrorKind::FieldNotOptional("kind (error)".into())
                            )
                        )?)?,
                        e.message,
                    )
                })
            }
        }
    }
}

impl From<Data> for Result<protocol::Data, RRError> {
    fn from(value: Data) -> Self {
        match value
            .kind
            .ok_or(RRErrorKind::SerializationError(
                PSerializationErrorKind::FieldNotOptional("kind (data.kind)".into()),
            ))?
        {
            data::Kind::UInteger(int) => Ok(protocol::Data::UInteger(int)),
            data::Kind::SInteger(int) => Ok(protocol::Data::SInteger(int)),
            data::Kind::NullValue(_) => Ok(protocol::Data::NULL),
            data::Kind::StringValue(str) => Ok(protocol::Data::String(str)),
            data::Kind::Array(arr) => Ok(protocol::Data::Array({
                let mut v: Vec<protocol::Data> = Vec::new();
                for d in arr.elements {
                    v.push(Result::from(d)?)
                }
                v
            })),
            data::Kind::ByteStream(s) => Ok(protocol::Data::ByteStream(s.into())),
        }
    }
}
