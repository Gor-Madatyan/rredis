use crate::protocol::storage::StorageSink;
use crate::protocol::{error::RRError, Data, NetworkFrame};

pub trait Handler: Clone + Send + Sync + 'static {
    fn handle_set_request(&mut self, key: String, value: Data, payload: Option<Data>, sink: StorageSink) -> impl Future<Output=Result<NetworkFrame, RRError>> + Send;
    fn handle_get_request(&mut self, key: String, payload: Option<Data>, sink: StorageSink) -> impl Future<Output=Result<NetworkFrame, RRError>> + Send;
}