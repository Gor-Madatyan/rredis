pub mod conversions;
pub mod error;

use crate::protocol::error::{RRErrorKind, SerializationErrorKind};
use crate::repr;
use bytes::{Buf, Bytes};
use error::RRError;
use prost::Message;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

pub type ManyData = Vec<Data>;
pub type NetworkFrame = Frame<String>;

static ID: AtomicU64 = AtomicU64::new(1);

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
    Data { key: T, value: Data },
    /// Refusal is better than silence :)
    Error { error: RRError },
}

impl<T> Request<T>
where
    T: Into<String>,
{
    /// convert all keys to `String`, yielding the canonical network representation
    pub fn into_network(self) -> Request<String> {
        match self {
            Request::Get { key } => Request::Get { key: key.into() },
            Request::Set { key, value } => Request::Set {
                key: key.into(),
                value,
            },
            Request::Data { key, value } => Request::Data {
                key: key.into(),
                value,
            },
            Request::Error { error } => Request::Error { error },
        }
    }
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
    // correlates a response with the request that caused it (0 = unset)
    request_id: u64,
}

impl<T> Frame<T>
where
    T: Into<String>,
{
    pub fn new(request: Request<T>, payload: Option<Data>) -> Frame<T> {
        Self {
            request,
            payload,
            request_id: ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn new_from_id(request: Request<T>, request_id: u64, payload: Option<Data>) -> Frame<T> {
        Self {
            request,
            request_id,
            payload,
        }
    }

    /// create new `get` request
    pub fn new_get_request(key: T, payload: Option<Data>) -> Self {
        Self::new(Request::Get { key }, payload)
    }

    /// create new `set` request
    pub fn new_set_request(key: T, data: Data, payload: Option<Data>) -> Self {
        Self::new(Request::Set { key, value: data }, payload)
    }

    /// create [`Frame`] to respond to a request made by a client
    pub fn new_data_request(key: T, data: Data, payload: Option<Data>) -> Self {
        Self::new(Request::Data { key, value: data }, payload)
    }

    /// Sometimes it is important to say no !!!!
    pub fn new_error_request(error: RRError, payload: Option<Data>) -> Self {
        Self::new(Request::Error { error }, payload)
    }

    pub fn encode_length_delimited_to_vec(self) -> Vec<u8> {
        repr::Frame::from(self).encode_length_delimited_to_vec()
    }

    pub fn decode(buf: impl Buf) -> Result<Frame<String>, RRError> {
        Ok(Frame::try_from(repr::Frame::decode(buf).map_err(
            |_| RRErrorKind::SerializationError(SerializationErrorKind::FormatError),
        )?)?)
    }

    /// gives the primary request and the optional payload
    pub fn decompose(self) -> (Request<T>, Option<Data>, u64) {
        (self.request, self.payload, self.request_id)
    }

    /// convert into a [`NetworkFrame`], keeping the request id
    pub fn into_network(self) -> NetworkFrame {
        Frame {
            request: self.request.into_network(),
            payload: self.payload,
            request_id: self.request_id,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.request_id
    }
}
