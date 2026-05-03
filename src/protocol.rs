pub mod handler;
pub mod storage;
pub mod error;
use crate::repr;
use bytes::Bytes;
use std::error::Error;
use std::fmt::{Debug, Display};
pub type ManyData = Vec<Data>;
pub type NetworkFrame = Frame<String>;

/// Represents all the data that is sent __and/or__ received by server
/// ## BY THE WAY
/// you can store `JSON` and/or other serialization formats as ByteStream or String. It is up to you
#[derive(Clone, Debug)]
pub enum Data {
    /// Used for sending raw bytes (there is not better type for them), e.g. images
    ByteStream(Bytes),
    /// UTF-8 encoded string
    String(String),
    /// Signed 64 bits integer
    SInteger(i64),
    /// Unsigned 64 bits integer
    UInteger(u64),
    /// Array of [`Data`]
    Array(ManyData),
    /// Used to unset fields in set requests
    NULL,
}

/// The request types that can be sent/recieved
#[derive(Clone, Debug)]
pub enum Request<T>
where
    T: Into<String>,
{
    /// Get request, __sent to server__
    Get { key: T },
    /// Set request, __sent to server__
    Set { key: T, value: Data },
    /// plain data, __sent to client__
    Data { value: Data },
}

/// The basic unit transferred over network.\
/// it encapsulates all the possible means of communication acceptable.\
/// everything transferred over the network is a Frame.
/// # See also
/// [`Frame::new_data_request`], [`Frame::new_set_request`] and [`Frame::new_get_request`]
#[derive(Clone, Debug)]
pub struct Frame<T: Into<String>> {
    // the core request
    request: Request<T>,
    // additional context
    payload: Option<Data>,
}

impl<T> Frame<T>
where
    T: Into<String>,
{
    pub(crate) fn new(request: Request<T>, payload: Option<Data>) -> Frame<T> {
        Self { request, payload }
    }
    /// create new `get` request
    pub fn new_get_request(key: T, payload: Option<Data>) -> Self {
        Self {
            request: Request::Get { key },
            payload,
        }
    }

    /// create new `set` request
    pub fn new_set_request(key: T, data: Data, payload: Option<Data>) -> Self {
        Self {
            request: Request::Set { key, value: data },
            payload,
        }
    }

    /// create [`Frame`] to respond to a request made by a client
    pub fn new_data_request(data: Data, payload: Option<Data>) -> Self {
        Self {
            request: Request::Data { value: data },
            payload,
        }
    }

    /// gives the primary request and the optional payload
    pub fn decompose(self) -> (Request<T>, Option<Data>) {
        (self.request, self.payload)
    }
}


// Conversions to native types

impl<T:Into<String> + Display> From<Frame<T>> for repr::Frame {
    fn from(value: Frame<T>) -> Self {
         Self{
             payload: value.payload.map(|d|d.into()),
             request:Some(value.request.into()),

        }
    }
}

impl<T:Into<String>> From<Request<T>> for repr::frame::Request{
    fn from(value: Request<T>) -> Self {
        match value {
            Request::Get { key } => repr::frame::Request::Get(repr::GetRequest{key:key.into()}),
            Request::Set { key,value } => repr::frame::Request::Set(repr::SetRequest{key:key.into(), value: Some(value.into())}),
            Request::Data { value } => repr::frame::Request::Data(repr::DataRequest{value:Some(value.into())}),
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
