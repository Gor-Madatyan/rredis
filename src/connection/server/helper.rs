use crate::connection::Connection;
use crate::protocol::error::{NetworkErrorKind, RRError, RRErrorKind};
use crate::protocol::handler::Handler;
use crate::protocol::storage::{StorageProxy, StorageRequest};
use crate::protocol::Request;
use tokio::net::{TcpListener, ToSocketAddrs};

pub(super) async fn server(socket: impl ToSocketAddrs) -> Result<TcpListener, RRError> {
    Ok(TcpListener::bind(socket).await.map_err(|_| RRErrorKind::NetworkError(
        NetworkErrorKind::BindingToAddrFailed,
    ))?)
}


pub(super) fn handle_connection(mut connection: Connection, mut handler: impl Handler, tx: tokio::sync::mpsc::Sender<StorageRequest>) {
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
            if let Err(_) = res { break; } // TODO send error response
            if let Err(_) = connection.write_frame(res.unwrap()).await { break; }
        }
    });
}

pub(super) fn start_storage(mut storage: impl StorageProxy) {
    tokio::spawn(async move {
        storage.listen().await;
    });
}

pub(super) async fn event_loop(listener: TcpListener, handler: impl Handler, tx: tokio::sync::mpsc::Sender<StorageRequest>) -> Result<(), RRError> {
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