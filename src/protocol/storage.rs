use crate::protocol::{Data, RRError};
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

pub type StorageSink = Sender<StorageRequest>;

pub enum StorageRequest {
    Set(String, Data, tokio::sync::oneshot::Sender<Result<(), RRError>>),
    Get(String, tokio::sync::oneshot::Sender<Result<Data, RRError>>),
}

pub trait StorageProxy: Send + Sync + 'static {
    fn get_tx(&self) -> Sender<StorageRequest>;
    fn listen(&mut self) -> impl Future<Output=Result<(), RRError>> + Send;
}

pub struct DefaultStorageProxy {
    map: HashMap<String, Data>,
    rx: Receiver<StorageRequest>,
    tx: Sender<StorageRequest>,
}

impl StorageProxy for DefaultStorageProxy {
    fn get_tx(&self) -> StorageSink {
        self.tx.clone()
    }

    async fn listen(&mut self) -> Result<(), RRError> {
        loop {
            let req = self.rx.recv().await;
            if let Some(req) = req {
                match req {
                    StorageRequest::Set(key, value, tx) => {
                        self.map.insert(key, value);
                        tx.send(Ok(())).unwrap_or_default();
                    }
                    StorageRequest::Get(key, tx) => {
                        let result = self.map.get(&key);
                        tx.send(
                            match result {
                                None => Err(RRError::new("Not found")),
                                Some(val) => Ok(val.clone()),
                            }
                        ).unwrap_or_default();
                    }
                }
                continue;
            }
            return Ok(());
        }
    }
}

impl DefaultStorageProxy {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        Self {
            tx,
            rx,
            map: HashMap::new(),
        }
    }
}