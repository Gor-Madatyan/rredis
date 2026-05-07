use crate::protocol::error::{RRError, RRErrorKind, SerializationErrorKind};
use crate::repr::serialization_error_kind::Kind;
use crate::repr::{rr_error_kind, RrErrorKind};
impl From<RrErrorKind> for Result<RRErrorKind, RRError> {
    fn from(value: RrErrorKind) -> Self {
        let kind = value.kind.ok_or(RRErrorKind::SerializationError(
            SerializationErrorKind::FieldNotOptional("kind (error)".into()),
        ))?;

        match kind {
            rr_error_kind::Kind::SerializationError(e) => {
                match e.kind.ok_or(RRErrorKind::SerializationError(
                    SerializationErrorKind::FieldNotOptional("kind (error)".into()),
                ))? {
                    Kind::FieldNotOptional(k) => Ok(RRErrorKind::SerializationError(
                        SerializationErrorKind::FieldNotOptional(k)
                    )),
                    Kind::FormatError(_) => Ok(RRErrorKind::SerializationError(
                        SerializationErrorKind::FormatError
                    )),
                }
            }
            rr_error_kind::Kind::StorageError(e) => Ok(RRErrorKind::StorageError(e.try_into()?)),
            rr_error_kind::Kind::NetworkError(e) => Ok(RRErrorKind::NetworkError(e.try_into()?)),
        }
    }
}
