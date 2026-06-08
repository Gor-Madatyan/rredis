#[macro_export]
macro_rules! try_to_protocol {
    ($from:ty, $to:ty, ($a:ident) => $($b:tt)*) => {
        impl TryFrom<$from> for $to {
            type Error = RRError;
             fn try_from($a: $from) -> Result<Self, RRError> {
                $($b)*
             }
        }
    };
}

#[macro_export]
macro_rules! cast_or_throw {
    ($e:expr, $m:expr) => {
        field_not_optional!($e,$m).try_into()?
    };
}

#[macro_export]
macro_rules! field_not_optional {
    ($e:expr, $m:expr) => {
        $e.ok_or(RRErrorKind::from(PSerializationErrorKind::FieldNotOptional($m.into())))?
    };
}