// Conversions to native types

use crate::protocol::error::{RRError, RRErrorKind, SerializationErrorKind};
use crate::protocol::{Data, Frame, Request};
use crate::repr;
use std::fmt::Display;

impl<T: Into<String> + Display> From<Frame<T>> for repr::Frame {
    fn from(value: Frame<T>) -> Self {
        Self {
            payload: value.payload.map(|d| d.into()),
            request: Some(value.request.into()),
        }
    }
}

impl<T: Into<String>> From<Request<T>> for repr::frame::Request {
    fn from(value: Request<T>) -> Self {
        match value {
            Request::Get { key } => repr::frame::Request::Get(repr::GetRequest { key: key.into() }),
            Request::Set { key, value } => repr::frame::Request::Set(repr::SetRequest {
                key: key.into(),
                value: Some(value.into()),
            }),
            Request::Data { value } => repr::frame::Request::Data(repr::DataRequest {
                value: Some(value.into()),
            }),
            Request::Error { error } => repr::frame::Request::Error(error.into()),
        }
    }
}

impl From<Data> for repr::Data {
    fn from(value: Data) -> Self {
        match value {
            Data::UInteger(d) => repr::Data {
                kind: Some(repr::data::Kind::UInteger(d)),
            },
            Data::SInteger(d) => repr::Data {
                kind: Some(repr::data::Kind::SInteger(d)),
            },
            Data::String(s) => repr::Data {
                kind: Some(repr::data::Kind::StringValue(s)),
            },
            Data::NULL => repr::Data {
                kind: Some(repr::data::Kind::NullValue(repr::NullValue {})),
            },
            Data::ByteStream(b) => repr::Data {
                kind: Some(repr::data::Kind::ByteStream(b.into())),
            },
            Data::Array(a) => repr::Data {
                kind: Some(repr::data::Kind::Array(repr::ManyData {
                    elements: a.into_iter().map(|d| d.into()).collect(),
                })),
            },
        }
    }
}

impl From<RRError> for repr::ErrorRequest {
    fn from(value: RRError) -> Self {
        let (kind, message) = value.decompose();
        repr::ErrorRequest {
            kind: Some(kind.into()),
            message,
        }
    }
}

impl From<RRErrorKind> for repr::RrErrorKind {
    fn from(value: RRErrorKind) -> Self {
        match value {
            RRErrorKind::SerializationError(e) => match e {
                SerializationErrorKind::FieldNotOptional(s) => repr::RrErrorKind {
                    kind: Some(repr::rr_error_kind::Kind::SerializationError(
                        repr::SerializationErrorKind {
                            kind: Some(repr::serialization_error_kind::Kind::FieldNotOptional(s)),
                        },
                    )),
                },

                SerializationErrorKind::FormatError => repr::RrErrorKind {
                    kind: Some(repr::rr_error_kind::Kind::SerializationError(
                        repr::SerializationErrorKind {
                            kind: Some(repr::serialization_error_kind::Kind::FormatError(
                                repr::FormatError {},
                            )),
                        },
                    )),
                },
            },

            RRErrorKind::StorageError(e) => repr::RrErrorKind {
                kind: Some(repr::rr_error_kind::Kind::StorageError((e as i32) + 1)),
            },
            RRErrorKind::NetworkError(e) => repr::RrErrorKind {
                kind: Some(repr::rr_error_kind::Kind::NetworkError((e as i32) + 1)),
            },
        }
    }
}
