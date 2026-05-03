use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SerializationErrorKind {
    FieldNotOptional(&'static str),
    FormatError,
}

#[derive(Debug)]
pub enum StorageErrorKind {
    FieldNotFound,
    UnexpectedError,
}

#[derive(Debug)]
pub enum NetworkErrorKind {
    ConnectionFailed,
    BindingToAddrFailed,
    InvalidRequestType,
    FrameWriteError,
    FrameReadError,
}

#[derive(Debug)]
pub enum RRErrorKind {
    SerializationError(SerializationErrorKind),
    StorageError(StorageErrorKind),
    NetworkError(NetworkErrorKind),
}

/// The error type used absolutely for all errors from rredis.
#[derive(Debug)]
pub struct RRError {
    kind: RRErrorKind,
    message: Option<String>,
}

impl From<RRErrorKind> for RRError {
    fn from(kind: RRErrorKind) -> Self {
        Self::new(kind, None)
    }
}

impl RRError {
    pub fn new(kind: RRErrorKind, message: Option<String>) -> Self {
        Self { kind, message }
    }
}

impl Display for RRError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ref message) = self.message {
            write!(
                f,
                "Error of kind: {:?} with message: {}",
                self.kind, message
            )
        } else {
            write!(f, "Error of kind: {:?}", self.kind)
        }
    }
}

impl Error for RRError {}
