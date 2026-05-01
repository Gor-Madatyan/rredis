use std::error::Error;
use std::fmt::{Debug};
use bytes::Bytes;
pub type ManyData = Vec<Data>;

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
    NULL
}

/// The request types that can be sent/recieved
#[derive(Clone, Debug)]
pub enum Request<T>
    where T:Into<String>
{
    /// Get request, __sent to server__
    Get{key:T},
    /// Set request, __sent to server__
    Set{key:T,value:Data},
    /// plain data, __sent to client__
    Data{value:Data},
}

/// The basic unit transferred over network.\
/// it encapsulates all the possible means of communication acceptable.\
/// everything transferred over the network is a Frame.
/// # See also
/// [`Frame::new_data_request`], [`Frame::new_set_request`] and [`Frame::new_get_request`]
#[derive(Clone, Debug)]
pub struct Frame<T:Into<String>> {
    // the core request
    request: Request<T>,
    // additional context
    payload: Option<Data>
}

impl<T> Frame<T> where T: Into<String>{
    pub(crate) fn new(request: Request<T>, payload:Option<Data>) -> Frame<T> {
        Self{
            request,
            payload
        }
    }
    /// create new `get` request
    pub fn new_get_request(key: T, payload: Option<Data>) -> Self {
        Self{
            request: Request::Get{key},
            payload
        }
    }

    /// create new `set` request
    pub fn new_set_request(key: T, data: Data, payload: Option<Data>) -> Self {
        Self{
            request:Request::Set{key,value:data},
            payload
        }
    }

    /// create [`Frame`] to respond to a request made by a client
    pub fn new_data_request(data: Data, payload: Option<Data>) -> Self {
        Self{
            request:Request::Data{value:data},
            payload
        }
    }

    /// gives the primary request and the optional payload
    pub fn decompose(self) -> (Request<T>, Option<Data>) {
        (self.request, self.payload)
    }
}

#[derive(Debug)]
pub struct RRError{
    error: Option<Box<dyn Error>>,
    message:String
}

impl<E:Error + 'static> From<E> for RRError{
    fn from(value: E) -> Self {
        let message = value.to_string();
        Self{error:Some(Box::new(value)),message }
    }
}

impl RRError {
    pub fn new(message:impl Into<String>) -> RRError{
        Self{error:None, message:message.into()}
    }
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
       if let Some(ref err) = self.error {
           Some(&**err)
       }else { None }
    }
}