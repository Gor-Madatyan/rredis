use crate::protocol::error::{RRError, RRErrorKind, SerializationErrorKind};
use bytes::Buf;

pub mod protocol;
mod repr;

pub fn decode_length_delimiter(buf: impl Buf) -> Result<usize, RRError> {
    prost::decode_length_delimiter(buf).map_err(|_| RRErrorKind::SerializationError(
        SerializationErrorKind::FormatError
    ).into())
}