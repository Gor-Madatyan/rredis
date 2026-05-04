use crate::connection::Connection;
use crate::protocol::error::{NetworkErrorKind, RRError, RRErrorKind};
use crate::protocol::handler::Handler;
use crate::protocol::storage::{StorageProxy, StorageRequest};
use crate::protocol::Request;
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

    pub async fn server(socket: impl ToSocketAddrs) -> Result<TcpListener, RRError> {
        Ok(TcpListener::bind(socket).await.map_err(|_| RRErrorKind::NetworkError(
            NetworkErrorKind::BindingToAddrFailed,
        ))?)
    }

    pub async fn build(self) -> Result<Server<S, H>, RRError> {
        Ok(Server::new(Self::server(self.address).await?, self.storage, self.handler))
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
        let Self { listener, storage, handler } = self;
        let tx = storage.get_tx();
        start_storage(storage);
        Ok(event_loop(listener, handler, tx).await?)
    }
}

fn handle_connection(mut connection: Connection, mut handler: impl Handler, tx: tokio::sync::mpsc::Sender<StorageRequest>) {
    tokio::spawn(async move {
        loop {
            let frame = connection.read_frame().await;
            if let Err(_) = frame { break; }
            let (request, payload) = frame.unwrap().decompose();
            let res = match request {
                Request::Get { key } => handler.handle_get_request(key, payload, tx.clone()).await,
                Request::Set { key, value } => handler.handle_set_request(key, value, payload, tx.clone()).await,
                Request::Data { .. } => Err(RRErrorKind::NetworkError(
                    NetworkErrorKind::InvalidRequestType
                ).into())
            };
            if let Err(_) = res { break; }
            if let Err(_) = connection.write_frame(res.unwrap()).await { break; }
        }
    });
}

fn start_storage(mut storage: impl StorageProxy) {
    tokio::spawn(async move {
        storage.listen().await;
    });
}

async fn event_loop(listener: TcpListener, handler: impl Handler, tx: tokio::sync::mpsc::Sender<StorageRequest>) -> Result<(), RRError> {
    loop {
        let (connection, _) = listener.accept().await.map_err(|_| RRErrorKind::NetworkError(
            NetworkErrorKind::ConnectionFailed,
        ))?;
        let connection = Connection::new(connection);
        let _handler = handler.clone();
        let tx = tx.clone();
        handle_connection(connection, _handler, tx);
    }
}