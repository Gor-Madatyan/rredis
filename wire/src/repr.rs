use crate::field_not_optional;
use crate::protocol::error::{
    RRError, RRErrorKind, SerializationErrorKind as PSerializationErrorKind,
};
use crate::repr::frame::Request;
use crate::{cast_or_throw, protocol, try_to_protocol};

mod macros;

// to include codegen
include!(concat!(env!("OUT_DIR"), "/network_protocol.rs"));

// Conversions to protocol types

try_to_protocol! {Frame, protocol::NetworkFrame, (value) =>
      let request = value.request;
      let payload = value.payload;

      let request = cast_or_throw!(request, "request");
      let payload = match payload {
          None => None,
          Some(payload) => Some(payload.try_into()?),
      };
      Ok(protocol::Frame::new_from_id(request, value.request_id, payload))
}

try_to_protocol! {Request, protocol::Request<String>, (value) =>
        match value {
            Request::Get(req) => Ok(protocol::Request::Get { key: req.key }),
            Request::Set(req) => Ok(protocol::Request::Set {
                key: req.key,
                value: cast_or_throw!(req.value, "value (set request)"),
        }),
            Request::Data(req) => Ok(protocol::Request::Data {
                key: req.key,
                value: cast_or_throw!(req.value, "value (data request)"),
        }),
            Request::Error(e) => Ok(protocol::Request::Error {
                error: RRError::new(
                    cast_or_throw!(e.kind,"kind (error)"),
                    e.message,
                ),
        }),
        }
}

try_to_protocol! {Data, protocol::Data, (value) =>
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

try_to_protocol! {RrErrorKind, RRErrorKind, (value) =>
    let kind = field_not_optional!(value.kind, "kind (error)");

        match kind {
            rr_error_kind::Kind::SerializationError(e) =>
                match field_not_optional!(e.kind, "kind (error)") {
                    serialization_error_kind::Kind::FieldNotOptional(k) =>
                        Ok(PSerializationErrorKind::FieldNotOptional(k).into()),
                    serialization_error_kind::Kind::FormatError(_) =>
                        Ok(PSerializationErrorKind::FormatError.into()),
                },
            rr_error_kind::Kind::StorageError(e) => Ok(RRErrorKind::StorageError(e.try_into()?)),
            rr_error_kind::Kind::NetworkError(e) => Ok(RRErrorKind::NetworkError(e.try_into()?)),
        }
}
