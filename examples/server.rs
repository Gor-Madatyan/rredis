use my_redis::connection::Connection;
use my_redis::protocol::error::{RRErrorKind, StorageErrorKind};
use my_redis::protocol::handler::Handler;
use my_redis::protocol::storage::{DefaultStorageProxy, StorageRequest, StorageSink};
use my_redis::protocol::{error::RRError, Data, NetworkFrame};

struct MyHandler {}

impl Clone for MyHandler {
    fn clone(&self) -> Self {
        MyHandler {}
    }
}

impl Handler for MyHandler {
    async fn handle_set_request(&mut self, key: String, value: Data, _payload: Option<Data>, sink: StorageSink) -> Result<NetworkFrame, RRError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        sink.send(StorageRequest::Set(key, value, tx)).await.map_err(|_| RRErrorKind::StorageError(StorageErrorKind::UnexpectedError))?;
        let _ = rx.await.map_err(|_| RRErrorKind::StorageError(StorageErrorKind::UnexpectedError))??;
        Ok(NetworkFrame::new_data_request(Data::NULL, None))
    }

    async fn handle_get_request(&mut self, key: String, _payload: Option<Data>, sink: StorageSink) -> Result<NetworkFrame, RRError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        sink.send(StorageRequest::Get(key, tx)).await.map_err(|_| RRErrorKind::StorageError(StorageErrorKind::UnexpectedError))?;
        let rx = rx.await.map_err(|_| RRErrorKind::StorageError(StorageErrorKind::UnexpectedError))??;
        println!("Got response: {:?}", rx);
        Ok(NetworkFrame::new_data_request(rx, None))
    }
}

#[tokio::main]
async fn main() {
    let storage = DefaultStorageProxy::new();
    if let Err(app) = Connection::app("127.0.0.1:1234", MyHandler {}, storage).await {
        println!("{:?}", app);
        return;
    }
}