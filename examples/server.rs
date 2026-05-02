use my_redis::connection::Connection;
use my_redis::protocol::handler::Handler;
use my_redis::protocol::storage::{DefaultStorageProxy, StorageRequest, StorageSink};
use my_redis::protocol::{Data, NetworkFrame, RRError};

struct MyHandler {}

impl Clone for MyHandler {
    fn clone(&self) -> Self {
        MyHandler {}
    }
}

impl Handler for MyHandler {
    async fn handle_set_request(&mut self, key: String, value: Data, payload: Option<Data>, sink: StorageSink) -> Result<NetworkFrame, RRError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        sink.send(StorageRequest::Set(key, value, tx)).await?;
        let _ = rx.await??;
        Ok(NetworkFrame::new_data_request(Data::NULL, None))
    }

    async fn handle_get_request(&mut self, key: String, payload: Option<Data>, sink: StorageSink) -> Result<NetworkFrame, RRError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        sink.send(StorageRequest::Get(key, tx)).await?;
        let rx = rx.await??;
        println!("Got response: {:?}", rx);
        return Ok(NetworkFrame::new_data_request(rx, None));
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