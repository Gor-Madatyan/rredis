use crate::protocol;
use crate::protocol::RRError;
use crate::repr::frame::Request;

include!(concat!(env!("OUT_DIR"), "/network_protocol.rs"));

impl From<Frame> for Result<protocol::Frame<String>,RRError> {
    fn from(value: Frame) -> Self {
        let request = value.request;
        let payload = value.payload;

        let request = Result::from(request.ok_or(RRError::new("request can't ne None"))?)?;
        let payload = match payload {
            None => None,
            Some(payload) => Some(Result::from(payload)?),
        };
        Ok(protocol::Frame::new(request,payload))
    }
}

impl From<Request> for Result<protocol::Request<String>, RRError> {
    fn from(value: Request) -> Self {
        match value {
            Request::Get(req) => Ok(protocol::Request::Get {key:req.key}),
            Request::Set(req) => Ok(protocol::Request::Set {key:req.key,
                value:Result::from(req.value.ok_or(RRError::new("Currently 'value' field is required (it was not renamed)"))?)?
            }),
            Request::Data(req) => Ok(protocol::Request::Data {
                value:Result::from(req.value.ok_or(RRError::new("Currently 'value' field is required (it was not renamed)"))?)?
            }),
        }
    }
}

impl From<Data> for Result<protocol::Data, RRError> {
    fn from(value: Data) -> Self {
        match value.kind.ok_or(RRError::new("The kind of data is required"))? {
            data::Kind::UInteger(int) => Ok(protocol::Data::UInteger(int)),
            data::Kind::SInteger(int) => Ok(protocol::Data::SInteger(int)),
            data::Kind::NullValue(_) => Ok(protocol::Data::NULL),
            data::Kind::StringValue(str) => Ok(protocol::Data::String(str)),
            data::Kind::Array(arr) => Ok(protocol::Data::Array(
                {
                    let mut v:Vec<protocol::Data> = Vec::new();
                    for d in arr.elements{
                        v.push(Result::from(d)?)
                    }
                    v
                }
            )),
            data::Kind::ByteStream(s) => Ok(protocol::Data::ByteStream(s.into()))
        }
    }
}