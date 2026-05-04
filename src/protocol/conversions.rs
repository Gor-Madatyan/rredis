// Conversions to native types

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
            Request::Set { key, value } => repr::frame::Request::Set(repr::SetRequest { key: key.into(), value: Some(value.into()) }),
            Request::Data { value } => repr::frame::Request::Data(repr::DataRequest { value: Some(value.into()) }),
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
