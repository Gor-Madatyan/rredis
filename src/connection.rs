use crate::protocol::error::{NetworkErrorKind, RRErrorKind, SerializationErrorKind};
use crate::protocol::storage::StorageProxy;
use crate::protocol::{error::RRError, handler::Handler, Frame, NetworkFrame, Request};
use crate::repr;
use prost::Message;
use std::fmt::Display;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

/// The main structure you will work with.
/// You will send [Frame]s and/or receive them through this abstraction.
pub struct Connection{
    socket:TcpStream
}

impl Connection{
    /// encapsulate the [TcpStream]
    pub async fn new(socket: impl ToSocketAddrs) -> Result<Self, RRError> {
        Ok(Self {
            socket: TcpStream::connect(socket).await.map_err(|_| RRErrorKind::NetworkError(
                NetworkErrorKind::ConnectionFailed,
            ))?
        })
    }

    pub async fn app(socket_addr: impl ToSocketAddrs, handler: impl Handler, mut storage_proxy: impl StorageProxy) -> Result<(), RRError> {
        let listener = TcpListener::bind(socket_addr).await.map_err(|_| RRErrorKind::NetworkError(
            NetworkErrorKind::BindingToAddrFailed,
        ))?;
        let tx = storage_proxy.get_tx();
        tokio::spawn(async move {
            let _ = storage_proxy.listen().await;
        });
        loop {
            let (connection, _) = listener.accept().await.map_err(|_| RRErrorKind::NetworkError(
                NetworkErrorKind::ConnectionFailed,
            ))?;
            let mut connection = Connection { socket: connection };
            let mut handlerc = handler.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                loop {
                    let frame = connection.read_frame().await;
                    if let Err(_) = frame { break; }
                    let (request, payload) = frame.unwrap().decompose();
                    let res = match request {
                        Request::Get { key } => handlerc.handle_get_request(key, payload, tx.clone()).await,
                        Request::Set { key, value } => handlerc.handle_set_request(key, value, payload, tx.clone()).await,
                        Request::Data { .. } => Err(RRErrorKind::NetworkError(
                            NetworkErrorKind::InvalidRequestType
                        ).into())
                    };
                    if let Err(_) = res { break; }
                    if let Err(_) = connection.write_frame(res.unwrap()).await { break; }
                }
            });
        }
    }

    /// Abstraction for sending [Frame] and awaiting the response
    pub async fn sendrecv(&mut self, frame: Frame<impl Into<String> + Display>) -> Result<NetworkFrame, RRError> {
        self.write_frame(frame).await?;
        self.read_frame().await
    }

    /// Send the [Frame].
    /// When the connection closes you will receive an [error](RRError)
    pub async fn write_frame<T:Into<String> + Display>(&mut self, frame: Frame<T>)->Result<(),RRError>{
        let frame:repr::Frame = frame.into();
        let b = frame.encode_length_delimited_to_vec();
        self.socket.write_all(b.as_slice()).await.map_err(|_| RRErrorKind::NetworkError(
            NetworkErrorKind::FrameWriteError,
        ))?;
        Ok(())
    }

    /// Suspend until the next [Frame] comes.
    /// When the connection closes you will receive an [error](RRError)
    pub async fn read_frame(&mut self) -> Result<NetworkFrame, RRError>
    where
    {
        let len = self.advance_stream().await?;
        let mut buf = vec![0u8; len];
        self.socket.read_exact(&mut buf).await.map_err(|_| RRErrorKind::NetworkError(
            NetworkErrorKind::FrameReadError,
        ))?;

        Ok(Result::from(repr::Frame::decode(buf.as_slice()).map_err(|_| RRErrorKind::SerializationError(
            SerializationErrorKind::FormatError
        ))?)?)
    }

    /// Advances the stream past the length delimiter and returns the delimiter so you can continue reading
    async fn advance_stream(&mut self)->Result<usize,RRError>{
        let mut buf = [0u8;8];
        for i in 0..10{
            buf[i] = self.socket.read_u8().await.map_err(|_| RRErrorKind::NetworkError(
                NetworkErrorKind::FrameReadError,
            ))?;
            if let Ok(len) = prost::decode_length_delimiter(buf.as_slice()){
                return Ok(len);
            }
        }
        Err(RRErrorKind::SerializationError(
            SerializationErrorKind::FormatError
        ).into())
    }
}