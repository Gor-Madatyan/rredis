use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum SerializationErrorKind {
    FieldNotOptional(String),
    FormatError,
}

#[derive(Debug, Clone)]
pub enum StorageErrorKind {
    FieldNotFound,
    UnexpectedError,
}

impl TryFrom<i32> for StorageErrorKind {
    type Error = RRError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(StorageErrorKind::FieldNotFound),
            2 => Ok(StorageErrorKind::UnexpectedError),
            _ => Err(RRErrorKind::SerializationError(SerializationErrorKind::FormatError).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetworkErrorKind {
    ConnectionFailed,
    BindingToAddrFailed,
    InvalidRequestType,
    FrameWriteError,
    FrameReadError,
}

impl TryFrom<i32> for NetworkErrorKind {
    type Error = RRError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NetworkErrorKind::ConnectionFailed),
            2 => Ok(NetworkErrorKind::BindingToAddrFailed),
            3 => Ok(NetworkErrorKind::InvalidRequestType),
            4 => Ok(NetworkErrorKind::FrameWriteError),
            5 => Ok(NetworkErrorKind::FrameReadError),
            _ => Err(RRErrorKind::SerializationError(SerializationErrorKind::FormatError).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RRErrorKind {
    SerializationError(SerializationErrorKind),
    StorageError(StorageErrorKind),
    NetworkError(NetworkErrorKind),
}

impl From<SerializationErrorKind> for RRErrorKind {
    fn from(value: SerializationErrorKind) -> Self {
        RRErrorKind::SerializationError(value)
    }
}

impl From<StorageErrorKind> for RRErrorKind {
    fn from(value: StorageErrorKind) -> Self {
        RRErrorKind::StorageError(value)
    }
}

impl From<NetworkErrorKind> for RRErrorKind {
    fn from(value: NetworkErrorKind) -> Self {
        RRErrorKind::NetworkError(value)
    }
}

/// The error type used absolutely for all errors from rredis.
#[derive(Debug, Clone)]
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
    pub fn decompose(self) -> (RRErrorKind, Option<String>) {
        (self.kind, self.message)
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
