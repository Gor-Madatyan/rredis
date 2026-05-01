use tokio::net::TcpStream;
use crate::protocol::{Frame, RRError};
use crate::repr;
use prost::{Message};
use tokio::io::AsyncReadExt;

pub struct Connection{
    socket:TcpStream
}

impl Connection{
    pub fn new(socket:TcpStream) -> Self{
        Self{
            socket
        }
    }

    pub async fn read_frame(&mut self)->Result<Frame<String>,RRError> where{
        let len = self.advance_stream().await?;
        let mut buf = vec![0u8; len];
        self.socket.read_exact(&mut buf).await?;

        Ok(Result::from(repr::Frame::decode(buf.as_slice())?)?)
    }

    /// Advances the stream past the length delimiter and returns the delimiter so you can continue reading
    async fn advance_stream(&mut self)->Result<usize,RRError>{
        let mut buf = [0u8;8];
        for i in 0..10{
            buf[i]=self.socket.read_u8().await?;
            if let Ok(len) = prost::decode_length_delimiter(buf.as_slice()){
                return Ok(len);
            }
        }
        Err(RRError::new("Invalid length delimiter"))
    }
}