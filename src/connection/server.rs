mod helper;

use crate::protocol::error::RRError;
use crate::protocol::handler::Handler;
use crate::protocol::storage::StorageProxy;
use helper::*;
use tokio::net::{TcpListener, ToSocketAddrs};

pub struct ServerBuilder<A: ToSocketAddrs, S: StorageProxy, H: Handler> {
    address: A,
    storage: S,
    handler: H,
}

impl<A: ToSocketAddrs, S: StorageProxy, H: Handler> ServerBuilder<A, S, H> {
    pub fn new(address: A, storage_proxy: S, handler: H) -> Self {
        Self {
            address,
            storage: storage_proxy,
            handler,
        }
    }

    pub async fn build(self) -> Result<Server<S, H>, RRError> {
        Ok(Server::new(
            server(self.address).await?,
            self.storage,
            self.handler,
        ))
    }
}

pub struct Server<S: StorageProxy, H: Handler> {
    listener: TcpListener,
    storage: S,
    handler: H,
}

impl<S: StorageProxy, H: Handler> Server<S, H> {
    fn new(listener: TcpListener, storage: S, handler: H) -> Self {
        Self {
            listener,
            storage,
            handler,
        }
    }

    pub async fn run(self) -> Result<(), RRError> {
        let Self {
            listener,
            storage,
            handler,
        } = self;
        let tx = storage.get_tx();
        start_storage(storage);
        Ok(event_loop(listener, handler, tx).await?)
    }
}
