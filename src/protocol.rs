use bytes::Bytes;
pub type ManyData = Vec<Data>;

/// The request types that can be sent/recieved
pub enum RequestType{
    Get,
    Set
}

/// Represents all the data that is sent __and/or__ received by server
/// ## BY THE WAY
/// you can store `JSON` and/or other serialization formats as ByteStream or String. It is up to you
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

/// The basic unit transferred over network.\
/// get and set requests are both implemented as the same [`Frame`] structure.
/// # See also
/// [`Frame::new_set_request`] and [`Frame::new_get_request`]
pub struct Frame<T:ToString> {
    // the key to access
    key: T,
    // data to transfer (optional for get requests)
    data: Option<Data>,
    // additional context
    payload: Option<Data>
}

impl<T> Frame<T> where T: ToString{

    /// create new `get` request\
    /// since the request is get, only the key to get is required and optional payload
    pub fn new_get_request(key: T, payload: Option<Data>) -> Self {
        Self{
            key,
            data:None,
            payload
        }
    }

    /// create new `get` request\
    /// since the request is get, only the key to get is required and optional payload
    pub fn new_set_request(key: T, data: Data, payload: Data) -> Self {
        Self{
            key,
            data:Some(data),
            payload:Some(payload)
        }
    }

    pub fn request_type(&self) -> RequestType {
        if let Some(_) = self.data { RequestType::Set } else { RequestType::Get }
    }
}