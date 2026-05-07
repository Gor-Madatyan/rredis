pub mod error;
pub mod conversions;

use crate::protocol::error::{RRErrorKind, SerializationErrorKind};
use crate::repr;
use bytes::{Buf, Bytes};
use error::RRError;
use prost::Message;
use std::fmt::Debug;

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
    /// Refusal is better than silence :)
    Error { error: RRError },
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
    pub fn new(request: Request<T>, payload: Option<Data>) -> Frame<T> {
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

    /// Sometimes it is important to say no !!!!
    pub fn new_error_request(error: RRError, payload: Option<Data>) -> Self {
        Self {
            request: Request::Error { error },
            payload,
        }
    }

    pub fn encode_length_delimited_to_vec(self) -> Vec<u8> {
        repr::Frame::from(self).encode_length_delimited_to_vec()
    }

    pub fn decode(buf: impl Buf) -> Result<Frame<String>, RRError> {
        Ok(Frame::try_from(
            repr::Frame::decode(buf).map_err(|_| {
                RRErrorKind::SerializationError(SerializationErrorKind::FormatError)
            })?,
        )?)
    }

    /// gives the primary request and the optional payload
    pub fn decompose(self) -> (Request<T>, Option<Data>) {
        (self.request, self.payload)
    }
}