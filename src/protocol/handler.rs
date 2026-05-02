use crate::protocol::{Data, NetworkFrame, RRError};

pub trait Handler: Clone + Send + Sync + 'static {
    fn handle_set_request(&mut self, key: String, value: Data, payload: Option<Data>) -> impl Future<Output=Result<NetworkFrame, RRError>> + Send;
    fn handle_get_request(&mut self, key: String, payload: Option<Data>) -> impl Future<Output=Result<NetworkFrame, RRError>> + Send;
}