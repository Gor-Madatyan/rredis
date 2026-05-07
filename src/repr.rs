mod error_conversions;

use crate::protocol;
use crate::protocol::error::{
    RRError, RRErrorKind, SerializationErrorKind as PSerializationErrorKind,
};
use crate::repr::frame::Request;

// to include codegen
include!(concat!(env!("OUT_DIR"), "/network_protocol.rs"));

// Conversions to protocol types

impl TryFrom<Frame> for protocol::NetworkFrame {
    type Error = RRError;
    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let request = value.request;
        let payload = value.payload;

        let request = request
            .ok_or(RRErrorKind::SerializationError(
                PSerializationErrorKind::FieldNotOptional("request".into()),
            ))?
            .try_into()?;
        let payload = match payload {
            None => None,
            Some(payload) => Some(payload.try_into()?),
        };
        Ok(protocol::Frame::new(request, payload))
    }
}

impl TryFrom<Request> for protocol::Request<String> {
    type Error = RRError;
    fn try_from(value: Request) -> Result<protocol::Request<String>, RRError> {
        match value {
            Request::Get(req) => Ok(protocol::Request::Get { key: req.key }),
            Request::Set(req) => Ok(protocol::Request::Set {
                key: req.key,
                value: req
                    .value
                    .ok_or(RRErrorKind::SerializationError(
                        PSerializationErrorKind::FieldNotOptional("value (set request)".into()),
                    ))?
                    .try_into()?,
            }),
            Request::Data(req) => Ok(protocol::Request::Data {
                value: req
                    .value
                    .ok_or(RRErrorKind::SerializationError(
                        PSerializationErrorKind::FieldNotOptional("value (data request)".into()),
                    ))?
                    .try_into()?,
            }),
            Request::Error(e) => Ok(protocol::Request::Error {
                error: RRError::new(
                    Result::from(e.kind.ok_or(RRErrorKind::SerializationError(
                        PSerializationErrorKind::FieldNotOptional("kind (error)".into()),
                    ))?)?,
                    e.message,
                ),
            }),
        }
    }
}

impl TryFrom<Data> for protocol::Data {
    type Error = RRError;
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value.kind.ok_or(RRErrorKind::SerializationError(
            PSerializationErrorKind::FieldNotOptional("kind (data.kind)".into()),
        ))? {
            data::Kind::UInteger(int) => Ok(protocol::Data::UInteger(int)),
            data::Kind::SInteger(int) => Ok(protocol::Data::SInteger(int)),
            data::Kind::NullValue(_) => Ok(protocol::Data::NULL),
            data::Kind::StringValue(str) => Ok(protocol::Data::String(str)),
            data::Kind::Array(arr) => Ok(protocol::Data::Array({
                let mut v: Vec<protocol::Data> = Vec::new();
                for d in arr.elements {
                    v.push(d.try_into()?)
                }
                v
            })),
            data::Kind::ByteStream(s) => Ok(protocol::Data::ByteStream(s.into())),
        }
    }
}
