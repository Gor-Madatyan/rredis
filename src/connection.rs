use tokio::net::TcpStream;
use crate::protocol::Frame;

pub struct Connection{
    socket:TcpStream
}

impl Connection{
    pub fn new(socket:TcpStream) -> Self{
        Self{
            socket
        }
    }

    pub async fn read_frame<T:ToString>(&self)->Frame<T>{
        todo!()
    }
}