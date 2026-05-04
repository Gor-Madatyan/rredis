pub mod server;
use crate::protocol::error::{NetworkErrorKind, RRErrorKind, SerializationErrorKind};
use crate::protocol::{error::RRError, Frame, NetworkFrame};
use crate::repr;
use prost::Message;
use std::fmt::Display;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

/// The main structure you will work with.
/// You will send [Frame]s and/or receive them through this abstraction.
pub struct Connection{
    socket:TcpStream
}

impl Connection{
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }

    /// encapsulate the [TcpStream]
    pub async fn to(socket: impl ToSocketAddrs) -> Result<Self, RRError> {
        Ok(Self {
            socket: TcpStream::connect(socket).await.map_err(|_| RRErrorKind::NetworkError(
                NetworkErrorKind::ConnectionFailed,
            ))?
        })
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