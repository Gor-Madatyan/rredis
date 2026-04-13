use bytes::Bytes;
pub type ManyData = Vec<Data>;

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

/// The basic unit transferred over network
/// `key` is used to specify 'path'\
/// if the `data` is [`None`], it is implied that it is a get request\
/// `payload` is used to set additional context
pub struct Frame<T:ToString> {
    key: T,
    data: Option<Data>,
    payload: Option<Data>
}
